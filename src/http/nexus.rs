use axum::response::IntoResponse;
use std::env;

pub async fn nexus() -> impl IntoResponse {
    let server_name = env::var("SERVER_DOMAIN").expect("SERVER_DOMAIN not set");
    [("PassportURLs", format!("DALogin={server_name}/login.srf"))]
}
