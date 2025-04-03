use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    iat: i64,
    exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<String>, // Subject (Developer ID, not needed for Music ID in this case)
}

/// Developer Token Generator
pub struct DeveloperTokenGenerator {
    team_id: String,
    key_id: String,
    private_key_path: String,
}

impl DeveloperTokenGenerator {
    /// Create a new generator
    pub fn new(team_id: String, key_id: String, private_key_path: String) -> Self {
        Self {
            team_id,
            key_id,
            private_key_path,
        }
    }

    /// Load private key from p8 file
    fn load_private_key(&self) -> Result<Vec<u8>> {
        let path = Path::new(&self.private_key_path);
        if !path.exists() {
            return Err(anyhow!(
                "Private key file not found: {}",
                self.private_key_path
            ));
        }

        let mut file = File::open(path)?;
        let mut key_data = Vec::new();
        file.read_to_end(&mut key_data)?;

        debug!(
            "p8 file loaded successfully: {} ({} bytes)",
            self.private_key_path,
            key_data.len()
        );
        Ok(key_data)
    }

    /// Generate developer token
    pub fn generate_token(&self, expiration_hours: i64) -> Result<String> {
        let key_data = self.load_private_key()?;

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        // Set the time
        let now = Utc::now();
        let expiration = now + Duration::hours(expiration_hours);

        let claims = Claims {
            iss: self.team_id.clone(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            sub: None, // Not required for Music ID
        };

        let key = EncodingKey::from_ec_pem(&key_data)
            .map_err(|e| anyhow!("Failed to load private key: {}", e))?;

        let token =
            encode(&header, &claims, &key).map_err(|e| anyhow!("Failed to encode token: {}", e))?;

        debug!(
            "Generated Apple Music developer token: valid for {} hours",
            expiration_hours
        );
        Ok(token)
    }
}
