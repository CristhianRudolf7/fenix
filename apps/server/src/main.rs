use anyhow::{Result, bail};
use fenix_agent::Runtime;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("owner") {
        return fenix_server::auth::configure_from_stdin();
    }
    if std::env::args().nth(1).as_deref() == Some("eval") {
        let mut args = std::env::args().skip(2);
        let config = args.next().ok_or_else(|| {
            anyhow::anyhow!("Usage: fenix eval CONFIG.json OUTPUT_DIR [--keep-services]")
        })?;
        let output = args
            .next()
            .ok_or_else(|| anyhow::anyhow!("Evaluation output directory required"))?;
        let keep_services = match args.next().as_deref() {
            Some("--keep-services") => true,
            None => false,
            _ => bail!("Unknown evaluation option"),
        };
        if args.next().is_some() {
            bail!("Unexpected evaluation arguments");
        }
        let config = serde_json::from_slice(&std::fs::read(config)?)?;
        return fenix_agent::evaluation::run(config, std::path::Path::new(&output), keep_services)
            .await;
    }
    let mut data_dir = None;
    let mut skills_dir = None;
    let mut port = 4173;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        anyhow::anyhow!("--data-dir requires a path")
                    })?))
            }
            "--skills-dir" => {
                skills_dir =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        anyhow::anyhow!("--skills-dir requires a path")
                    })?))
            }
            "--port" => {
                port = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--port requires a number"))?
                    .parse()?
            }
            "--help" | "-h" => {
                println!(
                    "Fenix local assistant\n\nfenix owner EMAIL --password-stdin [--data-dir PATH]\nfenix eval CONFIG.json NEW_OUTPUT_DIR [--keep-services]\nfenix [--port 4173] [--data-dir PATH] [--skills-dir PATH]\n\nData defaults to the OS application-data directory; an existing local .fenix is preserved. Skills default to ~/.fenix/skills, shared across data directories. --skills-dir overrides it.\n\nConfigure the owner with fenix owner EMAIL --password-stdin, then open http://127.0.0.1:4173 to sign in and configure the model and optional Telegram."
                );
                return Ok(());
            }
            _ => bail!("Unknown option. Use --help"),
        }
    }
    let data_dir = match data_dir {
        Some(path) => path,
        None => fenix_execution::fs::layout::default_data_dir()?,
    };
    let mut shutdown_source = fenix_process::shutdown::Shutdown::listen()?;
    let runtime = Runtime::open(&data_dir, skills_dir.as_deref()).await?;
    let ip = Ipv4Addr::LOCALHOST;
    let address = SocketAddr::new(IpAddr::V4(ip), port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    let port = listener.local_addr()?.port();
    let mut workers = runtime.start();
    workers.push(runtime.start_model_health());
    workers.push(runtime.start_web_search());
    workers.push(runtime.start_skill_localizations());
    println!(
        "Fénix is ready at http://127.0.0.1:{port}\nRuntime data: {}\nShared skills: {}",
        runtime.data_dir.display(),
        runtime.skills_dir.display()
    );
    let public_origin = std::env::var("FENIX_PUBLIC_ORIGIN").ok();
    let app = fenix_server::personal_router(runtime.clone(), port, public_origin.as_deref())?;
    let shutdown = runtime.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_source.wait().await;
            let _ = shutdown.shutdown().await;
        })
        .await?;
    runtime.shutdown().await?;
    for worker in workers {
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), worker).await;
    }
    Ok(())
}
