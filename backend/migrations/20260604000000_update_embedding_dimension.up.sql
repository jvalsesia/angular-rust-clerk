-- Drop the old HNSW index for embedding_768 if it exists
DROP INDEX IF EXISTS ix_chat_embeddings_hnsw_768;

-- Remove the old embedding_768 column from chat_embeddings if it exists
ALTER TABLE chat_embeddings DROP COLUMN IF EXISTS embedding_768;

-- Add the new embedding_3072 column (standard vector type of 3072 dimensions)
ALTER TABLE chat_embeddings ADD COLUMN embedding_3072 vector(3072);
