extern crate actix_web;
extern crate actix_files;
extern crate actix_identity;
extern crate actix_multipart;
extern crate actix_rt;
extern crate futures;
extern crate mime;
extern crate time;
extern crate sanitize_filename;

extern crate rand;
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate handlebars;

use actix_web::{web, get, http, App, HttpResponse, HttpServer, HttpRequest};
use actix_web::middleware::errhandlers::{ErrorHandlers, ErrorHandlerResponse};
use actix_files::Files as ActixFiles;
use actix_identity::Identity;
use actix_identity::{CookieIdentityPolicy, IdentityService, RequestIdentity};
use actix_multipart::Multipart;
use actix_service::Service;
use futures::{StreamExt, TryStreamExt};
use handlebars::{Handlebars, Helper, RenderContext, RenderError, Output, Context};
use rand::Rng;

use serde::{Serialize, Deserialize};
// use std::collections::hash_map::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, Shutdown};
use std::sync::RwLock;
use env_logger;

mod dumpster_base;
use dumpster_base::RwLockedDumpster;

pub const MAIN_DIR: &str = "../sounds";
const EMPTY_QUEUE: &str = "Queue is empty!";
////const SCOPE: &str = "/bot";
const MAX_FILE_LENGTH: usize = 10_485_760; // 10MB
const SAVE_SOUNDS: &str = "../sounds/";
// const MAX_FILE_LENGTH: usize = 800; // test

#[derive(Debug, Serialize, Deserialize)]
struct PlayRequest {
    value: String,
}

// directory mortik za kesnise
// ako kaj slozim da se more delati z direktorijaj
#[derive(Debug)]
struct Files {
    full_file_name: String,
    without_extention: String,
    directory: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChangeDisplayName {
    value: String,
    new_display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GarbageLogin {
    user: String,
    pass: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginZnidaric {
    user: String,
    pass: String,
}

fn jsonfy_queue(recieved_message: Vec<u8>) -> serde_json::Value {
    let mut queue_vec: Vec<&str> = Vec::new();
    let parsed_bytes = String::from_utf8(recieved_message).unwrap();
    parsed_bytes.split('?').for_each(|q| {
        queue_vec.push(q);
    });

    json!({
        "success": "queue_success",
        "queue": queue_vec
    })
}

fn create_tcp_stream() -> Option<TcpStream> {
    let tcp_stream = match TcpStream::connect("localhost:1337") {
        Ok(stream) => Some(stream),
        Err(e) => {
            println!("{}", e);
            None
        },
    };
    tcp_stream
}

fn dumpster_index(hm: web::Data<RwLockedDumpster>) -> serde_json::Value {
    let mut values_vec = Vec::new();
    let hash_map = hm.dumpster_base_struct.read().unwrap();
    hash_map.values().for_each(|val| {
        values_vec.push(val.clone());
    });

    values_vec.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    json!({
        "paths": &values_vec,
    })
}

fn update_db(filename: String, hm: web::Data<RwLockedDumpster>) {

    let mut hash_map = hm.dumpster_base_struct.write().unwrap();
    let file_without_extention = filename.split(".").collect::<Vec<&str>>()[0].to_owned();

    hash_map.insert(
        filename.clone(),            
        dumpster_base::DumpsterBaseJson {
                full_file_name: filename,
                without_extention: file_without_extention.clone(),
                display_name: file_without_extention,
    });
    // sejvanje je sporo kaj puz
    dumpster_base::update_dumpster_db(&mut *hash_map).unwrap();
}

async fn index(id: Identity, hb: web::Data<Handlebars<'_>>, hm: web::Data<RwLockedDumpster>) -> HttpResponse {
    // retarderano, trenutno neznam kak drugac sloziti
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }
    // async fn index(hb: web::Data<Handlebars<'_>>, hm: web::Data<HashMap<String, Files>>) -> HttpResponse {
    HttpResponse::Ok().body(hb.render("index", &dumpster_index(hm)).unwrap())
    // HttpResponse::Ok().body(hb.render("index", &load_index_new(hm)).unwrap())
}

async fn index_redirect(id: Identity) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }
    HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish()
}

async fn play_request(id: Identity, info: web::Json<PlayRequest>, hm: web::Data<RwLockedDumpster>) -> HttpResponse {
// async fn button_test2(info: web::Json<PathNew>, hm: web::Data<HashMap<String, Files>>) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }

    if !hm.dumpster_base_struct.read().unwrap().contains_key(&info.value) {
        return HttpResponse::BadRequest().finish();
    }

    if let Some(mut tcp_stream) = create_tcp_stream() {
        // let data = json!({
            //     "command": "play",
            //     "value": &info.value,
            //     "file_name": &hm.get(&info.value).unwrap(),
            // });
            
        let data = json!({
            "command": "play",
            "value": &info.value,
            "file_name": &hm.dumpster_base_struct.read().unwrap().get(&info.value).unwrap().full_file_name,
        });
        
        tcp_stream.write(&data.to_string().as_bytes()).expect("Err");
        tcp_stream.flush().unwrap();
        
        let mut buffer = [0; 8];
        println!("Number of recieved bytes: {}", tcp_stream.read(&mut buffer).unwrap());
        
        let message = String::from_utf8(buffer.to_vec()).unwrap();
        println!("{}", message);
        
        tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
        HttpResponse::Ok().json(json!({ "success": "Ok" }))
    } else {
        return HttpResponse::BadRequest().finish()
    }
}

async fn queue(id: Identity) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }

    if let Some(mut tcp_stream) = create_tcp_stream() {
        let data = json!({
            "command": "queue"
        });

        tcp_stream.write(&data.to_string().as_bytes()).expect("Err");
        tcp_stream.flush().unwrap();
        
        let mut message_length = [0; 8];
        tcp_stream.read(&mut message_length).unwrap();

        let mut length: Vec<u8> = Vec::new();
        message_length.iter().for_each(|b| {
            if *b > 0u8 {
                length.push(*b);
            }
        });

        let len = String::from_utf8(length).unwrap().parse::<u32>().unwrap();
        if len == 0 {
            return HttpResponse::Ok().json(json!({
                "success": "queue_success",
                "queue": EMPTY_QUEUE
            }));
        }
        // println!("Len: {}", len);
        let mut message_buf: Vec<u8> = vec![0;len as usize];

        // paziti na 2 read !!!!!!
        // tcp_stream.read(&mut message_buf).unwrap();
        println!("Number of recieved bytes: {}", tcp_stream.read(&mut message_buf).unwrap());
        
        tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
        HttpResponse::Ok().json(jsonfy_queue(message_buf))
    } else {
        return HttpResponse::BadRequest().finish();
    }
}

async fn skip(id: Identity) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }

    if let Some(mut tcp_stream) = create_tcp_stream() {
        let data = json!({
            "command": "skip"
        });
        
        tcp_stream.write(&data.to_string().as_bytes()).expect("Err");
        tcp_stream.flush().unwrap();
        
        let mut message_bits = [0; 8];
        tcp_stream.read(&mut message_bits).unwrap();
        
        let mut message: Vec<u8> = Vec::new();
        message_bits.iter().for_each(|b| {
            if *b > 0u8 {
                message.push(*b);
            }
        });
        
        match String::from_utf8(message).unwrap().parse::<u32>().unwrap() {
            4 => return HttpResponse::Ok().json(json!({ "success": "skip_success" })),
            5 => return HttpResponse::Ok().json(json!({ "success": "skip_empty" })),
            _ => return HttpResponse::BadRequest().finish(),
        };
    } else {
        return HttpResponse::BadRequest().finish();
    }
}

async fn stop(id: Identity) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }

    if let Some(mut tcp_stream) = create_tcp_stream() {
        let data = json!({
            "command": "stop"
        });
            
        tcp_stream.write(&data.to_string().as_bytes()).expect("Err");
        tcp_stream.flush().unwrap();
            
        let mut message_bits = [0; 8];
        tcp_stream.read(&mut message_bits).unwrap();
            
        let mut message: Vec<u8> = Vec::new();
        message_bits.iter().for_each(|b| {
            if *b > 0u8 {
                message.push(*b);
            }
        });
            
        match String::from_utf8(message).unwrap().parse::<u32>().unwrap() {
            6 => return HttpResponse::Ok().json(json!({ "success": "stop_success" })),
            7 => return HttpResponse::Ok().json(json!({ "success": "stop_empty" })),
            _ => return HttpResponse::BadRequest().finish(),
        };
    } else {
        return HttpResponse::BadRequest().finish();
    }
}

async fn rename(id: Identity, info: web::Form<ChangeDisplayName>, hm: web::Data<RwLockedDumpster>) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }
    let mut hash_map = hm.dumpster_base_struct.write().unwrap();

    hash_map.get_mut(&info.value).unwrap().display_name = info.new_display_name.clone();
    // sejvanje je sporo kaj puz
    match dumpster_base::update_dumpster_db(&mut *hash_map) {
        Ok(_) => return HttpResponse::SeeOther().header(http::header::LOCATION, "/edit").finish(),
        Err(_) => return HttpResponse::BadRequest().finish(),
    }
}

async fn login_get(id: Identity, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    if id.identity().is_some() {
        return HttpResponse::SeeOther().header(http::header::LOCATION, "/").finish();
    }
    HttpResponse::Ok().body(hb.render("login", &()).unwrap())
}

async fn login_post(id: Identity, form: web::Form<GarbageLogin>, pass: web::Data<GarbageLogin>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    if (form.user == pass.user && form.pass == pass.pass) || id.identity().is_some() {
        id.remember("epik gazda".to_owned());
        return HttpResponse::SeeOther().header(http::header::LOCATION, "/").finish();
    }
    HttpResponse::Ok().body(hb.render("login", &json!({"invalid": true})).unwrap())
}

async fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish()
}

async fn edit(id: Identity, hb: web::Data<Handlebars<'_>>, hm: web::Data<RwLockedDumpster>) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }
    HttpResponse::Ok().body(hb.render("edit", &dumpster_index(hm)).unwrap())
}

async fn remove(id: Identity, info: web::Form<ChangeDisplayName>, hm: web::Data<RwLockedDumpster>) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }
    let mut hash_map = hm.dumpster_base_struct.write().unwrap();

    match hash_map.remove_entry(&info.value) {
        Some(entry) => println!("Removing entry from dumpster base: {:?}", entry),
        None => return HttpResponse::BadRequest().finish()
    }
    // sejvanje je sporo kaj puz
    match dumpster_base::update_dumpster_db(&mut *hash_map) {
        Ok(_) => return HttpResponse::SeeOther().header(http::header::LOCATION, "/edit").finish(),
        Err(_) => return HttpResponse::BadRequest().finish(),
    }
}

async fn upload_get(id: Identity, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }
    HttpResponse::Ok().body(hb.render("upload", &()).unwrap())
}

async fn upload_post(id: Identity, mut payload: Multipart, req: HttpRequest, hm: web::Data<RwLockedDumpster>) -> Result<HttpResponse, actix_web::Error> {
    match id.identity() {
        Some(_) => (),
        None => return Ok(HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish()),
    }

    if req.headers().get("Content-Length").unwrap().to_str().unwrap().parse::<usize>().unwrap() > MAX_FILE_LENGTH {
        println!("File size too large");
        return Err(actix_web::Error::from(HttpResponse::InternalServerError().finish()));
    }

    let mime = "audio/*".parse::<mime::Mime>().unwrap();
    let mut file_length = 0;
    let mut db_filename = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        if !mime.type_().eq(&field.content_type().type_()) {
            return Err(actix_web::Error::from(HttpResponse::InternalServerError().finish()));
        }
        let content_type = field.content_disposition().unwrap();
        let mut filename = sanitize_filename::sanitize(content_type.get_filename().unwrap());
        let mut filepath = format!("{}{}", SAVE_SOUNDS, filename);
        while std::path::Path::new(&filepath).exists() {
            let temp_file_name = filename.clone();
            let split_extention = temp_file_name.split(".").collect::<Vec<&str>>();
            filename = format!("{}69.{}", split_extention[0], split_extention[1]);
            filepath = format!("{}{}69.{}", SAVE_SOUNDS, split_extention[0], split_extention[1]);
        }
        let invalid = filepath.clone();
        db_filename = filename.clone();
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_length += data.len();
            if file_length > MAX_FILE_LENGTH {
                std::fs::remove_file(invalid).unwrap();
                return Err(actix_web::Error::from(HttpResponse::InternalServerError().finish()));
            }
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }
    update_db(db_filename, hm);
    Ok(HttpResponse::Ok().into())
}

async fn volimo_znidarica(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    HttpResponse::Ok().body(hb.render("volimoZnidarica", &()).unwrap())
}

async fn login_znidaric_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    HttpResponse::Ok().body(hb.render("loginZnidaric", &()).unwrap())
}

async fn login_znidaric_post(form: web::Form<LoginZnidaric>, pass: web::Data<LoginZnidaric>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    if form.user == pass.user && form.pass == pass.pass {
        return HttpResponse::SeeOther().header(http::header::LOCATION, "/banajMatijosa").finish();
    }
    HttpResponse::Ok().body(hb.render("loginZnidaric", &json!({"invalid": true})).unwrap())
}

async fn banaj_matijosa_get(hb: web::Data<Handlebars<'_>>, req: HttpRequest) -> HttpResponse {
    match req.headers().get("referer") {
        Some(r) => {
            if r.to_str().unwrap().contains("loginZnidaric") {
                return HttpResponse::Ok().body(hb.render("banajMatijosa", &()).unwrap());
            } else {
                return HttpResponse::SeeOther().header(http::header::LOCATION, "/loginZnidaric").finish();
            }
        },
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/loginZnidaric").finish(),
    }
}

async fn banaj_matijosa_post(req: HttpRequest) -> HttpResponse {
    match req.headers().get("referer") {
        Some(r) => {
            if r.to_str().unwrap().contains("banajMatijosa") {
                if let Some(mut tcp_stream) = create_tcp_stream() {
                    let data = json!({
                        "command": "banaj",
                    });
                    
                    tcp_stream.write(&data.to_string().as_bytes()).expect("Err");
                    tcp_stream.flush().unwrap();
                    
                    let mut buffer = [0; 8];
                    println!("Number of recieved bytes: {}", tcp_stream.read(&mut buffer).unwrap());
                    
                    println!("{}", buffer[0]);
        
                    tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
        
                    if buffer[0] == 56 { // 8
                        return HttpResponse::Ok().json(json!({ "success": "Ok" }));
                    } else if buffer[0] == 57 { // 9
                        return HttpResponse::Ok().json(json!({ "success": "Mortik je vec banati" }));
                    } else {
                        return HttpResponse::BadRequest().finish();
                    }
                } else {
                    return HttpResponse::BadRequest().finish()
                }
            } else {
                return HttpResponse::BadRequest().finish();
            }
        },
        None => return HttpResponse::BadRequest().finish(),
    }
}

async fn four_o_four() -> HttpResponse {
    HttpResponse::NotFound().body("<h1>404</h1>")
}


fn counter_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let count = h
        .param(0)
        .and_then(|ref v| v.value().as_u64())
        .ok_or(RenderError::new("Param 0 with u64 type is required"))?;
    out.write(&(count + 1).to_string())?;
    Ok(())
}


// auth -> tablica z tipkaj (public i private tipke)
// talica public (baza za public i svakome useru, nova tablica z posebnaj pathaj)
// DB -> public -> ID | name | PATH
// DB -> private -> isto kak public sam naziv tablice bode genererani
// display queue
// skip

// dodaj na sakoga 401 ili nikaj ako je autorizerani
// guard pogledne ako ima 401 i redirecta

// napraviti hash tablicu fajli koje mores pokrenuti

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("counter", Box::new(counter_helper));
    handlebars.register_templates_directory(".html", "./templates").unwrap();
    
    let handlebars_ref = web::Data::new(handlebars);
    // let file_hash_ref = web::Data::new(create_file_hash());
    // let dumpster_db_ref = web::Data::new(dumpster_base::read_db());
    let dumpster_db2_ref = web::Data::new(
        RwLockedDumpster {
            dumpster_base_struct: RwLock::new(dumpster_base::read_db()),
        }
    );

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // std::fs::create_dir_all("./tmp").unwrap();

    let private_key = rand::thread_rng().gen::<[u8; 32]>();

    let login_user_pass: web::Data<GarbageLogin> = web::Data::new(serde_json::from_str(&std::fs::read_to_string("login.json").unwrap()).unwrap());
    let login_znidaric: web::Data<LoginZnidaric> = web::Data::new(serde_json::from_str(&std::fs::read_to_string("loginZnidaric.json").unwrap()).unwrap());
    HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("pojecme")
                    .login_deadline(time::Duration::hours(24))
                    .max_age(86400)
                    .secure(false),
            ))
            .wrap(actix_web::middleware::Logger::default())
            .data(web::JsonConfig::default().limit(1024))
            .app_data(handlebars_ref.clone())
            // .app_data(file_hash_ref.clone())
            .app_data(dumpster_db2_ref.clone())
            .app_data(login_user_pass.clone())
            .app_data(login_znidaric.clone())
//            .service(web::scope(SCOPE)
            .service(web::resource("").route(web::get().to(index_redirect)))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/sendReq").route(web::post().to(play_request)))
            .service(web::resource("/queue").route(web::get().to(queue)))
            .service(web::resource("/skip").route(web::get().to(skip)))
            .service(web::resource("/stop").route(web::get().to(stop)))
            .service(web::resource("/edit").route(web::get().to(edit)))
            .service(web::resource("/rename").route(web::post().to(rename)))
            .service(
                web::resource("/login")
                .route(web::get().to(login_get))
                .route(web::post().to(login_post))
   	        )
            .service(web::resource("/logout").route(web::post().to(logout)))
	        .service(web::resource("/remove").route(web::post().to(remove)))
	        .service(
		        web::resource("/upload")
                .route(web::get().to(upload_get))
                .route(web::post().to(upload_post))
	        )
            .service(web::resource("/volimoZnidarica").route(web::get().to(volimo_znidarica)))
            .service(
                web::resource("/loginZnidaric")
                .route(web::get().to(login_znidaric_get))
                .route(web::post().to(login_znidaric_post))
            )
            .service(
                web::resource("/banajMatijosa")
                .route(web::get().to(banaj_matijosa_get))
                .route(web::post().to(banaj_matijosa_post))
            )
            .default_service(web::route().to(four_o_four))
            .service(ActixFiles::new("/static", "./static/"))
//            )
    })
    .bind("localhost:6969")?
    .run()
    .await
}

