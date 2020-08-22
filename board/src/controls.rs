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

    if let Some(mut tcp_stream) = create_tcp_stream() {
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
        // 0 | 48 => Started playing (queue was empty before starting)
        // 1 | 49 => Added to queue
        // 2 | 50 => RETARDERANI SAM
        // 3 | 51 => Error joining channel. At least one person needs to be joined in a voice channel!
        println!("{}", buffer[0]);
        let response;
        match buffer[0] {
            48 => response = json!({ "success": "playing" }),
            49 => response = json!({ "success": "added" }),
            51 => response = json!({ "error": "error joining" }),
            _ => response = json!({ "error": "unknown error" }),
        };
        tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
        HttpResponse::Ok().json(response)
    } else {
        return HttpResponse::BadRequest().finish();
    }
}

/// Gets queue from bot
/// Uses 2 tcp reads, byte length of queue, queue
pub async fn queue(hm: web::Data<dumpster_base::RwLockedDumpster>) -> HttpResponse {
    if let Some(mut tcp_stream) = create_tcp_stream() {
        let data = json!({
            "command": "queue"
        });

        tcp_stream.write(&data.to_string().as_bytes()).expect("Err");
        tcp_stream.flush().unwrap();
        let mut message_length = [0; 8];
        // get queue byte length
        println!(
            "Number of recieved bytes 1: {}",
            // get queue elements
            tcp_stream.read(&mut message_length).unwrap()
        );

        let mut length: Vec<u8> = Vec::new();
        message_length.iter().for_each(|b| {
            if *b > 0u8 {
                length.push(*b);
            }
        });

        println!("{:?}", length);
        // Malo retarderano moglo bi bole
        let len = match String::from_utf8(length) {
            Ok(l) => {
                match l.parse::<u32>() {
                    Ok(parsed_length) => parsed_length,
                    Err(e) => {
                        println!("{}", e);
                        tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
                        return HttpResponse::BadRequest().finish();
                    }
                }
            },
            Err(e) => {
                println!("{}", e);
                tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
                return HttpResponse::BadRequest().finish();
            }
        };
        println!("{}", &len);
        // let len = String::from_utf8(length).unwrap().parse::<u32>().unwrap();
        let mut message_buf: Vec<u8> = vec![0; len as usize];

        println!(
            "Number of recieved bytes: {}",
            // get queue elements
            tcp_stream.read(&mut message_buf).unwrap()
        );

        let queue: Queue = serde_json::from_slice(message_buf.as_slice()).unwrap();
        if queue.queue.len() == 0 {
            return HttpResponse::Ok().json(json!({
                "success": "queue_success",
                "queue": EMPTY_QUEUE
            }));
        }

        let hash_map = hm.dumpster_base_struct.read().unwrap();
        let mut queue_display_names: Vec<String> = Vec::new();
        for value in queue.queue {
            queue_display_names.push(hash_map.get(&value).unwrap().display_name.clone());
        }

        tcp_stream.shutdown(Shutdown::Both).expect("Shutdown error");
        HttpResponse::Ok().json(json!({
            "success": "queue_success",
            "queue": queue_display_names }))
    } else {
        return HttpResponse::BadRequest().finish();
    }
}

pub async fn skip() -> HttpResponse {
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
pub async fn stop() -> HttpResponse {
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
