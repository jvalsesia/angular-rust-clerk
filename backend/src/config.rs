use std::env;

/// Configuration parameters for the backend service.
#[derive(Debug, Clone)]
pub struct Config {
    /// Port number the HTTP listener binds to.
    pub port: u16,
    /// List of allowed host origins for incoming CORS requests.
    pub allowed_origins: Vec<String>,
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

        Self {
            port,
            allowed_origins,
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
        }

        let config = Config::from_env();
        assert_eq!(config.port, 3000);
        assert_eq!(config.allowed_origins, vec!["http://localhost:4200"]);

        // Test override case
        unsafe {
            env::set_var("PORT", "8080");
            env::set_var(
                "ALLOWED_ORIGINS",
                "http://localhost:3000, https://clerk.com",
            );
        }

        let config = Config::from_env();
        assert_eq!(config.port, 8080);
        assert_eq!(
            config.allowed_origins,
            vec!["http://localhost:3000", "https://clerk.com"]
        );

        // Clean up environment variables
        unsafe {
            env::remove_var("PORT");
            env::remove_var("ALLOWED_ORIGINS");
        }
    }
}
