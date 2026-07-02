mod config;
mod miner;

use axum::{extract::State, routing::get, Json, Router};
use miner::SharedStatus;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    let cfg = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("configuration error: {e}");
            std::process::exit(1);
        }
    };

    let status: SharedStatus = Arc::new(RwLock::new(miner::MinerStatus::default()));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let supervisor = miner::spawn_supervisor(cfg.clone(), status.clone(), shutdown_rx);

    let app = Router::new()
        .route("/health", get(health))
        .with_state(status);

    let addr = format!("0.0.0.0:{}", cfg.port);
    tracing::info!("dashboard listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind port");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_tx))
        .await
        .expect("server error");

    // Wait for the supervisor to kill the miner before exiting.
    let _ = supervisor.await;
}

async fn shutdown_signal(tx: watch::Sender<bool>) {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to install SIGTERM handler");
    tokio::select! {
        _ = ctrl_c => {}
        _ = term.recv() => {}
    }
    tracing::info!("shutdown signal received");
    let _ = tx.send(true);
}

async fn health(State(status): State<SharedStatus>) -> Json<serde_json::Value> {
    let s = status.read().await;
    Json(serde_json::json!({
        "status": "ok",
        "miner": {
            "running": s.running,
            "pid": s.pid,
            "restarts": s.restarts,
            "uptime_seconds": s.started_at
                .filter(|_| s.running)
                .map(|t| t.elapsed().as_secs()),
            "last_error": s.last_error,
        }
    }))
}
