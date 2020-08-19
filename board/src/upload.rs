    use crate::prelude::*;

const SAVE_SOUNDS: &str = "../sounds/";
const MAX_FILE_LENGTH: usize = 10_485_760; // 10MB

fn update_db(filename: String, hm: web::Data<dumpster_base::RwLockedDumpster>) {

    let mut hash_map = hm.dumpster_base_struct.write().unwrap();
    let file_without_extention = filename.split(".").collect::<Vec<&str>>()[0].to_owned();

    hash_map.insert(
        filename.clone(),            
        dumpster_base::DumpsterBaseJson {
            full_file_name: filename,
            without_extention: file_without_extention.clone(),
            display_name: file_without_extention,
            time_stamp: {
                if let Ok(time_stamp) = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
                    time_stamp.as_secs()
                } else {
                    0
                }
            }
    });
    // sejvanje je sporo kaj puz
    dumpster_base::update_dumpster_db(&mut *hash_map).unwrap();
}


pub async fn upload_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    HttpResponse::Ok().body(hb.render("upload", &()).unwrap())
}

pub async fn upload_post(mut payload: Multipart, req: HttpRequest, hm: web::Data<dumpster_base::RwLockedDumpster>) -> Result<HttpResponse, actix_web::Error> {
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