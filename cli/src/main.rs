#[tokio::main]
async fn main() -> anyhow::Result<()> {
    hdp_cli::cli::hdp_cli_run().await
}
