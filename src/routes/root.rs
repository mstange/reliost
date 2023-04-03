use actix_web::Responder;

pub async fn greet() -> impl Responder {
    "Hello world!".to_string()
}
