use dotenv::dotenv;
use wildberries_test::server::Server; 

#[actix_web::main]
async fn main() {
    dotenv().ok();
    let app = Server::builder()
        .host("0.0.0.0".to_owned())
        .port(8080)
        .build();
    app.start().await.unwrap();
}
