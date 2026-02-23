use backend_rust::bootstrap;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    bootstrap().await
}
