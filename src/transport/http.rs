//! Streamable-HTTP transport bring-up.
//!
//! Builds an `rmcp` `StreamableHttpService` with `LocalSessionManager`,
//! mounts it on `hyper-util` over a `TcpListener`, and serves until
//! ctrl-c. The service factory clones the handler per session — each
//! session gets a fresh handler instance.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as ConnBuilder;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::tower::{
    StreamableHttpServerConfig, StreamableHttpService,
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_service::Service;

use crate::server::McpServerHandler;

pub async fn serve(handler: McpServerHandler, bind: SocketAddr) -> Result<()> {
    let session_manager = Arc::new(LocalSessionManager::default());
    let ct = CancellationToken::new();

    // `StreamableHttpServerConfig` is `#[non_exhaustive]`; can't use
    // struct-update syntax from outside `rmcp`.
    let mut config = StreamableHttpServerConfig::default();
    config.cancellation_token = ct.clone();

    let factory_handler = handler.clone();
    let service = StreamableHttpService::new(
        move || Ok::<_, std::io::Error>(factory_handler.clone()),
        session_manager,
        config,
    );

    let listener: TcpListener = TcpListener::bind(bind)
        .await
        .with_context(|| format!("http transport bind {bind}"))?;
    tracing::info!(%bind, "streamable-http transport listening");

    let shutdown_ct = ct.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("ctrl-c received, shutting down http transport");
            shutdown_ct.cancel();
        }
    });

    let conn_builder = ConnBuilder::new(TokioExecutor::new());

    loop {
        tokio::select! {
            () = ct.cancelled() => {
                tracing::info!("http transport cancellation requested, exiting accept loop");
                break;
            }
            accept = listener.accept() => {
                let (stream, peer) = match accept {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::warn!(?err, "accept error, continuing");
                        continue;
                    }
                };
                tracing::debug!(%peer, "accepted connection");

                let service = service.clone();
                let conn_builder = conn_builder.clone();
                tokio::spawn(async move {
                    let io = TokioIo::new(stream);
                    let svc = hyper::service::service_fn(move |req| {
                        let mut svc = service.clone();
                        async move { svc.call(req).await }
                    });
                    if let Err(err) = conn_builder.serve_connection(io, svc).await {
                        tracing::debug!(?err, "connection ended");
                    }
                });
            }
        }
    }

    Ok(())
}
