use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use sipesantren::{
    build_app_router,
    config::AppConfig,
    db::init_db_pool,
    notification::ConfigurableWaGateway,
    seed::seed_initial_data,
    sse::SseBroadcaster,
    AppState,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sipesantren=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::from_env();
    tokio::fs::create_dir_all(&config.photo_storage_dir).await?;

    let db = init_db_pool(&config.database_url).await?;
    if config.seed_on_startup {
        seed_initial_data(&db).await?;
    }

    let sse = Arc::new(SseBroadcaster::new());
    let notifier = Arc::new(ConfigurableWaGateway::new(db.clone(), &config));
    let photo_base_dir = Arc::new(PathBuf::from(&config.photo_storage_dir));

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let state = AppState {
        db,
        config: Arc::new(config),
        sse,
        notifier,
        photo_base_dir,
    };

    let app = build_app_router(state);
    tracing::info!("SiPesantren server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
