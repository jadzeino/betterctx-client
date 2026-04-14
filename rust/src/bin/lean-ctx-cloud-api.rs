use better_ctx::cloud_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cloud_server::run().await
}
