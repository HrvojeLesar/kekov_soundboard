extern crate actix;
extern crate actix_web;
extern crate actix_web_actors;
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

use actix::prelude::*;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_files::Files as ActixFiles;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use handlebars::{Handlebars, Helper, RenderContext, RenderError, Output, Context};
use rand::Rng;

use std::time::Duration;
use std::net::{TcpStream};
use std::sync::RwLock;
use env_logger;

use crate::prelude::*;

pub mod dumpster_base;
pub use dumpster_base::RwLockedDumpster;
pub mod controls;
pub mod index;
pub mod login;
pub mod upload;
pub mod prelude;
pub mod logged_guard;
pub mod websocket_voice_monitor;
pub mod websocket_voice_monitor_server;

const MAIN_DIR: &str = "../sounds";
pub const PORT: u16 = 1337;

/// Creates a tcp stream for communication with the discord bot
pub fn create_tcp_stream() -> Result<TcpStream, HttpResponse> {
    let mut trys: u8 = 0;
    while {
        match TcpStream::connect(format!("localhost:{}", crate::PORT)) {
            Ok(stream) => {
                stream.set_read_timeout(Some(Duration::new(5, 0))).expect("Error setting read timeout!");
                stream.set_write_timeout(Some(Duration::new(5, 0))).expect("Error setting write timeout!");
                return Ok(stream);
            },
            Err(e) => {
                println!("{}", e);
            }
        }

        trys += 1;

        trys < 5
    } {}

    Err(HttpResponse::BadRequest().finish())
}

/// Displays 404 for any invalid url
async fn four_o_four() -> HttpResponse {
    HttpResponse::NotFound().body("<h1>404</h1>")
}

pub async fn volimo_znidarica(hb: web::Data<Handlebars<'_>>) -> HttpResponse {	
    HttpResponse::Ok().body(hb.render("volimoZnidarica", &()).unwrap())	
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


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("counter", Box::new(counter_helper));
    handlebars.register_templates_directory(".html", "./templates").unwrap();
    
    let handlebars_ref = web::Data::new(handlebars);
    let dumpster_db2_ref = web::Data::new(
        RwLockedDumpster {
            dumpster_base_struct: RwLock::new(dumpster_base::read_db()),
        }
    );
    
    let connected_members = web::Data::new(RwLock::new(websocket_voice_monitor::ConnectedMembers::default()));
    let websocket_voice_monitor_server = web::Data::new(websocket_voice_monitor_server::VoiceMonitorServer::new(connected_members.clone()).start());
    websocket_voice_monitor::local_communication(connected_members.clone(), websocket_voice_monitor_server.clone());

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let private_key = rand::thread_rng().gen::<[u8; 32]>();

    let login_user_pass: web::Data<login::GarbageLogin> = web::Data::new(serde_json::from_str(&std::fs::read_to_string("login.json").unwrap()).unwrap());
    HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("pojecme")
                    .login_deadline(time::Duration::hours(168))
                    .max_age(604_800)
                    .secure(true),
            ))
            .wrap(actix_web::middleware::Logger::default())
            .data(web::JsonConfig::default().limit(1024))
            .app_data(handlebars_ref.clone())
            .app_data(dumpster_db2_ref.clone())
            .app_data(login_user_pass.clone())
            .app_data(connected_members.clone())
            .app_data(websocket_voice_monitor_server.clone())
            .service(web::resource("").route(web::get().to(index::index)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/").route(web::get().to(index::index)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/send-req").route(web::post().to(controls::play_request)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/queue").route(web::get().to(controls::queue)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/skip").route(web::get().to(controls::skip)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/stop").route(web::get().to(controls::stop)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/edit").route(web::get().to(controls::edit)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/rename").route(web::post().to(controls::rename)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/remove").route(web::post().to(controls::remove)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/get-buttons").route(web::get().to(controls::serve_buttons)).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/ws-voice-monitor/").to(websocket_voice_monitor::start_voice_monitor).wrap(logged_guard::LoggedGuard))
            .service(web::resource("/volimoZnidarica").route(web::get().to(volimo_znidarica)))
            .service(
                web::resource("/login")
                .route(web::get().to(login::login_get))
                .route(web::post().to(login::login_post))
   	        )
            .service(web::resource("/logout").route(web::post().to(login::logout)))
	        .service(
		        web::resource("/upload").wrap(logged_guard::LoggedGuard)
                .route(web::get().to(upload::upload_get))
                .route(web::post().to(upload::upload_post))
	        )
            .default_service(web::route().to(four_o_four))
            .service(ActixFiles::new("/static", "./static/"))
    })
    .bind("localhost:6969")?
    .run()
    .await
}

