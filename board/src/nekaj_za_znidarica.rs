use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginZnidaric {
    user: String,
    pass: String,
}

pub async fn volimo_znidarica(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    HttpResponse::Ok().body(hb.render("volimoZnidarica", &()).unwrap())
}

pub async fn login_znidaric_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    HttpResponse::Ok().body(hb.render("loginZnidaric", &()).unwrap())
}

pub async fn login_znidaric_post(form: web::Form<LoginZnidaric>, pass: web::Data<LoginZnidaric>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    if form.user == pass.user && form.pass == pass.pass {
        return HttpResponse::SeeOther().header(http::header::LOCATION, "/banajMatijosa").finish();
    }
    HttpResponse::Ok().body(hb.render("loginZnidaric", &json!({"invalid": true})).unwrap())
}

pub async fn banaj_matijosa_get(hb: web::Data<Handlebars<'_>>, req: HttpRequest) -> HttpResponse {
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

pub async fn banaj_matijosa_post(req: HttpRequest) -> HttpResponse {
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
