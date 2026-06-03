use backend::{config::Config, create_app};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing logger
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    // Load server configurations
    let config = Config::from_env();
    info!("Loaded configuration: {:?}", config);

    // Initialize database connection pool
    let pool = match backend::db::init_db_pool(&config.database_url).await {
        Ok(p) => p,
        Err(_) => {
            std::process::exit(1);
        }
    };

    // Run migrations
    info!("Running database migrations...");
    match sqlx::migrate!("./migrations")
        .run(&pool)
        .await
    {
        Ok(_) => info!("Database migrations successfully completed!"),
        Err(e) => {
            tracing::error!("FATAL [db] Database migration failed: {}", e);
            std::process::exit(1);
        }
    }

    // Verify LiteLLM connection
    backend::check_litellm_connection(&config.litellm_url, &config.litellm_api_key).await;

    let app = create_app(&config, Some(pool));


    // Bind listener socket to the configured port
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting Axum server on http://{}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
