#[tokio::main]
async fn main() -> anyhow::Result<()> {
    hdp_cli::run().await
}
