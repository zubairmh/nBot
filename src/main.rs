use actix_web::{get, post, web, App, HttpServer, Responder};
use rabbitmq_stream_client::{types::Message, Environment};
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    acc_no: i32,
    balance: i32
}

#[derive(Serialize, Deserialize)]
struct Location {
    long: f32,
    lat: f32,
}

#[derive(Serialize, Deserialize)]
struct PaymentRequest {
    from: User,
    to: User,
    medium: String,
    location:  Location,
    amount: i32,
    on: String,
    transaction_id: i32
}


// Server Routes
#[post("/api/pay")]
async fn pay(info: web::Json<PaymentRequest>) -> impl Responder {
    let environment = Environment::builder().build().await.unwrap();
    let mut producer = environment
        .producer()
        .name("t1")
        .build("transactions")
        .await
        .unwrap();
    let _ = producer
        .send_with_confirm(Message::builder().body(format!("{}",serde_json::to_string(&info).unwrap())).build())
        .await;
    format!("Hello")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(pay))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
