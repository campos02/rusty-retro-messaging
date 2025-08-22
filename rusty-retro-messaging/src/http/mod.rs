use crate::message::Message;
use axum::{
    Router,
    http::HeaderValue,
    middleware,
    routing::{get, post},
};
use hyper::{Request, body::Incoming};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use log::{error, info};
use sqlx::{MySql, Pool};
use std::env;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_service::Service;

mod login_server;
mod nexus;
mod register;
mod rst;
mod stats;
mod xml;

/// Starts the HTTP server with hyper so headers can be served with title case
pub async fn listen(pool: Pool<MySql>, broadcast_tx: broadcast::Sender<Message>) {
    let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL not set");
    let cors = CorsLayer::new().allow_origin(
        frontend_url
            .parse::<HeaderValue>()
            .expect("Could not convert FRONTEND_URL to header"),
    );

    let app = Router::new()
        .route("/_r2m/stats", get(stats::stats))
        .with_state(broadcast_tx)
        .route("/_r2m/register", post(register::register))
        .layer(cors)
        .route("/rdr/pprdr.asp", get(nexus::nexus))
        .route("/login.srf", get(login_server::login_server))
        .route(
            "/RST.srf",
            post(rst::rst).layer(middleware::from_fn(xml::content_type_xml::content_type_xml)),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Could not bind HTTP server");

    info!("HTTP server listening on port 3000");

    loop {
        let (socket, _remote_addr) = match listener.accept().await {
            Ok(l) => l,
            Err(error) => {
                error!("Could not get socket from accepted HTTP connection: {error}");
                continue;
            }
        };

        let tower_service = app.clone();
        tokio::spawn(async move {
            let socket = TokioIo::new(socket);
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let mut builder = server::conn::auto::Builder::new(TokioExecutor::new());
            builder.http1().title_case_headers(true);

            if let Err(err) = builder
                .serve_connection_with_upgrades(socket, hyper_service)
                .await
            {
                error!("Failed to serve connection: {err:#}");
            }
        });
    }
}
