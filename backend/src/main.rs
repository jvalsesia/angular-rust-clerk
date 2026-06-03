use backend::{config::Config, create_app};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing logger
    tracing_subscriber::fmt::init();

    // Load server configurations
    let config = Config::from_env();
    info!("Loaded configuration: {:?}", config);

    // Initialize database connection pool
    info!("Initializing database connection pool...");
    let pool = backend::db::init_db_pool(&config.database_url)
        .await
        .expect("Failed to initialize database pool");

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    info!("Database migrations successfully completed!");

    let app = create_app(&config, Some(pool));


    // Bind listener socket to the configured port
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting Axum server on http://{}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
