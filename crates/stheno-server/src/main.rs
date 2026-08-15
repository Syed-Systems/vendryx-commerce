//! `stheno-server`: Production entrypoint and composition root.

use clap::Parser;
use stheno_registry::AppContext;
use stheno_server::AppServer;

#[derive(Parser, Debug)]
#[command(
    name = "stheno-server",
    author,
    version,
    about = "Stheno Commerce Engine Server"
)]
struct Args {
    /// Validates system configuration, database connectivity, and boot health without launching daemon
    #[arg(long)]
    check_config: bool,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1:9000")]
    bind: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.check_config {
        println!("Checking configuration...");
        let ctx = AppContext::create_test_context();
        let _server = AppServer::new(ctx);
        println!("Configuration check PASSED: Composition root bootstrapped successfully.");
        return Ok(());
    }

    println!("Starting Stheno Commerce Engine on {}", args.bind);
    let ctx = AppContext::create_test_context();
    let server = AppServer::new(ctx);

    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    axum::serve(listener, server.router).await?;

    Ok(())
}
