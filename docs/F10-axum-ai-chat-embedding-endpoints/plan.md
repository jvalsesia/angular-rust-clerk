# Implementation Plan: Axum AI Chat & Embedding Endpoints (F10)

**Prerequisites:**
*   PostgreSQL database with the `pgvector` extension running (F08).
*   LiteLLM Proxy service online (F09).
*   Environment Variables: `LITELLM_URL` and `LITELLM_API_KEY` defined.

---

### Stage 1: Configurations & Setup

**1. Configuration Parameters** - Update the configuration module in the backend application to read and parse the LiteLLM Proxy endpoint URL and authorization API keys from environment variables.

**2. Environment Secrets Setup** - Add fallback configuration keys for the LiteLLM Proxy URL and mock master keys to the local environment variables definition files.

---

### Stage 2: Database Storage Utilities

**3. Database Queries Module** - Implement database utility methods in the backend application to save conversation metadata, create new chat sessions, insert individual messages, and insert vector embeddings.

**4. Semantic Vector Queries** - Write database search methods to execute vector similarity queries against the embeddings table using cosine-distance operations, filtering results based on the Clerk user identity.

---

### Stage 3: Chat Handler & SSE Integration

**5. Embedding Client Service** - Implement a client method in the backend application to call the LiteLLM Proxy embeddings endpoint and extract floating-point vector arrays.

**6. Axum Chat Route and SSE Handler** - Create the chat route handler in the backend web definitions. Parse JSON inputs, retrieve Clerk JWT claims, fetch embeddings, run similarity queries to construct prompt context, call the LiteLLM Proxy chat completion endpoint, and stream event-driven tokens back to the client using Server-Sent Events.

**7. Connection Drop Cleanup** - Integrate asynchronous tracking into the chat handler to monitor client disconnect events, immediately aborting upstream gateway requests to prevent token leakage.
