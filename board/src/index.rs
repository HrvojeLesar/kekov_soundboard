use crate::prelude::*;

pub fn dumpster_index(hm: web::Data<dumpster_base::RwLockedDumpster>) -> serde_json::Value {
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

pub async fn index(id: Identity, hb: web::Data<Handlebars<'_>>, hm: web::Data<dumpster_base::RwLockedDumpster>) -> HttpResponse {
    // retarderano, trenutno neznam kak drugac sloziti
    match id.identity() {
        Some(_) => (),
        None => return HttpResponse::SeeOther().header(http::header::LOCATION, "/login").finish(),
    }
    HttpResponse::Ok().body(hb.render("index", &dumpster_index(hm)).unwrap())
}