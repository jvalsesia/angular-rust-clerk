use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Initializes the PostgreSQL database connection pool.
pub async fn init_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let mut attempt = 1;
    let max_attempts = 5;
    let delay = Duration::from_secs(2);

    loop {
        info!("Connecting to database (Attempt {}/{})...", attempt, max_attempts);
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .connect(database_url)
            .await
        {
            Ok(pool) => {
                info!("Successfully connected to database!");
                return Ok(pool);
            }
            Err(e) => {
                if attempt >= max_attempts {
                    error!("FATAL [db] Database connection could not be established after {} attempts. Error: {}", max_attempts, e);
                    return Err(e);
                }
                warn!("Database connection failed: {}. Retrying in {:?}...", e, delay);
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

/// Creates a new chat session for a user.
pub async fn create_session(pool: &PgPool, user_id: &str, title: &str) -> Result<Uuid, sqlx::Error> {
    let sql = "INSERT INTO chat_sessions (user_id, title) VALUES ($1, $2) RETURNING id";
    use sqlx::Row;
    let row = sqlx::query(sql)
        .bind(user_id)
        .bind(title)
        .fetch_one(pool)
        .await?;
    Ok(row.get("id"))
}

/// Saves a message in a chat session.
pub async fn save_message(
    pool: &PgPool,
    session_id: Uuid,
    role: &str,
    content: &str,
    msg_index: i32,
) -> Result<Uuid, sqlx::Error> {
    let sql = "INSERT INTO chat_messages (session_id, role, content, message_index) VALUES ($1, $2, $3, $4) RETURNING id";
    use sqlx::Row;
    let row = sqlx::query(sql)
        .bind(session_id)
        .bind(role)
        .bind(content)
        .bind(msg_index)
        .fetch_one(pool)
        .await?;
    Ok(row.get("id"))
}

/// Saves the text embedding associated with a message.
pub async fn save_embedding(
    pool: &PgPool,
    message_id: Uuid,
    embedding: &[f32],
    is_1536: bool,
) -> Result<(), sqlx::Error> {
    let vector_str = format!("[{}]", embedding.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    let sql = if is_1536 {
        "INSERT INTO chat_embeddings (message_id, embedding_1536) VALUES ($1, $2::vector)"
    } else {
        "INSERT INTO chat_embeddings (message_id, embedding_3072) VALUES ($1, $2::vector)"
    };
    sqlx::query(sql)
        .bind(message_id)
        .bind(vector_str)
        .execute(pool)
        .await?;
    Ok(())
}

/// Retrieves semantically matched relevant context from user's chat history.
pub async fn get_semantic_context(
    pool: &PgPool,
    user_id: &str,
    embedding: &[f32],
    is_1536: bool,
    limit: i64,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let vector_str = format!("[{}]", embedding.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    let sql = if is_1536 {
        "SELECT m.role, m.content \
         FROM chat_embeddings e \
         JOIN chat_messages m ON e.message_id = m.id \
         JOIN chat_sessions s ON m.session_id = s.id \
         WHERE s.user_id = $1 AND (e.embedding_1536 <=> $2::vector) < 0.5 \
         ORDER BY e.embedding_1536 <=> $2::vector ASC \
         LIMIT $3"
    } else {
        "SELECT m.role, m.content \
         FROM chat_embeddings e \
         JOIN chat_messages m ON e.message_id = m.id \
         JOIN chat_sessions s ON m.session_id = s.id \
         WHERE s.user_id = $1 AND (e.embedding_3072 <=> $2::vector) < 0.5 \
         ORDER BY e.embedding_3072 <=> $2::vector ASC \
         LIMIT $3"
    };

    use sqlx::Row;
    let rows = sqlx::query(sql)
        .bind(user_id)
        .bind(vector_str)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    let mut messages = Vec::new();
    for row in rows {
        let role: String = row.get("role");
        let content: String = row.get("content");
        messages.push((role, content));
    }
    // Reverse context messages so they are loaded in chronological order in the prompt context
    messages.reverse();
    Ok(messages)
}

/// Retrieves chronological recent messages of a chat session as a fallback.
pub async fn get_recent_messages(
    pool: &PgPool,
    session_id: Uuid,
    limit: i64,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let sql = "SELECT role, content FROM chat_messages \
               WHERE session_id = $1 \
               ORDER BY message_index DESC \
               LIMIT $2";
    use sqlx::Row;
    let rows = sqlx::query(sql)
        .bind(session_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    let mut messages = Vec::new();
    for row in rows {
        let role: String = row.get("role");
        let content: String = row.get("content");
        messages.push((role, content));
    }
    // Reverse context messages so they are loaded in chronological order
    messages.reverse();
    Ok(messages)
}
