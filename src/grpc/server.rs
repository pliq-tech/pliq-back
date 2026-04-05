//! Tonic gRPC server configuration and startup.
//!
//! Stub implementation — actual service handlers will be added
//! when pliq-ai integration is ready.

use std::net::SocketAddr;

/// Start the gRPC server on the given address.
///
/// Currently a placeholder. The tonic Server requires at least one
/// registered service to start. This function will be activated when
/// gRPC services (health, fraud detection, matching) are implemented.
pub async fn start_grpc_server(
    _addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("gRPC server: no services registered yet, skipping");
    // tonic::transport::Server::builder()
    //     .add_service(service)
    //     .serve(addr)
    //     .await?;
    Ok(())
}
