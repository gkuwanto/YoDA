use axum::{Json, response::IntoResponse, http::StatusCode, Extension, extract::Path};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use crate::models::{User, Campaign, Session, Character, GameState, InitiativeEntry, EventLog};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::env;
use crate::middleware::AuthUser;
use chrono::DateTime;

// Auth handlers
#[derive(Deserialize, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

pub async fn register(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Check for existing user
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1 OR username = $2"
    )
    .bind(&payload.email)
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    if exists > 0 {
        return (StatusCode::CONFLICT, "Email or username already exists");
    }

    // Hash password
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(payload.password.as_bytes(), &argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng)) {
        Ok(hash) => hash.to_string(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"),
    };

    // Insert user
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let res = sqlx::query(
        "INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => (StatusCode::CREATED, "Registered"),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to register user"),
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn login(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Fetch user by email
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        _ => return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
    };

    // Verify password
    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
    };
    let argon2 = Argon2::default();
    let valid = argon2.verify_password(payload.password.as_bytes(), &parsed_hash).is_ok();
    if !valid {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
    }

    // Issue JWT
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let exp = (Utc::now() + chrono::Duration::days(7)).timestamp() as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        exp,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()));
    match token {
        Ok(token) => (
            StatusCode::OK,
            axum::Json(LoginResponse { token })
        ).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response(),
    }
}

// Campaign handlers
#[derive(Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub description: Option<String>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct CampaignResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub dm_id: Uuid,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_campaign(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<CreateCampaignRequest>,
) -> impl IntoResponse {
    let campaign_id = Uuid::new_v4();
    let now = Utc::now();
    let settings = payload.settings.unwrap_or_else(|| serde_json::json!({}));
    
    let res = sqlx::query_as::<_, Campaign>(
        "INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    )
    .bind(campaign_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(user.0)
    .bind(&settings)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(campaign) => (
            StatusCode::CREATED,
            axum::Json(CampaignResponse {
                id: campaign.id,
                name: campaign.name,
                description: campaign.description,
                dm_id: campaign.dm_id,
                settings: campaign.settings,
                created_at: campaign.created_at,
                updated_at: campaign.updated_at,
            })
        ).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create campaign").into_response(),
    }
}

pub async fn list_campaigns(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
) -> impl IntoResponse {
    let campaigns = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE dm_id = $1 OR id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $1) ORDER BY created_at DESC"
    )
    .bind(user.0)
    .fetch_all(&pool)
    .await;

    match campaigns {
        Ok(campaigns) => {
            let responses: Vec<CampaignResponse> = campaigns.into_iter().map(|c| CampaignResponse {
                id: c.id,
                name: c.name,
                description: c.description,
                dm_id: c.dm_id,
                settings: c.settings,
                created_at: c.created_at,
                updated_at: c.updated_at,
            }).collect();
            axum::Json(responses).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch campaigns").into_response(),
    }
}

pub async fn get_campaign(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(campaign_id): Path<Uuid>,
) -> impl IntoResponse {
    let campaign = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE id = $1 AND (dm_id = $2 OR id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2))"
    )
    .bind(campaign_id)
    .bind(user.0)
    .fetch_optional(&pool)
    .await;

    match campaign {
        Ok(Some(campaign)) => {
            let response = CampaignResponse {
                id: campaign.id,
                name: campaign.name,
                description: campaign.description,
                dm_id: campaign.dm_id,
                settings: campaign.settings,
                created_at: campaign.created_at,
                updated_at: campaign.updated_at,
            };
            axum::Json(response).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, "Campaign not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch campaign").into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub settings: Option<serde_json::Value>,
}

pub async fn update_campaign(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(campaign_id): Path<Uuid>,
    Json(payload): Json<UpdateCampaignRequest>,
) -> impl IntoResponse {
    // Check if user is DM of this campaign
    let is_dm = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM campaigns WHERE id = $1 AND dm_id = $2)"
    )
    .bind(campaign_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !is_dm {
        return (StatusCode::FORBIDDEN, "Only the DM can update campaigns").into_response();
    }

    let now = Utc::now();
    let res = sqlx::query_as::<_, Campaign>(
        "UPDATE campaigns SET name = COALESCE($1, name), description = $2, settings = COALESCE($3, settings), updated_at = $4 WHERE id = $5 RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.settings)
    .bind(now)
    .bind(campaign_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(campaign) => {
            let response = CampaignResponse {
                id: campaign.id,
                name: campaign.name,
                description: campaign.description,
                dm_id: campaign.dm_id,
                settings: campaign.settings,
                created_at: campaign.created_at,
                updated_at: campaign.updated_at,
            };
            axum::Json(response).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update campaign").into_response(),
    }
}

pub async fn delete_campaign(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(campaign_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check if user is DM of this campaign
    let is_dm = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM campaigns WHERE id = $1 AND dm_id = $2)"
    )
    .bind(campaign_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !is_dm {
        return (StatusCode::FORBIDDEN, "Only the DM can delete campaigns").into_response();
    }

    let res = sqlx::query("DELETE FROM campaigns WHERE id = $1")
        .bind(campaign_id)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "Campaign deleted").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete campaign").into_response(),
    }
}

// Session handlers
#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub campaign_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub game_state: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_session(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    // Check if user is DM of this campaign or a player
    let has_access = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM campaigns WHERE id = $1 AND dm_id = $2) OR EXISTS(SELECT 1 FROM campaign_players WHERE campaign_id = $1 AND player_id = $2)"
    )
    .bind(payload.campaign_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !has_access {
        return (StatusCode::FORBIDDEN, "Access denied to this campaign").into_response();
    }

    let session_id = Uuid::new_v4();
    let now = Utc::now();
    
    let res = sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (id, campaign_id, name, description, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
    )
    .bind(session_id)
    .bind(payload.campaign_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind("planned")
    .bind(serde_json::json!({}))
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(session) => {
            let response = SessionResponse {
                id: session.id,
                campaign_id: session.campaign_id,
                name: session.name,
                description: session.description,
                status: session.status,
                started_at: session.started_at,
                ended_at: session.ended_at,
                game_state: session.game_state,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            (StatusCode::CREATED, axum::Json(response)).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session").into_response(),
    }
}

pub async fn list_sessions(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
) -> impl IntoResponse {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT s.* FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE c.dm_id = $1 OR s.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $1)
         ORDER BY s.created_at DESC"
    )
    .bind(user.0)
    .fetch_all(&pool)
    .await;

    match sessions {
        Ok(sessions) => {
            let responses: Vec<SessionResponse> = sessions.into_iter().map(|s| SessionResponse {
                id: s.id,
                campaign_id: s.campaign_id,
                name: s.name,
                description: s.description,
                status: s.status,
                started_at: s.started_at,
                ended_at: s.ended_at,
                game_state: s.game_state,
                created_at: s.created_at,
                updated_at: s.updated_at,
            }).collect();
            axum::Json(responses).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch sessions").into_response(),
    }
}

pub async fn get_session(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    let session = sqlx::query_as::<_, Session>(
        "SELECT s.* FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE s.id = $1 AND (c.dm_id = $2 OR s.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2))"
    )
    .bind(session_id)
    .bind(user.0)
    .fetch_optional(&pool)
    .await;

    match session {
        Ok(Some(session)) => {
            let response = SessionResponse {
                id: session.id,
                campaign_id: session.campaign_id,
                name: session.name,
                description: session.description,
                status: session.status,
                started_at: session.started_at,
                ended_at: session.ended_at,
                game_state: session.game_state,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            axum::Json(response).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, "Session not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch session").into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateSessionRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub game_state: Option<serde_json::Value>,
}

pub async fn update_session(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<UpdateSessionRequest>,
) -> impl IntoResponse {
    // Check if user is DM of this session's campaign
    let is_dm = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE s.id = $1 AND c.dm_id = $2)"
    )
    .bind(session_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !is_dm {
        return (StatusCode::FORBIDDEN, "Only the DM can update sessions").into_response();
    }

    let now = Utc::now();
    let res = sqlx::query_as::<_, Session>(
        "UPDATE sessions SET name = COALESCE($1, name), status = COALESCE($2, status), game_state = COALESCE($3, game_state), updated_at = $4 WHERE id = $5 RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.status)
    .bind(&payload.game_state)
    .bind(now)
    .bind(session_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(session) => {
            let response = SessionResponse {
                id: session.id,
                campaign_id: session.campaign_id,
                name: session.name,
                description: session.description,
                status: session.status,
                started_at: session.started_at,
                ended_at: session.ended_at,
                game_state: session.game_state,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            axum::Json(response).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update session").into_response(),
    }
}

pub async fn start_session(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check if user is DM of this session's campaign
    let is_dm = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE s.id = $1 AND c.dm_id = $2)"
    )
    .bind(session_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !is_dm {
        return (StatusCode::FORBIDDEN, "Only the DM can start sessions").into_response();
    }

    let now = Utc::now();
    let res = sqlx::query_as::<_, Session>(
        "UPDATE sessions SET status = 'active', started_at = $1, updated_at = $1 WHERE id = $2 RETURNING *"
    )
    .bind(now)
    .bind(session_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(session) => {
            let response = SessionResponse {
                id: session.id,
                campaign_id: session.campaign_id,
                name: session.name,
                description: session.description,
                status: session.status,
                started_at: session.started_at,
                ended_at: session.ended_at,
                game_state: session.game_state,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            axum::Json(response).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to start session").into_response(),
    }
}

pub async fn end_session(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check if user is DM of this session's campaign
    let is_dm = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE s.id = $1 AND c.dm_id = $2)"
    )
    .bind(session_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !is_dm {
        return (StatusCode::FORBIDDEN, "Only the DM can end sessions").into_response();
    }

    let now = Utc::now();
    let res = sqlx::query_as::<_, Session>(
        "UPDATE sessions SET status = 'ended', ended_at = $1, updated_at = $1 WHERE id = $2 RETURNING *"
    )
    .bind(now)
    .bind(session_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(session) => {
            let response = SessionResponse {
                id: session.id,
                campaign_id: session.campaign_id,
                name: session.name,
                description: session.description,
                status: session.status,
                started_at: session.started_at,
                ended_at: session.ended_at,
                game_state: session.game_state,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            axum::Json(response).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to end session").into_response(),
    }
}

// Character handlers
#[derive(Deserialize)]
pub struct CreateCharacterRequest {
    pub campaign_id: Uuid,
    pub name: String,
    pub race: Option<String>,
    pub class: Option<String>,
    pub level: Option<i32>,
    pub hp_max: Option<i32>,
    pub ac: Option<i32>,
    pub speed: Option<i32>,
    pub stats: Option<serde_json::Value>,
    pub inventory: Option<serde_json::Value>,
    pub spells: Option<serde_json::Value>,
    pub features: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct CharacterResponse {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub player_id: Option<Uuid>,
    pub name: String,
    pub race: Option<String>,
    pub class: Option<String>,
    pub level: i32,
    pub hp_current: Option<i32>,
    pub hp_max: Option<i32>,
    pub ac: Option<i32>,
    pub speed: Option<i32>,
    pub stats: serde_json::Value,
    pub inventory: serde_json::Value,
    pub spells: serde_json::Value,
    pub features: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_character(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<CreateCharacterRequest>,
) -> impl IntoResponse {
    // Check if user has access to this campaign
    let has_access = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM campaigns WHERE id = $1 AND dm_id = $2) OR EXISTS(SELECT 1 FROM campaign_players WHERE campaign_id = $1 AND player_id = $2)"
    )
    .bind(payload.campaign_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !has_access {
        return (StatusCode::FORBIDDEN, "Access denied to this campaign").into_response();
    }

    let character_id = Uuid::new_v4();
    let now = Utc::now();
    let level = payload.level.unwrap_or(1);
    let hp_current = payload.hp_max; // Start with max HP
    
    let res = sqlx::query_as::<_, Character>(
        "INSERT INTO characters (id, campaign_id, player_id, name, race, class, level, hp_current, hp_max, ac, speed, stats, inventory, spells, features, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) RETURNING *"
    )
    .bind(character_id)
    .bind(payload.campaign_id)
    .bind(user.0) // Assign to current user by default
    .bind(&payload.name)
    .bind(&payload.race)
    .bind(&payload.class)
    .bind(level)
    .bind(hp_current)
    .bind(payload.hp_max)
    .bind(payload.ac)
    .bind(payload.speed)
    .bind(payload.stats.unwrap_or_else(|| serde_json::json!({})))
    .bind(payload.inventory.unwrap_or_else(|| serde_json::json!([])))
    .bind(payload.spells.unwrap_or_else(|| serde_json::json!([])))
    .bind(payload.features.unwrap_or_else(|| serde_json::json!([])))
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(character) => {
            let response = CharacterResponse {
                id: character.id,
                campaign_id: character.campaign_id,
                player_id: character.player_id,
                name: character.name,
                race: character.race,
                class: character.class,
                level: character.level,
                hp_current: character.hp_current,
                hp_max: character.hp_max,
                ac: character.ac,
                speed: character.speed,
                stats: character.stats,
                inventory: character.inventory,
                spells: character.spells,
                features: character.features,
                created_at: character.created_at,
                updated_at: character.updated_at,
            };
            (StatusCode::CREATED, axum::Json(response)).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create character").into_response(),
    }
}

pub async fn list_characters(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
) -> impl IntoResponse {
    let characters = sqlx::query_as::<_, Character>(
        "SELECT c.* FROM characters c 
         INNER JOIN campaigns cam ON c.campaign_id = cam.id 
         WHERE cam.dm_id = $1 OR c.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $1)
         ORDER BY c.created_at DESC"
    )
    .bind(user.0)
    .fetch_all(&pool)
    .await;

    match characters {
        Ok(characters) => {
            let responses: Vec<CharacterResponse> = characters.into_iter().map(|c| CharacterResponse {
                id: c.id,
                campaign_id: c.campaign_id,
                player_id: c.player_id,
                name: c.name,
                race: c.race,
                class: c.class,
                level: c.level,
                hp_current: c.hp_current,
                hp_max: c.hp_max,
                ac: c.ac,
                speed: c.speed,
                stats: c.stats,
                inventory: c.inventory,
                spells: c.spells,
                features: c.features,
                created_at: c.created_at,
                updated_at: c.updated_at,
            }).collect();
            axum::Json(responses).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch characters").into_response(),
    }
}

pub async fn get_character(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(character_id): Path<Uuid>,
) -> impl IntoResponse {
    let character = sqlx::query_as::<_, Character>(
        "SELECT c.* FROM characters c 
         INNER JOIN campaigns cam ON c.campaign_id = cam.id 
         WHERE c.id = $1 AND (cam.dm_id = $2 OR c.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2))"
    )
    .bind(character_id)
    .bind(user.0)
    .fetch_optional(&pool)
    .await;

    match character {
        Ok(Some(character)) => {
            let response = CharacterResponse {
                id: character.id,
                campaign_id: character.campaign_id,
                player_id: character.player_id,
                name: character.name,
                race: character.race,
                class: character.class,
                level: character.level,
                hp_current: character.hp_current,
                hp_max: character.hp_max,
                ac: character.ac,
                speed: character.speed,
                stats: character.stats,
                inventory: character.inventory,
                spells: character.spells,
                features: character.features,
                created_at: character.created_at,
                updated_at: character.updated_at,
            };
            axum::Json(response).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, "Character not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch character").into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateCharacterRequest {
    pub name: Option<String>,
    pub race: Option<String>,
    pub class: Option<String>,
    pub level: Option<i32>,
    pub hp_current: Option<i32>,
    pub hp_max: Option<i32>,
    pub ac: Option<i32>,
    pub speed: Option<i32>,
    pub stats: Option<serde_json::Value>,
    pub inventory: Option<serde_json::Value>,
    pub spells: Option<serde_json::Value>,
    pub features: Option<serde_json::Value>,
}

pub async fn update_character(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<UpdateCharacterRequest>,
) -> impl IntoResponse {
    // Check if user owns this character or is DM of the campaign
    let has_access = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM characters c 
         INNER JOIN campaigns cam ON c.campaign_id = cam.id 
         WHERE c.id = $1 AND (c.player_id = $2 OR cam.dm_id = $2))"
    )
    .bind(character_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !has_access {
        return (StatusCode::FORBIDDEN, "Access denied to this character").into_response();
    }

    let now = Utc::now();
    let res = sqlx::query_as::<_, Character>(
        "UPDATE characters SET 
         name = COALESCE($1, name), 
         race = $2, 
         class = $3, 
         level = COALESCE($4, level), 
         hp_current = $5, 
         hp_max = $6, 
         ac = $7, 
         speed = $8, 
         stats = COALESCE($9, stats), 
         inventory = COALESCE($10, inventory), 
         spells = COALESCE($11, spells), 
         features = COALESCE($12, features), 
         updated_at = $13 
         WHERE id = $14 RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.race)
    .bind(&payload.class)
    .bind(payload.level)
    .bind(payload.hp_current)
    .bind(payload.hp_max)
    .bind(payload.ac)
    .bind(payload.speed)
    .bind(&payload.stats)
    .bind(&payload.inventory)
    .bind(&payload.spells)
    .bind(&payload.features)
    .bind(now)
    .bind(character_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(character) => {
            let response = CharacterResponse {
                id: character.id,
                campaign_id: character.campaign_id,
                player_id: character.player_id,
                name: character.name,
                race: character.race,
                class: character.class,
                level: character.level,
                hp_current: character.hp_current,
                hp_max: character.hp_max,
                ac: character.ac,
                speed: character.speed,
                stats: character.stats,
                inventory: character.inventory,
                spells: character.spells,
                features: character.features,
                created_at: character.created_at,
                updated_at: character.updated_at,
            };
            axum::Json(response).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update character").into_response(),
    }
}

pub async fn delete_character(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(character_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check if user owns this character or is DM of the campaign
    let has_access = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM characters c 
         INNER JOIN campaigns cam ON c.campaign_id = cam.id 
         WHERE c.id = $1 AND (c.player_id = $2 OR cam.dm_id = $2))"
    )
    .bind(character_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !has_access {
        return (StatusCode::FORBIDDEN, "Access denied to this character").into_response();
    }

    let res = sqlx::query("DELETE FROM characters WHERE id = $1")
        .bind(character_id)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "Character deleted").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete character").into_response(),
    }
}

// Game state handlers
#[derive(Deserialize)]
pub struct UpdateInitiativeRequest {
    pub session_id: Uuid,
    pub initiative_order: Vec<InitiativeEntry>,
    pub current_turn: Option<Uuid>,
    pub round: Option<i32>,
    pub combat_active: Option<bool>,
}

pub async fn update_initiative(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<UpdateInitiativeRequest>,
) -> impl IntoResponse {
    // Check if user is DM of this session's campaign
    let is_dm = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE s.id = $1 AND c.dm_id = $2)"
    )
    .bind(payload.session_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !is_dm {
        return (StatusCode::FORBIDDEN, "Only the DM can update initiative").into_response();
    }

    // Get current game state
    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE id = $1"
    )
    .bind(payload.session_id)
    .fetch_one(&pool)
    .await;

    let session = match session {
        Ok(s) => s,
        Err(_) => return (StatusCode::NOT_FOUND, "Session not found").into_response(),
    };

    // Parse current game state
    let mut game_state: GameState = serde_json::from_value(session.game_state.clone())
        .unwrap_or_else(|_| GameState {
            initiative_order: Vec::new(),
            current_turn: None,
            round: 1,
            combat_active: false,
            conditions: Vec::new(),
        });

    // Update game state
    game_state.initiative_order = payload.initiative_order;
    game_state.current_turn = payload.current_turn;
    if let Some(round) = payload.round {
        game_state.round = round;
    }
    if let Some(combat_active) = payload.combat_active {
        game_state.combat_active = combat_active;
    }

    // Save updated game state
    let now = Utc::now();
    let res = sqlx::query_as::<_, Session>(
        "UPDATE sessions SET game_state = $1, updated_at = $2 WHERE id = $3 RETURNING *"
    )
    .bind(serde_json::to_value(game_state).unwrap())
    .bind(now)
    .bind(payload.session_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(_) => (StatusCode::OK, "Initiative updated").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update initiative").into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateCharacterHPRequest {
    pub hp_current: i32,
    pub hp_max: Option<i32>,
}

pub async fn update_character_hp(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(character_id): Path<Uuid>,
    Json(payload): Json<UpdateCharacterHPRequest>,
) -> impl IntoResponse {
    // Check if user owns this character or is DM of the campaign
    let has_access = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM characters c 
         INNER JOIN campaigns cam ON c.campaign_id = cam.id 
         WHERE c.id = $1 AND (c.player_id = $2 OR cam.dm_id = $2))"
    )
    .bind(character_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !has_access {
        return (StatusCode::FORBIDDEN, "Access denied to this character").into_response();
    }

    let now = Utc::now();
    let res = sqlx::query_as::<_, Character>(
        "UPDATE characters SET hp_current = $1, hp_max = COALESCE($2, hp_max), updated_at = $3 WHERE id = $4 RETURNING *"
    )
    .bind(payload.hp_current)
    .bind(payload.hp_max)
    .bind(now)
    .bind(character_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(character) => {
            let response = CharacterResponse {
                id: character.id,
                campaign_id: character.campaign_id,
                player_id: character.player_id,
                name: character.name,
                race: character.race,
                class: character.class,
                level: character.level,
                hp_current: character.hp_current,
                hp_max: character.hp_max,
                ac: character.ac,
                speed: character.speed,
                stats: character.stats,
                inventory: character.inventory,
                spells: character.spells,
                features: character.features,
                created_at: character.created_at,
                updated_at: character.updated_at,
            };
            axum::Json(response).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update character HP").into_response(),
    }
}

// Event Log handlers
#[derive(Deserialize)]
pub struct CreateEventLogRequest {
    pub session_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
}

#[derive(Serialize)]
pub struct EventLogResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_event_log(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<CreateEventLogRequest>,
) -> impl IntoResponse {
    // Check if user has access to this session
    let session_access = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE s.id = $1 AND (c.dm_id = $2 OR s.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2))"
    )
    .bind(payload.session_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await;

    match session_access {
        Ok(count) if count > 0 => {
            let event_id = Uuid::new_v4();
            let now = Utc::now();
            
            let event_log = sqlx::query_as::<_, EventLog>(
                "INSERT INTO event_logs (id, session_id, event_type, event_data, created_by, created_at) 
                 VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
            )
            .bind(event_id)
            .bind(payload.session_id)
            .bind(&payload.event_type)
            .bind(&payload.event_data)
            .bind(user.0)
            .bind(now)
            .fetch_one(&pool)
            .await;

            match event_log {
                Ok(event) => (
                    StatusCode::CREATED,
                    axum::Json(EventLogResponse {
                        id: event.id,
                        session_id: event.session_id,
                        event_type: event.event_type,
                        event_data: event.event_data,
                        created_by: event.created_by,
                        created_at: event.created_at,
                    })
                ).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create event log").into_response(),
            }
        }
        _ => (StatusCode::FORBIDDEN, "Access denied to this session").into_response(),
    }
}

pub async fn list_event_logs(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check if user has access to this session
    let session_access = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sessions s 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE s.id = $1 AND (c.dm_id = $2 OR s.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2))"
    )
    .bind(session_id)
    .bind(user.0)
    .fetch_one(&pool)
    .await;

    match session_access {
        Ok(count) if count > 0 => {
            let events = sqlx::query_as::<_, EventLog>(
                "SELECT * FROM event_logs WHERE session_id = $1 ORDER BY created_at ASC"
            )
            .bind(session_id)
            .fetch_all(&pool)
            .await;

            match events {
                Ok(events) => {
                    let responses: Vec<EventLogResponse> = events.into_iter().map(|e| EventLogResponse {
                        id: e.id,
                        session_id: e.session_id,
                        event_type: e.event_type,
                        event_data: e.event_data,
                        created_by: e.created_by,
                        created_at: e.created_at,
                    }).collect();
                    
                    axum::Json(responses).into_response()
                }
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch event logs").into_response(),
            }
        }
        _ => (StatusCode::FORBIDDEN, "Access denied to this session").into_response(),
    }
}

pub async fn get_event_log(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    // Check if user has access to this event's session
    let event = sqlx::query_as::<_, EventLog>(
        "SELECT el.* FROM event_logs el 
         INNER JOIN sessions s ON el.session_id = s.id 
         INNER JOIN campaigns c ON s.campaign_id = c.id 
         WHERE el.id = $1 AND (c.dm_id = $2 OR s.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2))"
    )
    .bind(event_id)
    .bind(user.0)
    .fetch_optional(&pool)
    .await;

    match event {
        Ok(Some(event)) => {
            let response = EventLogResponse {
                id: event.id,
                session_id: event.session_id,
                event_type: event.event_type,
                event_data: event.event_data,
                created_by: event.created_by,
                created_at: event.created_at,
            };
            axum::Json(response).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Event log not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch event log").into_response(),
    }
}

// AI Integration handlers
#[derive(Deserialize)]
pub struct AIRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub session_id: Option<Uuid>,
    pub request_type: String, // "npc", "location", "encounter", "description", "chat"
}

#[derive(Serialize)]
pub struct AIResponse {
    pub response: String,
    pub tokens_used: Option<i32>,
    pub model: String,
}

pub async fn ai_generate(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<AIRequest>,
) -> impl IntoResponse {
    // For now, return a mock response
    // TODO: Implement actual AI integration
    let response = match payload.request_type.as_str() {
        "npc" => {
            format!("Generated NPC: A mysterious figure with a weathered cloak and piercing eyes. They seem to know more than they let on...")
        }
        "location" => {
            format!("Generated Location: A dimly lit tavern with smoke curling from the fireplace. The wooden beams creak with age, and the air is thick with the smell of ale and adventure.")
        }
        "encounter" => {
            format!("Generated Encounter: A group of bandits has set up an ambush in the forest. They're well-armed and seem desperate, suggesting they might be open to negotiation.")
        }
        "description" => {
            format!("Enhanced Description: The ancient castle looms before you, its weathered stone walls bearing the scars of countless battles. Torches flicker in the arrow slits, casting dancing shadows that seem to move of their own accord.")
        }
        "chat" => {
            format!("AI Assistant: Based on the current situation, I'd suggest considering the diplomatic approach. The goblins seem nervous and might be more interested in survival than combat.")
        }
        _ => {
            format!("AI Response: I'm here to help with your D&D session. What would you like me to assist with?")
        }
    };

    // Log the AI request as an event if session_id is provided
    if let Some(session_id) = payload.session_id {
        let _ = sqlx::query(
            "INSERT INTO event_logs (id, session_id, event_type, event_data, created_by, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(Uuid::new_v4())
        .bind(session_id)
        .bind("ai_request")
        .bind(serde_json::json!({
            "prompt": payload.prompt,
            "request_type": payload.request_type,
            "response": response
        }))
        .bind(user.0)
        .bind(Utc::now())
        .execute(&pool)
        .await;
    }

    let ai_response = AIResponse {
        response,
        tokens_used: Some(150), // Mock value
        model: "gpt-4".to_string(),
    };

    axum::Json(ai_response).into_response()
}

mod tests {
    use super::*;
    use sqlx::PgPool;
    use axum::http::StatusCode;
    use serde_json::json;

    async fn create_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }

    #[tokio::test]
    async fn test_register_user() {
        let pool = create_test_pool().await;
        
        // Clean up any existing test user
        sqlx::query("DELETE FROM users WHERE email = 'test@example.com'")
            .execute(&pool)
            .await
            .ok();

        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let response = register(Extension(pool), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_register_duplicate_user() {
        let pool = create_test_pool().await;
        
        // Create a user first
        let request = RegisterRequest {
            email: "duplicate@example.com".to_string(),
            username: "duplicateuser".to_string(),
            password: "testpass".to_string(),
        };

        register(Extension(pool.clone()), Json(request.clone())).await;

        // Try to register the same user again
        let response = register(Extension(pool), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_login_success() {
        let pool = create_test_pool().await;
        
        // Create a user first
        let register_request = RegisterRequest {
            email: "login@example.com".to_string(),
            username: "loginuser".to_string(),
            password: "testpass".to_string(),
        };

        register(Extension(pool.clone()), Json(register_request)).await;

        // Try to login
        let login_request = LoginRequest {
            email: "login@example.com".to_string(),
            password: "testpass".to_string(),
        };

        let response = login(Extension(pool), Json(login_request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
        
        // Parse the response to check for token
        let body_bytes = axum::body::to_bytes(response_parts.1, usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        let login_response: LoginResponse = serde_json::from_str(&response_text).unwrap();
        
        assert!(!login_response.token.is_empty());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let pool = create_test_pool().await;
        
        let login_request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "wrongpass".to_string(),
        };

        let response = login(Extension(pool), Json(login_request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_create_campaign() {
        let pool = create_test_pool().await;
        
        // Create a test user first
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("dm{}@example.com", timestamp))
            .bind(format!("dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = CreateCampaignRequest {
            name: "Test Campaign".to_string(),
            description: Some("A test campaign".to_string()),
            settings: Some(json!({"theme": "dark"})),
        };

        let auth_user = AuthUser(user_id);
        let response = create_campaign(Extension(pool), Extension(auth_user), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_list_campaigns() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("list{}@example.com", timestamp))
            .bind(format!("listuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("List Test Campaign")
            .bind("A campaign for testing list")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = list_campaigns(Extension(pool), Extension(auth_user)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_campaign() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("get{}@example.com", timestamp))
            .bind(format!("getuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Get Test Campaign")
            .bind("A campaign for testing get")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = get_campaign(Extension(pool), Extension(auth_user), Path(campaign_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_campaign() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("update{}@example.com", timestamp))
            .bind(format!("updateuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Update Test Campaign")
            .bind("A campaign for testing update")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = UpdateCampaignRequest {
            name: Some("Updated Campaign Name".to_string()),
            description: Some("Updated description".to_string()),
            settings: Some(json!({"theme": "light"})),
        };

        let auth_user = AuthUser(user_id);
        let response = update_campaign(Extension(pool), Extension(auth_user), Path(campaign_id), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_campaign() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("delete{}@example.com", timestamp))
            .bind(format!("deleteuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Delete Test Campaign")
            .bind("A campaign for testing delete")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = delete_campaign(Extension(pool), Extension(auth_user), Path(campaign_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_session() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("session_dm{}@example.com", timestamp))
            .bind(format!("session_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Session Test Campaign")
            .bind("A campaign for testing sessions")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = CreateSessionRequest {
            campaign_id: campaign_id,
            name: "Test Session".to_string(),
            description: Some("A test session".to_string()),
        };

        let auth_user = AuthUser(user_id);
        let response = create_session(Extension(pool), Extension(auth_user), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("list_sessions_dm{}@example.com", timestamp))
            .bind(format!("list_sessions_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("List Sessions Campaign")
            .bind("A campaign for testing list sessions")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("List Session 1")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = list_sessions(Extension(pool), Extension(auth_user)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_session() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("get_session_dm{}@example.com", timestamp))
            .bind(format!("get_session_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Get Session Campaign")
            .bind("A campaign for testing get session")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Get Session 1")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = get_session(Extension(pool), Extension(auth_user), Path(session_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_session() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("update_session_dm{}@example.com", timestamp))
            .bind(format!("update_session_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Update Session Campaign")
            .bind("A campaign for testing update session")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Update Session 1")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = UpdateSessionRequest {
            name: Some("Updated Session Name".to_string()),
            status: Some("active".to_string()),
            game_state: Some(json!({"test": "updated"})),
        };

        let auth_user = AuthUser(user_id);
        let response = update_session(Extension(pool), Extension(auth_user), Path(session_id), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_start_session() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("start_session_dm{}@example.com", timestamp))
            .bind(format!("start_session_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Start Session Campaign")
            .bind("A campaign for testing start session")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Start Session 1")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = start_session(Extension(pool), Extension(auth_user), Path(session_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_end_session() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("end_session_dm{}@example.com", timestamp))
            .bind(format!("end_session_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("End Session Campaign")
            .bind("A campaign for testing end session")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("End Session 1")
            .bind("active")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = end_session(Extension(pool), Extension(auth_user), Path(session_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_session_access_control() {
        let pool = create_test_pool().await;
        
        // Create two users
        let dm_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(dm_id)
            .bind(format!("session_dm_access{}@example.com", timestamp))
            .bind(format!("session_dm_accessuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(player_id)
            .bind(format!("session_player_access{}@example.com", timestamp))
            .bind(format!("session_player_accessuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        // Create a campaign
        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Access Control Campaign")
            .bind("A campaign for testing access control")
            .bind(dm_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        // Add player to campaign
        sqlx::query("INSERT INTO campaign_players (campaign_id, player_id) VALUES ($1, $2)")
            .bind(campaign_id)
            .bind(player_id)
            .execute(&pool)
            .await
            .unwrap();

        // Create a session
        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Access Control Session")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        // Test that DM can access session
        let dm_auth = AuthUser(dm_id);
        let dm_response = get_session(Extension(pool.clone()), Extension(dm_auth), Path(session_id)).await;
        let dm_response_parts = dm_response.into_response().into_parts();
        assert_eq!(dm_response_parts.0.status, StatusCode::OK);

        // Test that player can access session
        let player_auth = AuthUser(player_id);
        let player_response = get_session(Extension(pool.clone()), Extension(player_auth), Path(session_id)).await;
        let player_response_parts = player_response.into_response().into_parts();
        assert_eq!(player_response_parts.0.status, StatusCode::OK);

        // Test that unauthorized user cannot access session
        let unauthorized_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(unauthorized_id)
            .bind(format!("unauthorized{}@example.com", timestamp))
            .bind(format!("unauthorizeduser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let unauthorized_auth = AuthUser(unauthorized_id);
        let unauthorized_response = get_session(Extension(pool), Extension(unauthorized_auth), Path(session_id)).await;
        let unauthorized_response_parts = unauthorized_response.into_response().into_parts();
        assert_eq!(unauthorized_response_parts.0.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_session_status_transitions() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("status_dm{}@example.com", timestamp))
            .bind(format!("status_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Status Campaign")
            .bind("A campaign for testing status transitions")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        // Create a session
        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Status Session")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);

        // Test starting the session
        let start_response = start_session(Extension(pool.clone()), Extension(auth_user.clone()), Path(session_id)).await;
        let start_response_parts = start_response.into_response().into_parts();
        assert_eq!(start_response_parts.0.status, StatusCode::OK);

        // Verify session is now active
        let get_response = get_session(Extension(pool.clone()), Extension(auth_user.clone()), Path(session_id)).await;
        let get_response_parts = get_response.into_response().into_parts();
        assert_eq!(get_response_parts.0.status, StatusCode::OK);

        // Test ending the session
        let end_response = end_session(Extension(pool.clone()), Extension(auth_user.clone()), Path(session_id)).await;
        let end_response_parts = end_response.into_response().into_parts();
        assert_eq!(end_response_parts.0.status, StatusCode::OK);

        // Verify session is now ended
        let final_response = get_session(Extension(pool), Extension(auth_user), Path(session_id)).await;
        let final_response_parts = final_response.into_response().into_parts();
        assert_eq!(final_response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_character() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("character_dm{}@example.com", timestamp))
            .bind(format!("character_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Character Test Campaign")
            .bind("A campaign for testing characters")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = CreateCharacterRequest {
            campaign_id,
            name: "Test Character".to_string(),
            race: Some("Human".to_string()),
            class: Some("Fighter".to_string()),
            level: Some(5),
            hp_max: Some(45),
            ac: Some(18),
            speed: Some(30),
            stats: Some(json!({
                "strength": 16,
                "dexterity": 14,
                "constitution": 16,
                "intelligence": 10,
                "wisdom": 12,
                "charisma": 8
            })),
            inventory: Some(json!([])),
            spells: Some(json!([])),
            features: Some(json!([])),
        };

        let auth_user = AuthUser(user_id);
        let response = create_character(Extension(pool), Extension(auth_user), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_list_characters() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("list_characters_dm{}@example.com", timestamp))
            .bind(format!("list_characters_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("List Characters Campaign")
            .bind("A campaign for testing list characters")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let character_id = Uuid::new_v4();
        sqlx::query("INSERT INTO characters (id, campaign_id, player_id, name, race, class, level, hp_current, hp_max, ac, speed, stats, inventory, spells, features, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)")
            .bind(character_id)
            .bind(campaign_id)
            .bind(user_id)
            .bind("List Character 1")
            .bind("Elf")
            .bind("Wizard")
            .bind(3)
            .bind(20)
            .bind(20)
            .bind(12)
            .bind(30)
            .bind(json!({}))
            .bind(json!([]))
            .bind(json!([]))
            .bind(json!([]))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = list_characters(Extension(pool), Extension(auth_user)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_character() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("get_character_dm{}@example.com", timestamp))
            .bind(format!("get_character_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Get Character Campaign")
            .bind("A campaign for testing get character")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let character_id = Uuid::new_v4();
        sqlx::query("INSERT INTO characters (id, campaign_id, player_id, name, race, class, level, hp_current, hp_max, ac, speed, stats, inventory, spells, features, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)")
            .bind(character_id)
            .bind(campaign_id)
            .bind(user_id)
            .bind("Get Character 1")
            .bind("Dwarf")
            .bind("Cleric")
            .bind(4)
            .bind(32)
            .bind(32)
            .bind(16)
            .bind(25)
            .bind(json!({}))
            .bind(json!([]))
            .bind(json!([]))
            .bind(json!([]))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = get_character(Extension(pool), Extension(auth_user), Path(character_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_character() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("update_character_dm{}@example.com", timestamp))
            .bind(format!("update_character_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Update Character Campaign")
            .bind("A campaign for testing update character")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let character_id = Uuid::new_v4();
        sqlx::query("INSERT INTO characters (id, campaign_id, player_id, name, race, class, level, hp_current, hp_max, ac, speed, stats, inventory, spells, features, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)")
            .bind(character_id)
            .bind(campaign_id)
            .bind(user_id)
            .bind("Update Character 1")
            .bind("Halfling")
            .bind("Rogue")
            .bind(2)
            .bind(16)
            .bind(16)
            .bind(15)
            .bind(25)
            .bind(json!({}))
            .bind(json!([]))
            .bind(json!([]))
            .bind(json!([]))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = UpdateCharacterRequest {
            name: Some("Updated Character Name".to_string()),
            level: Some(3),
            hp_current: Some(20),
            hp_max: Some(20),
            ac: Some(16),
            race: Some("Halfling".to_string()),
            class: Some("Rogue".to_string()),
            speed: Some(25),
            stats: Some(json!({
                "strength": 8,
                "dexterity": 18,
                "constitution": 14,
                "intelligence": 12,
                "wisdom": 10,
                "charisma": 16
            })),
            inventory: Some(json!([])),
            spells: Some(json!([])),
            features: Some(json!([])),
        };

        let auth_user = AuthUser(user_id);
        let response = update_character(Extension(pool), Extension(auth_user), Path(character_id), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_character() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("delete_character_dm{}@example.com", timestamp))
            .bind(format!("delete_character_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Delete Character Campaign")
            .bind("A campaign for testing delete character")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let character_id = Uuid::new_v4();
        sqlx::query("INSERT INTO characters (id, campaign_id, player_id, name, race, class, level, hp_current, hp_max, ac, speed, stats, inventory, spells, features, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)")
            .bind(character_id)
            .bind(campaign_id)
            .bind(user_id)
            .bind("Delete Character 1")
            .bind("Gnome")
            .bind("Wizard")
            .bind(1)
            .bind(8)
            .bind(8)
            .bind(12)
            .bind(25)
            .bind(json!({}))
            .bind(json!([]))
            .bind(json!([]))
            .bind(json!([]))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = delete_character(Extension(pool), Extension(auth_user), Path(character_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_character_hp() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("update_hp_dm{}@example.com", timestamp))
            .bind(format!("update_hp_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Update HP Campaign")
            .bind("A campaign for testing update HP")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let character_id = Uuid::new_v4();
        sqlx::query("INSERT INTO characters (id, campaign_id, player_id, name, race, class, level, hp_current, hp_max, ac, speed, stats, inventory, spells, features, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)")
            .bind(character_id)
            .bind(campaign_id)
            .bind(user_id)
            .bind("HP Test Character")
            .bind("Human")
            .bind("Fighter")
            .bind(5)
            .bind(35)
            .bind(45)
            .bind(18)
            .bind(30)
            .bind(json!({}))
            .bind(json!([]))
            .bind(json!([]))
            .bind(json!([]))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = UpdateCharacterHPRequest {
            hp_current: 25,
            hp_max: Some(45),
        };

        let auth_user = AuthUser(user_id);
        let response = update_character_hp(Extension(pool), Extension(auth_user), Path(character_id), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_initiative() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("initiative_dm{}@example.com", timestamp))
            .bind(format!("initiative_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Initiative Campaign")
            .bind("A campaign for testing initiative")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Initiative Session")
            .bind("active")
            .bind(json!({
                "initiative_order": [],
                "current_turn": null,
                "round": 1,
                "combat_active": false,
                "conditions": []
            }))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let initiative_order = vec![
            InitiativeEntry {
                id: Uuid::new_v4(),
                name: "Player 1".to_string(),
                initiative: 18,
                is_player: true,
                character_id: None,
                user_id: Some(user_id),
                hp_current: Some(25),
                hp_max: Some(25),
                ac: Some(16),
            },
            InitiativeEntry {
                id: Uuid::new_v4(),
                name: "Goblin 1".to_string(),
                initiative: 15,
                is_player: false,
                character_id: None,
                user_id: None,
                hp_current: Some(7),
                hp_max: Some(7),
                ac: Some(15),
            },
        ];

        let request = UpdateInitiativeRequest {
            session_id,
            initiative_order,
            current_turn: None,
            round: Some(1),
            combat_active: Some(true),
        };

        let auth_user = AuthUser(user_id);
        let response = update_initiative(Extension(pool), Extension(auth_user), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_event_log() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("event_log_dm{}@example.com", timestamp))
            .bind(format!("event_log_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Event Log Campaign")
            .bind("A campaign for testing event logs")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Event Log Session")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = CreateEventLogRequest {
            session_id,
            event_type: "Test Event".to_string(),
            event_data: json!({"message": "This is a test event"}),
        };

        let auth_user = AuthUser(user_id);
        let response = create_event_log(Extension(pool), Extension(auth_user), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_list_event_logs() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        let random_suffix = rand::random::<u32>();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("event_log_dm{}_{}@example.com", timestamp, random_suffix))
            .bind(format!("event_log_dmuser{}_{}", timestamp, random_suffix))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Event Log Campaign")
            .bind("A campaign for testing event logs")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Event Log Session")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let event_id = Uuid::new_v4();
        sqlx::query("INSERT INTO event_logs (id, session_id, event_type, event_data, created_by, created_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(event_id)
            .bind(session_id)
            .bind("Test Event")
            .bind(json!({"message": "This is a test event"}))
            .bind(user_id)
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = list_event_logs(Extension(pool), Extension(auth_user), Path(session_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_event_log() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        let random_suffix = rand::random::<u32>();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("event_log_dm{}_{}@example.com", timestamp, random_suffix))
            .bind(format!("event_log_dmuser{}_{}", timestamp, random_suffix))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("Event Log Campaign")
            .bind("A campaign for testing event logs")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("Event Log Session")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let event_id = Uuid::new_v4();
        sqlx::query("INSERT INTO event_logs (id, session_id, event_type, event_data, created_by, created_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(event_id)
            .bind(session_id)
            .bind("Test Event")
            .bind(json!({"message": "This is a test event"}))
            .bind(user_id)
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let auth_user = AuthUser(user_id);
        let response = get_event_log(Extension(pool), Extension(auth_user), Path(event_id)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ai_generate() {
        let pool = create_test_pool().await;
        
        // Create a test user and campaign
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        sqlx::query("INSERT INTO users (id, email, username, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(user_id)
            .bind(format!("ai_dm{}@example.com", timestamp))
            .bind(format!("ai_dmuser{}", timestamp))
            .bind("hashed_password")
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let campaign_id = Uuid::new_v4();
        sqlx::query("INSERT INTO campaigns (id, name, description, dm_id, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(campaign_id)
            .bind("AI Campaign")
            .bind("A campaign for testing AI integration")
            .bind(user_id)
            .bind(json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO sessions (id, campaign_id, name, status, game_state, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(session_id)
            .bind(campaign_id)
            .bind("AI Session")
            .bind("planned")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(&pool)
            .await
            .unwrap();

        let request = AIRequest {
            prompt: "What's the weather like today?".to_string(),
            context: Some("The weather is sunny and warm.".to_string()),
            session_id: Some(session_id),
            request_type: "chat".to_string(),
        };

        let auth_user = AuthUser(user_id);
        let response = ai_generate(Extension(pool), Extension(auth_user), Json(request)).await;
        let response_parts = response.into_response().into_parts();
        
        assert_eq!(response_parts.0.status, StatusCode::OK);
    }
} 