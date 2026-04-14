#[cfg(feature = "cloud-server")]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(e) = better_ctx::cloud_server::run().await {
        eprintln!("Cloud server error: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "cloud-server"))]
fn main() {
    eprintln!("Build with --features cloud-server to enable the cloud API server.");
    std::process::exit(1);
}
