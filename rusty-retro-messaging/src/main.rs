#[tokio::main]
async fn main() {
    msnp_server::listen().await;
}
