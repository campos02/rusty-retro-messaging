use axum::{
    Router,
    routing::{get, post},
};
use diesel::{
    MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use hyper::{Request, body::Incoming};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use tower_service::Service;

mod login_server;
mod nexus;
mod register;

/// Starts the HTTP server with hyper so headers can be served with title case
pub async fn listen(pool: Pool<ConnectionManager<MysqlConnection>>) {
    let app = Router::new()
        .route(
            "/",
            get(|| async { "MSNP server, start by connecting on port 1863" }),
        )
        .route("/register", post(register::register))
        .route("/rdr/pprdr.asp", get(nexus::nexus))
        .route("/login.srf", get(login_server::login_server))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    loop {
        let (socket, _remote_addr) = listener.accept().await.unwrap();
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
                eprintln!("Failed to serve connection: {err:#}");
            }
        });
    }
}
