use crate::registry::PeerRegistry;
use crate::ws_handler;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tracing::info;

pub async fn run_server(addr: &str) -> std::io::Result<()> {
    let registry = web::Data::new(Arc::new(PeerRegistry::new()));

    info!("Signaling server starting on {addr}");

    HttpServer::new(move || {
        App::new()
            .app_data(registry.clone())
            .route("/ws", web::get().to(ws_handler::ws_handler))
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind(addr)?
    .run()
    .await
}
