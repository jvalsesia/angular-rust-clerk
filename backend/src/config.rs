use std::env;

/// Configuration parameters for the backend service.
#[derive(Debug, Clone)]
pub struct Config {
    /// Port number the HTTP listener binds to.
    pub port: u16,
    /// List of allowed host origins for incoming CORS requests.
    pub allowed_origins: Vec<String>,
    /// Clerk JWKS URL for cryptographic key retrieval.
    pub clerk_jwks_url: String,
    /// Clerk Issuer URL for JWT claim validation.
    pub clerk_issuer: String,
}

impl Config {
    /// Parses configurations from the current runtime environment.
    /// Attempts to read from a local `.env` configuration file if present.
    pub fn from_env() -> Self {
        // We ignore the error if no .env file exists since env variables may be supplied directly
        let _ = dotenvy::dotenv();

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);

        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:4200".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let clerk_jwks_url = env::var("CLERK_JWKS_URL")
            .unwrap_or_else(|_| "https://api.clerk.com/v1/jwks".to_string());

        let clerk_issuer = env::var("CLERK_ISSUER")
            .unwrap_or_else(|_| "https://gentle-ophaph-98.clerk.accounts.dev".to_string());

        Self {
            port,
            allowed_origins,
            clerk_jwks_url,
            clerk_issuer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        // Test default case
        unsafe {
            env::remove_var("PORT");
            env::remove_var("ALLOWED_ORIGINS");
            env::remove_var("CLERK_JWKS_URL");
            env::remove_var("CLERK_ISSUER");
        }

        let config = Config::from_env();
        assert_eq!(config.port, 3000);
        assert_eq!(config.allowed_origins, vec!["http://localhost:4200"]);
        assert_eq!(config.clerk_jwks_url, "https://api.clerk.com/v1/jwks");
        assert_eq!(config.clerk_issuer, "https://gentle-ophaph-98.clerk.accounts.dev");

        // Test override case
        unsafe {
            env::set_var("PORT", "8080");
            env::set_var(
                "ALLOWED_ORIGINS",
                "http://localhost:3000, https://clerk.com",
            );
            env::set_var("CLERK_JWKS_URL", "https://clerk.example.com/jwks");
            env::set_var("CLERK_ISSUER", "https://clerk.example.com");
        }

        let config = Config::from_env();
        assert_eq!(config.port, 8080);
        assert_eq!(
            config.allowed_origins,
            vec!["http://localhost:3000", "https://clerk.com"]
        );
        assert_eq!(config.clerk_jwks_url, "https://clerk.example.com/jwks");
        assert_eq!(config.clerk_issuer, "https://clerk.example.com");

        // Clean up environment variables
        unsafe {
            env::remove_var("PORT");
            env::remove_var("ALLOWED_ORIGINS");
            env::remove_var("CLERK_JWKS_URL");
            env::remove_var("CLERK_ISSUER");
        }
    }
}
