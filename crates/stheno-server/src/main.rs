//! `stheno-server`: Production entrypoint and composition root.

use clap::Parser;
use std::time::Duration;
use stheno_registry::AppContext;
use stheno_server::db::DbConfig;
use stheno_server::{load_entitlements, AppServer};

/// How long boot waits for the database before giving up. Long enough to ride
/// out a container starting alongside us, short enough to fail a broken
/// deployment rather than hang in it.
const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

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

    /// Runs pending database migrations on startup
    #[arg(long)]
    migrate: bool,

    /// Populates the database with a demo catalogue, regions, locales and an owner
    /// account, then exits. Explicit because seeding used to run on every boot.
    #[arg(long)]
    seed_demo: bool,

    /// Reports whether the database schema matches the embedded migrations, then exits.
    /// Exits non-zero on drift.
    #[arg(long)]
    check_migrations: bool,

    /// Registers a tenant, its subscription and its domain, then exits.
    ///
    /// Format: `<id>:<name>:<tier>:<domain>`, e.g.
    /// `ten_acme:Acme:pro:acme.localhost`. Creating a merchant is a platform
    /// operation, which is why there is no `POST /admin/stores` (ADR-012).
    #[arg(long, value_name = "ID:NAME:TIER:DOMAIN")]
    create_tenant: Option<String>,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1:9000")]
    bind: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Every path needs a database, `--check-config` included: it exists to
    // validate connectivity, and migrated services have no in-memory mode to
    // fall back to.
    let db_config = DbConfig::from_env()?;
    let pool = db_config.build_pool()?;
    stheno_server::db::check_connectivity(&pool, DB_CONNECT_TIMEOUT).await?;

    if args.check_config {
        println!("Checking configuration...");
        let ctx = AppContext::create_test_context();
        // A bad tier config must abort here rather than booting a deployment whose
        // entitlements nobody can reason about (ENTITLEMENTS.md 5b).
        let entitlements = load_entitlements()?;
        let server = AppServer::new(ctx, entitlements, pool);
        println!(
            "Configuration check PASSED: composition root bootstrapped, database reachable, tier = {}.",
            server.entitlements.tier().as_str()
        );
        return Ok(());
    }

    if args.migrate {
        let applied = stheno_db::migrations::run(&db_config.database_url).await?;
        println!("Applied {} migration(s).", applied.len());
        for name in &applied {
            println!("  - {name}");
        }
    }

    if args.check_migrations {
        let pending = stheno_db::migrations::pending(&db_config.database_url).await?;
        if pending.is_empty() {
            println!("Schema is up to date; no pending migrations.");
            return Ok(());
        }
        eprintln!(
            "Schema drift: {} migration(s) have not been applied:",
            pending.len()
        );
        for name in &pending {
            eprintln!("  - {name}");
        }
        std::process::exit(1);
    }

    if let Some(spec) = args.create_tenant.as_deref() {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() < 3 {
            eprintln!("--create-tenant expects ID:NAME:TIER[:DOMAIN]");
            std::process::exit(2);
        }
        let platform = stheno_platform::PlatformService::new(pool.clone());
        let created = platform
            .create_tenant(stheno_platform::NewTenant {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                tier: parts[2].to_string(),
                domain: parts.get(3).map(|d| d.to_string()),
            })
            .await?;
        println!(
            "Created tenant {} ({}) on tier {}.",
            created.id, created.name, parts[2]
        );
        return Ok(());
    }

    if args.seed_demo {
        // Seeding writes tenant-owned rows, so it must run inside a tenant scope
        // (ADR-012) — otherwise every insert takes its tenant_id DEFAULT from an unset
        // setting and violates NOT NULL.
        let tenant_id = std::env::var("STHENO_TENANT").unwrap_or_else(|_| "ten_default".into());
        stheno_db::tenant::scope(tenant_id.clone(), stheno_server::seed_demo(pool.clone())).await;
        println!("Demo data seeded for tenant {tenant_id}.");
        return Ok(());
    }

    let entitlements = load_entitlements()?;
    println!(
        "Starting Stheno Commerce Engine on {} (tier: {}, pool: {})",
        args.bind,
        entitlements.tier().as_str(),
        db_config.pool_size
    );
    let ctx = AppContext::create_test_context();
    let server = AppServer::new(ctx, entitlements, pool);

    // The periodic work. Started before the listener so a scheduling typo aborts the boot
    // instead of leaving a process that serves traffic and quietly collects no money.
    let jobs = stheno_server::scheduled_jobs(server.pool.clone(), server.payment_service.clone());
    match stheno_server::scheduler::spawn(server.pool.clone(), jobs) {
        Ok(n) => println!("Scheduler started: {n} job(s)."),
        Err(e) => {
            eprintln!("Scheduler refused to start: {e}");
            std::process::exit(1);
        }
    }

    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    axum::serve(listener, server.router).await?;

    Ok(())
}
