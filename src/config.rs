use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub max_body_bytes: usize,
    pub max_steps: usize,
    pub no_browser: bool,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub app_dir: String,
    pub executed_dir: String,
    pub examples_dir: String,
    pub manifest_path: String,
    pub platform_state_path: String,
    pub version: String,
}

fn resolve_path(rel: &str) -> String {
    if Path::new(rel).exists() {
        rel.to_string()
    } else {
        let parent = format!("../{}", rel);
        if Path::new(&parent).exists() {
            parent
        } else {
            rel.to_string()
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let _ = dotenvy::dotenv();

        let host = env::var("ABAPC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("ABAPC_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(7345);
        let no_browser = env::var("ABAPC_NO_BROWSER")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let database_url = env::var("DATABASE_URL").ok().filter(|s| !s.is_empty());
        let redis_url = env::var("REDIS_URL").ok().filter(|s| !s.is_empty());

        Self {
            host,
            port,
            max_body_bytes: 2 * 1024 * 1024, // 2MB limit
            max_steps: 10_000_000,
            no_browser,
            database_url,
            redis_url,
            app_dir: resolve_path("app"),
            executed_dir: resolve_path("executed codes"),
            examples_dir: resolve_path("examples"),
            manifest_path: resolve_path("runtime/released-apis.json"),
            platform_state_path: resolve_path("runtime/local-platform-state.json"),
            version: "5.0.1".to_string(),
        }
    }
}
