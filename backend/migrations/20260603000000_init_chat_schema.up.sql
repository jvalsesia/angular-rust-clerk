-- Enable the vector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create chat sessions table
CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL DEFAULT 'New Chat',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ix_chat_sessions_user_id ON chat_sessions(user_id);

-- Create chat messages table
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    message_index INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ix_chat_messages_session_id_index ON chat_messages(session_id, message_index);

-- Create chat embeddings table
CREATE TABLE chat_embeddings (
    message_id UUID PRIMARY KEY REFERENCES chat_messages(id) ON DELETE CASCADE,
    embedding_1536 vector(1536),
    embedding_768 vector(768)
);

-- HNSW Cosine Similarity indexes (requires pgvector)
CREATE INDEX ix_chat_embeddings_hnsw_1536 ON chat_embeddings USING hnsw (embedding_1536 vector_cosine_ops);
CREATE INDEX ix_chat_embeddings_hnsw_768 ON chat_embeddings USING hnsw (embedding_768 vector_cosine_ops);
