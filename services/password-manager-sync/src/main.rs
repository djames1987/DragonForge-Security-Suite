use std::{env, net::SocketAddr, sync::Arc};

use dragonforge_sync_server::{AppState, InMemoryStore, SyncStore, build_router};
use zeroize::Zeroize;

#[cfg(feature = "postgres")]
use dragonforge_sync_server::PostgresStore;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("DragonForge sync server failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let bind: SocketAddr = env::var("DRAGONFORGE_SYNC_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8787".to_owned())
        .parse()?;
    let production = env::var("DRAGONFORGE_SYNC_ENVIRONMENT")
        .is_ok_and(|value| value.eq_ignore_ascii_case("production"));
    let database_url = env::var("DRAGONFORGE_SYNC_DATABASE_URL").ok();
    let tls_proxy = env::var("DRAGONFORGE_SYNC_TLS_PROXY")
        .is_ok_and(|value| value.eq_ignore_ascii_case("true"));
    let public_base_url = env::var("DRAGONFORGE_SYNC_PUBLIC_BASE_URL").ok();
    let rate_limit_requests = parse_bounded_u32_env(
        "DRAGONFORGE_SYNC_RATE_LIMIT_REQUESTS",
        240,
        10,
        10_000,
    )?;
    let rate_limit_window_seconds = parse_bounded_u64_env(
        "DRAGONFORGE_SYNC_RATE_LIMIT_WINDOW_SECONDS",
        60,
        1,
        3_600,
    )?;

    let mut admin_token = env::var("DRAGONFORGE_SYNC_ADMIN_TOKEN").ok();
    validate_runtime_config(
        bind,
        production,
        cfg!(feature = "postgres"),
        database_url.as_deref(),
        admin_token.as_deref(),
        tls_proxy,
        public_base_url.as_deref(),
    )?;

    #[cfg(feature = "postgres")]
    let store: Arc<dyn SyncStore> = if let Some(database_url) = database_url.as_deref() {
        Arc::new(PostgresStore::connect(database_url).await?)
    } else {
        eprintln!(
            "WARNING: DRAGONFORGE_SYNC_DATABASE_URL is not set; using volatile in-memory storage"
        );
        Arc::new(InMemoryStore::default())
    };

    #[cfg(not(feature = "postgres"))]
    let store: Arc<dyn SyncStore> = {
        if env::var_os("DRAGONFORGE_SYNC_DATABASE_URL").is_some() {
            return Err(
                "server was built without the postgres feature but DRAGONFORGE_SYNC_DATABASE_URL is set"
                    .into(),
            );
        }
        eprintln!(
            "WARNING: sync server was built without PostgreSQL support; using volatile in-memory storage"
        );
        Arc::new(InMemoryStore::default())
    };

    if admin_token.is_none() {
        eprintln!(
            "WARNING: DRAGONFORGE_SYNC_ADMIN_TOKEN is not set; account provisioning endpoint is disabled"
        );
    }

    let state = AppState::with_rate_limit(
        store,
        admin_token.as_deref(),
        rate_limit_requests,
        std::time::Duration::from_secs(rate_limit_window_seconds),
    );
    if let Some(token) = admin_token.as_mut() {
        token.zeroize();
    }
    let router = build_router(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    eprintln!("DragonForge sync server listening on {bind}");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn validate_runtime_config(
    bind: SocketAddr,
    production: bool,
    postgres_enabled: bool,
    database_url: Option<&str>,
    admin_token: Option<&str>,
    tls_proxy: bool,
    public_base_url: Option<&str>,
) -> Result<(), std::io::Error> {
    let invalid = |message: &str| std::io::Error::new(std::io::ErrorKind::InvalidInput, message);

    if admin_token.is_some_and(|token| token.len() < 32) {
        return Err(invalid(
            "DRAGONFORGE_SYNC_ADMIN_TOKEN must be at least 32 characters",
        ));
    }
    if !production {
        return Ok(());
    }
    if !postgres_enabled || database_url.is_none() {
        return Err(invalid(
            "production sync requires PostgreSQL support and DRAGONFORGE_SYNC_DATABASE_URL",
        ));
    }
    let admin_token = admin_token.ok_or_else(|| {
        invalid("production sync requires DRAGONFORGE_SYNC_ADMIN_TOKEN")
    })?;
    if admin_token.len() != 64 || !admin_token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid(
            "production DRAGONFORGE_SYNC_ADMIN_TOKEN must be a 64-character hexadecimal secret",
        ));
    }
    if !tls_proxy {
        return Err(invalid(
            "production sync requires DRAGONFORGE_SYNC_TLS_PROXY=true",
        ));
    }
    let public_base_url = public_base_url.ok_or_else(|| {
        invalid("production sync requires DRAGONFORGE_SYNC_PUBLIC_BASE_URL")
    })?;
    if !public_base_url.starts_with("https://")
        || public_base_url.len() > 2048
        || public_base_url.chars().any(char::is_whitespace)
    {
        return Err(invalid(
            "production DRAGONFORGE_SYNC_PUBLIC_BASE_URL must be a bounded HTTPS URL",
        ));
    }
    if bind.port() == 0 {
        return Err(invalid("production sync bind port must be non-zero"));
    }
    Ok(())
}

fn parse_bounded_u32_env(
    name: &str,
    default: u32,
    minimum: u32,
    maximum: u32,
) -> Result<u32, std::io::Error> {
    let Some(raw) = env::var(name).ok() else {
        return Ok(default);
    };
    let value = raw.parse::<u32>().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{name} must be an integer"))
    })?;
    if !(minimum..=maximum).contains(&value) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{name} must be between {minimum} and {maximum}"),
        ));
    }
    Ok(value)
}

fn parse_bounded_u64_env(
    name: &str,
    default: u64,
    minimum: u64,
    maximum: u64,
) -> Result<u64, std::io::Error> {
    let Some(raw) = env::var(name).ok() else {
        return Ok(default);
    };
    let value = raw.parse::<u64>().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{name} must be an integer"))
    })?;
    if !(minimum..=maximum).contains(&value) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{name} must be between {minimum} and {maximum}"),
        ));
    }
    Ok(value)
}

async fn shutdown_signal() {
    if tokio::signal::ctrl_c().await.is_err() {
        eprintln!("DragonForge sync server shutdown signal listener failed");
    }
}


#[cfg(test)]
mod tests {
    use super::validate_runtime_config;

    #[test]
    fn production_requires_persistent_database_tls_and_strong_admin_secret() {
        let bind = "0.0.0.0:8787".parse().unwrap();
        assert!(validate_runtime_config(bind, true, true, Some("postgres://db"), Some(&"a".repeat(64)), true, Some("https://sync.example.com")).is_ok());
        assert!(validate_runtime_config(bind, true, false, Some("postgres://db"), Some(&"a".repeat(64)), true, Some("https://sync.example.com")).is_err());
        assert!(validate_runtime_config(bind, true, true, None, Some(&"a".repeat(64)), true, Some("https://sync.example.com")).is_err());
        assert!(validate_runtime_config(bind, true, true, Some("postgres://db"), Some("weak"), true, Some("https://sync.example.com")).is_err());
        assert!(validate_runtime_config(bind, true, true, Some("postgres://db"), Some(&"a".repeat(64)), false, Some("https://sync.example.com")).is_err());
        assert!(validate_runtime_config(bind, true, true, Some("postgres://db"), Some(&"a".repeat(64)), true, Some("http://sync.example.com")).is_err());
    }
}
