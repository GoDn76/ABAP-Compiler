use abapc_studio::api::handlers::AppState;
use abapc_studio::config::AppConfig;
use abapc_studio::db::{PostgresClient, RedisSessionStore};
use abapc_studio::platform::LocalPlatform;
use abapc_studio::server::create_router;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::load();

    // Ensure directories exist
    let _ = tokio::fs::create_dir_all(&config.executed_dir).await;

    // Load local platform state if available
    let mut platform = LocalPlatform::new();
    if let Ok(state_str) = tokio::fs::read_to_string(&config.platform_state_path).await {
        if let Ok(state_json) = serde_json::from_str::<serde_json::Value>(&state_str) {
            platform.import_state(&state_json);
        }
    }

    let shared_platform = Arc::new(RwLock::new(platform));
    let redis_store = RedisSessionStore::new(config.redis_url.clone());
    let postgres_client = PostgresClient::new(config.database_url.clone());

    let state = AppState {
        config: config.clone(),
        platform: shared_platform,
        redis: redis_store,
        postgres: postgres_client,
    };

    let router = create_router(state);

    // Port auto-increment loop (7345 up to 7365)
    let start_port = config.port;
    let mut bound_listener = None;
    let mut active_port = start_port;

    for port in start_port..start_port + 20 {
        let addr_str = format!("{}:{}", config.host, port);
        match TcpListener::bind(&addr_str).await {
            Ok(listener) => {
                bound_listener = Some(listener);
                active_port = port;
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                continue;
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    let listener = bound_listener
        .ok_or_else(|| "Could not find a free local port in range 7345-7365.")?;

    let base_url = format!("http://{}:{}", config.host, active_port);
    let gui_url = format!("{}/sapgui", base_url);

    println!("\nABAPC Studio {} · SAP GUI Practice Simulator", config.version);
    println!("Practice simulator: {}", gui_url);
    println!("Compiler IDE:       {}", base_url);
    println!("Press Ctrl+C to stop it.\n");

    if !config.no_browser {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            open_browser(&gui_url);
        });
    }

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd").args(["/c", "start", "", url]).spawn();

    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let _ = Command::new("xdg-open").arg(url).spawn();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
