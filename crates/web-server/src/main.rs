use std::net::SocketAddr;

use axum::{
    Router, 
    routing::{get, post},
};
use tower_http::services::ServeDir;
use tower_http::compression::CompressionLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use grpc_api::{CvadService, generated::cvad::v1::cvad_server::CvadServer};

mod handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_routes = Router::new()
        .route("/get_remain_days", get(handlers::remain_days::get_remain_days));

    // Axum REST API
    let rest_app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/api/v1", api_routes)
        .fallback_service(ServeDir::new("/app/frontend").append_index_html_on_directories(true))
        .layer(CompressionLayer::new());

    // gRPC service
    let grpc_service = CvadServer::new(CvadService::default());

    // 두 서버의 병렬 실행
    let rest_addr = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    let grpc_addr: SocketAddr = ("0.0.0.0:50051").parse()?;

    tokio::try_join!(
        async {
            println!("Starting REST server at {}", rest_addr.local_addr()?);
            axum::serve(rest_addr, rest_app.into_make_service()).await?;
            Ok::<_, anyhow::Error>(())
        },
        async {
            println!("Starting gRPC server at {}", grpc_addr);
            tonic::transport::Server::builder()
                .add_service(grpc_service)
                .serve(grpc_addr)
                .await?;
            Ok::<_, anyhow::Error>(())
        }
    )?;

    Ok(())
}