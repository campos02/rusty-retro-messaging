use axum::response::IntoResponse;
use std::env;

pub async fn nexus() -> impl IntoResponse {
    let server_name = env::var("SERVER_NAME").expect("SERVER_NAME not set");
    [("PassportURLs", format!("DALogin={server_name}/login.srf"))]
}
