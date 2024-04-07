use core::panic;
use tokio::time;

use actix_web::{get, post, web, App, HttpServer, Responder};
use serde::{Serialize,Deserialize};
use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicConsumeArguments, BasicPublishArguments, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::DefaultConsumer,
    BasicProperties,
};


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
    transaction_id: i64
}


// Server Routes
#[post("/api/pay")]
async fn pay(info: web::Json<PaymentRequest>) -> impl Responder {
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();
    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();
    
    // open a channel on the connection
    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

        // declare a durable queue
    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::durable_client_named(
            "amqprs.examples.basic",
        ))
        .await
        .unwrap()
        .unwrap();


    let (response_queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::durable_client_named(
            "amqprs.examples.response",
        ))
        .await
        .unwrap()
        .unwrap();

    // bind the queue to exchange
    let routing_key = "amqprs.example";
    let exchange_name = "amq.topic";
    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            routing_key,
        ))
        .await
        .unwrap();

    //////////////////////////////////////////////////////////////////////////////
    // start consumer with given name
    let args = BasicConsumeArguments::new(&response_queue_name, "example_basic_pub_sub");

    channel
        .basic_consume(DefaultConsumer::new(args.no_ack), args)
        .await
        .unwrap();

    //////////////////////////////////////////////////////////////////////////////
    // publish message
    let content = serde_json::to_string(&info).unwrap().into_bytes();

    // create arguments for basic_publish
    let args = BasicPublishArguments::new(exchange_name, routing_key);

    channel
        .basic_publish(BasicProperties::default(), content, args)
        .await
        .unwrap();

    // keep the `channel` and `connection` object from dropping before pub/sub is done.
    // channel/connection will be closed when drop.
    time::sleep(time::Duration::from_secs(1)).await;
    // explicitly close
    channel.close().await.unwrap();
    connection.close().await.unwrap();
    format!("Hello")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(pay))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
