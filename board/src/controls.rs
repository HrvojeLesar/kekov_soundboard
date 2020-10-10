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

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveItem {
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Queue {
    queue: Vec<String>,
}

const EMPTY_QUEUE: &str = "Queue is empty!";

// fn jsonfy_queue(recieved_message: Vec<u8>) -> serde_json::Value {
//     let mut queue_vec: Vec<&str> = Vec::new();
//     let parsed_bytes = String::from_utf8(recieved_message).unwrap();
//     parsed_bytes.split('?').for_each(|q| {
//         queue_vec.push(q);
//     });

//     json!({
//         "success": "queue_success",
//         "queue": queue_vec
//     })
// }

pub async fn play_request(
    info: web::Json<PlayRequest>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
    if !hm
        .dumpster_base_struct
        .read()
        .unwrap()
        .contains_key(&info.value)
    {
        return HttpResponse::BadRequest().finish();
    }

    let mut tcp_stream = match create_tcp_stream() {
        Ok(stream) => {
            stream
        },
        Err(e) => {
            return e;
        }
    };

    let data = json!({
        "command": "play",
        "value": &info.value,
        "file_name": &hm.dumpster_base_struct.read().unwrap().get(&info.value).unwrap().full_file_name,
    });

    handle_tcp_write(&mut tcp_stream, data);
    println!("Tcp prejde");

    return handle_tcp_read(&mut tcp_stream, None);
}

/// Gets queue from bot
/// Uses 2 tcp reads, byte length of queue, queue
pub async fn queue(hm: web::Data<dumpster_base::RwLockedDumpster>) -> HttpResponse {

    let mut tcp_stream = match create_tcp_stream() {
        Ok(stream) => {
            stream
        },
        Err(e) => {
            return e;
        }
    };

    let data = json!({
        "command": "queue"
    });

    handle_tcp_write(&mut tcp_stream, data);

    return handle_tcp_read(&mut tcp_stream, Some(hm));
}

pub async fn skip() -> HttpResponse {

    let mut tcp_stream = match create_tcp_stream() {
        Ok(stream) => {
            stream
        },
        Err(e) => {
            return e;
        }
    };

    let data = json!({
        "command": "skip"
    });

    handle_tcp_write(&mut tcp_stream, data);

    return handle_tcp_read(&mut tcp_stream, None);
}
pub async fn stop() -> HttpResponse {

    let mut tcp_stream = match create_tcp_stream() {
        Ok(stream) => {
            stream
        },
        Err(e) => {
            return e;
        }
    };

    let data = json!({
        "command": "stop"
    });

    handle_tcp_write(&mut tcp_stream, data);

    return handle_tcp_read(&mut tcp_stream, None);
}

pub async fn rename(
    info: web::Form<ChangeDisplayName>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
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
    hb: web::Data<Handlebars<'_>>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
    HttpResponse::Ok().body(hb.render("edit", &index::dumpster_index(hm)).unwrap())
}

pub async fn remove(
    info: web::Form<RemoveItem>,
    hm: web::Data<dumpster_base::RwLockedDumpster>,
) -> HttpResponse {
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

pub async fn serve_buttons(hm: web::Data<dumpster_base::RwLockedDumpster>) -> HttpResponse {
    let hash_map = hm.dumpster_base_struct.read().unwrap();
    let values = hash_map.values().collect::<Vec<&dumpster_base::DumpsterBaseJson>>();

    HttpResponse::Ok().json(json!({
        "paths": &values,
    }))
}

fn handle_tcp_read(stream: &mut TcpStream, hm: Option<web::Data<dumpster_base::RwLockedDumpster>>) -> HttpResponse {
        let mut response = json!({ "error": "error" });
        println!("Response prejde");

        let mut buffer = [0; 8192];
        let bytes_recieved = stream.read(&mut buffer).unwrap();

        let message = String::from_utf8(Vec::from(&mut buffer[0..bytes_recieved])).unwrap();

        match message.chars().next().unwrap() {
            '0' => {
                response = json!({ "success": "playing" });
            },
            '1' => {
                response = json!({ "success": "added" });
            },
            '2' => {
                let hash_map = hm.unwrap();
                let hash_map = hash_map.dumpster_base_struct.read().unwrap();
                let mut queue_display_names: Vec<String> = Vec::new();

                let q = message.get(1..).unwrap();
                let queue: Queue = serde_json::from_str(q).unwrap();

                for value in queue.queue {
                    queue_display_names.push(hash_map.get(&value).unwrap().display_name.clone());
                }

                response = json!({
                    "success": "queue_success",
                    "queue": queue_display_names
                });
            },
            '3' => {
                response = json!({ "error": "error joining" });
            },
            '4' => {
                response = json!({ "success": "skip_success" });
            },
            '5' => {
                response = json!({
                    "success": "queue_success",
                    "queue": EMPTY_QUEUE
                });
            },
            '6' => {
                response = json!({ "success": "stop_success" });
            },
            '7' => {
                response = json!({ "success": "stop_empty" });
            },
            '8' => {

            },
            '9' => {

            },
            _ => {
                json!({ "error": "unknown error" });
            },
        }

    return HttpResponse::Ok().json(response);
}

fn handle_tcp_write(stream: &mut TcpStream, data: serde_json::Value) {
    // stream.write_all(&data.to_string().as_bytes()).await.unwrap();
    stream.write(&data.to_string().as_bytes()).expect("Error writing to stream!");
    stream.flush().unwrap();
}