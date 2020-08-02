use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayRequest {
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeDisplayName {
    value: String,
    new_display_name: String,
}

const EMPTY_QUEUE: &str = "Queue is empty!";

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

pub async fn play_request(
    id: Identity,
    info: web::Json<PlayRequest>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
    // async fn button_test2(info: web::Json<PathNew>, hm: web::Data<HashMap<String, Files>>) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish()
        }
    }

    if !hm
        .dumpster_base_struct
        .read()
        .unwrap()
        .contains_key(&info.value)
    {
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
        println!(
            "Number of recieved bytes: {}",
            tcp_stream.read(&mut buffer).unwrap()
        );
        let message = String::from_utf8(buffer.to_vec()).unwrap();
        println!("{}", message);
        tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
        HttpResponse::Ok().json(json!({ "success": "Ok" }))
    } else {
        return HttpResponse::BadRequest().finish();
    }
}
pub async fn queue(id: Identity) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish()
        }
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
        let mut message_buf: Vec<u8> = vec![0; len as usize];

        // paziti na 2 read !!!!!!
        // tcp_stream.read(&mut message_buf).unwrap();
        println!(
            "Number of recieved bytes: {}",
            tcp_stream.read(&mut message_buf).unwrap()
        );
        tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
        HttpResponse::Ok().json(jsonfy_queue(message_buf))
    } else {
        return HttpResponse::BadRequest().finish();
    }
}

pub async fn skip(id: Identity) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish()
        }
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
pub async fn stop(id: Identity) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish()
        }
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

pub async fn rename(
    id: Identity,
    info: web::Form<ChangeDisplayName>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish()
        }
    }
    let mut hash_map = hm.dumpster_base_struct.write().unwrap();

    hash_map.get_mut(&info.value).unwrap().display_name = info.new_display_name.clone();
    // sejvanje je sporo kaj puz
    match dumpster_base::update_dumpster_db(&mut *hash_map) {
        Ok(_) => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/edit")
                .finish()
        }
        Err(_) => return HttpResponse::BadRequest().finish(),
    }
}

pub async fn edit(
    id: Identity,
    hb: web::Data<Handlebars<'_>>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish()
        }
    }
    HttpResponse::Ok().body(hb.render("edit", &index::dumpster_index(hm)).unwrap())
}

pub async fn remove(
    id: Identity,
    info: web::Form<ChangeDisplayName>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
    match id.identity() {
        Some(_) => (),
        None => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/login")
                .finish()
        }
    }
    let mut hash_map = hm.dumpster_base_struct.write().unwrap();

    match hash_map.remove_entry(&info.value) {
        Some(entry) => println!("Removing entry from dumpster base: {:?}", entry),
        None => return HttpResponse::BadRequest().finish(),
    }
    // sejvanje je sporo kaj puz
    match dumpster_base::update_dumpster_db(&mut *hash_map) {
        Ok(_) => {
            return HttpResponse::SeeOther()
                .header(http::header::LOCATION, "/edit")
                .finish()
        }
        Err(_) => return HttpResponse::BadRequest().finish(),
    }
}
