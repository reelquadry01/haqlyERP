// Author: Quadri Atharu
// Standalone dev runner for haqly-erp-server (used outside of Tauri)

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,haqly_erp_server=debug")),
        )
        .init();

    tracing::info!("Starting HAQLY ERP server in standalone mode...");
    haqly_erp_server::start_server().await
}
