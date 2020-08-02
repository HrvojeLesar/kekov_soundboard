use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct GarbageLogin {
    user: String,
    pass: String,
}

pub async fn login_get(id: Identity, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    if id.identity().is_some() {
        return HttpResponse::SeeOther().header(http::header::LOCATION, "/").finish();
    }
    HttpResponse::Ok().body(hb.render("login", &()).unwrap())
}

pub async fn login_post(id: Identity, form: web::Form<GarbageLogin>, pass: web::Data<GarbageLogin>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    if (form.user == pass.user && form.pass == pass.pass) || id.identity().is_some() {
        id.remember("epik gazda".to_owned());
        return HttpResponse::SeeOther().header(http::header::LOCATION, "/").finish();
    }
    HttpResponse::Ok().body(hb.render("login", &json!({"invalid": true})).unwrap())
}

pub async fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish()
}