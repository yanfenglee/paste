use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};

use askama::Template;
use serde::{Deserialize, Serialize}; // bring trait in scope

mod local_cache;

#[derive(Template)]
#[template(path = "edit.html")]
struct EditTemplate<'a> {
    token: &'a str,
    content: &'a str,
}

#[derive(Deserialize, Serialize)]
struct Info {
    content: String,
}

#[get("/")]
async fn readme() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/readme.html"))
}

#[get("/{token}")]
async fn index(req: HttpRequest, web::Path(token): web::Path<String>) -> impl Responder {
    //format!("Hello {}! ", token)
    let content = local_cache::get::<String>(&token).unwrap_or("".to_owned());

    if let Some(agent) = req.headers().get("User-Agent") {
        if let Ok(agent) = agent.to_str() {
            if agent.starts_with("curl/") {

                let res = HttpResponse::build(StatusCode::OK)
                .content_type("text/plain; charset=utf-8")
                .body(content);

                return res 
            }
        }
    }

    let html = EditTemplate {
        token: &token,
        content: &content,
    }
    .render()
    .unwrap();

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/{token}")]
async fn save(web::Path(token): web::Path<String>, content: String) -> impl Responder {
    let _ = local_cache::set(&token, &content);
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index).service(save).service(readme))
        .bind("0.0.0.0:3322")?
        .run()
        .await
}
