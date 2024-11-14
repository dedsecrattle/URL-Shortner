use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::response::Redirect;
use rocket::serde::json::{json, Json, Value};
use rocket::{get, launch, post, routes, State};
use std::collections::HashMap;
use std::sync::Mutex;

type UrlDatabase = Mutex<HashMap<String, String>>;

#[derive(serde::Deserialize)]
struct UrlRequest {
    long_url: String,
}


// Generate a random short code
fn generate_short_code() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    rand_string
}

#[get("/")]
fn index() -> &'static str {
    "URL Shortener API - POST to /shorten with {\"long_url\": \"your-url\"} to create a short URL"
}

#[post("/shorten", format = "json", data = "<url_request>")]
fn shorten(url_request: Json<UrlRequest>, db: &State<UrlDatabase>) -> Value {
    let mut urls = db.lock().unwrap();

    let mut short_code = generate_short_code();
    while urls.contains_key(&short_code) {
        short_code = generate_short_code();
    }

    urls.insert(short_code.clone(), url_request.long_url.clone());

    json!({
        "short_url": format!("/s/{}", short_code),
        "long_url": url_request.long_url
    })
}

#[get("/s/<short_code>")]
fn redirect(short_code: String, db: &State<UrlDatabase>) -> Option<Redirect> {
    let urls = db.lock().unwrap();

    urls.get(&short_code)
        .map(|long_url| Redirect::to(long_url.clone()))
}

#[get("/urls")]
fn list_urls(db: &State<UrlDatabase>) -> Value {
    let urls = db.lock().unwrap();

    json!({
        "urls": urls.iter()
            .map(|(short, long)| json!({
                "short_url": format!("/s/{}", short),
                "long_url": long
            }))
            .collect::<Vec<_>>()
    })
}

#[launch]
fn rocket() -> _ {
    let url_db = UrlDatabase::new(HashMap::new());

    rocket::build()
        .mount("/", routes![index, shorten, redirect, list_urls])
        .manage(url_db)
}
