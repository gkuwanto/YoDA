use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub dm_id: Uuid,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CampaignPlayer {
    pub campaign_id: Uuid,
    pub player_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Character {
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EventLog {
    pub id: Uuid,
    pub session_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub initiative_order: Vec<InitiativeEntry>,
    pub current_turn: Option<Uuid>,
    pub round: i32,
    pub combat_active: bool,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitiativeEntry {
    pub id: Uuid,
    pub name: String,
    pub initiative: i32,
    pub is_player: bool,
    pub character_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub hp_current: Option<i32>,
    pub hp_max: Option<i32>,
    pub ac: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Condition {
    pub target_id: Uuid,
    pub condition_type: String,
    pub duration: Option<i32>,
    pub description: String,
    pub applied_at: DateTime<Utc>,
} 