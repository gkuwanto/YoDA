use axum::{http::{Request, StatusCode}, middleware::Next, response::Response};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;
use axum::body::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub struct AuthUser(pub Uuid);

pub async fn jwt_auth(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get("authorization").and_then(|h| h.to_str().ok());
    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            );
            if let Ok(data) = token_data {
                if let Ok(user_id) = Uuid::parse_str(&data.claims.sub) {
                    req.extensions_mut().insert(AuthUser(user_id));
                    return Ok(next.run(req).await);
                }
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::env;
    use chrono::Utc;

    fn create_test_token(user_id: Uuid) -> String {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let exp = (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
    }

    #[test]
    fn test_create_valid_token() {
        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id);
        
        // Verify the token can be decoded
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );
        
        assert!(token_data.is_ok());
        let claims = token_data.unwrap().claims;
        assert_eq!(claims.sub, user_id.to_string());
    }

    #[test]
    fn test_create_expired_token() {
        let user_id = Uuid::new_v4();
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let exp = (Utc::now() - chrono::Duration::hours(1)).timestamp() as usize; // Expired
        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap();
        
        // Verify the token is expired
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );
        
        assert!(token_data.is_err());
    }

    #[test]
    fn test_invalid_token_format() {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let token_data = decode::<Claims>(
            "invalid.token.here",
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );
        
        assert!(token_data.is_err());
    }

    #[test]
    fn test_claims_serialization() {
        let user_id = Uuid::new_v4();
        let exp = (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };
        
        // Test serialization
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: Claims = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.sub, user_id.to_string());
        assert_eq!(deserialized.exp, exp);
    }
} 