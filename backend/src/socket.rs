use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::Extension;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use chrono::DateTime;
use futures::{SinkExt, StreamExt};
use crate::models::InitiativeEntry;

// Shared state for managing active sessions and connections
#[derive(Clone)]
pub struct SessionState {
    pub sessions: Arc<RwLock<HashMap<Uuid, SessionInfo>>>,
}

#[derive(Clone)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub campaign_id: Uuid,
    pub connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub user_id: Uuid,
    pub username: String,
    pub is_dm: bool,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    JoinSession { session_id: Uuid },
    LeaveSession { session_id: Uuid },
    DiceRoll { dice: String, reason: Option<String> },
    ChatMessage { message: String },
    UpdateGameState { game_state: serde_json::Value },
    PlayerAction { action: String, data: serde_json::Value },
    UpdateCharacter { character_id: Uuid, updates: serde_json::Value },
    UpdateInitiative { session_id: Uuid, initiative_order: Vec<InitiativeEntry> },
    NextTurn { session_id: Uuid },
    UpdateHP { character_id: Uuid, hp_current: i32, hp_max: Option<i32> },
    CreateEventLog { session_id: Uuid, event_type: String, event_data: serde_json::Value },
    AIRequest { prompt: String, request_type: String, context: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    SessionJoined { session_id: Uuid, players: Vec<PlayerInfo> },
    PlayerJoined { player: PlayerInfo },
    PlayerLeft { player_id: Uuid },
    DiceRolled { player_id: Uuid, result: DiceResult },
    ChatMessage { player_id: Uuid, message: String, timestamp: DateTime<Utc> },
    GameStateUpdated { game_state: serde_json::Value },
    CharacterUpdated { character: CharacterInfo },
    InitiativeUpdated { session_id: Uuid, initiative_order: Vec<InitiativeEntry>, current_turn: Option<Uuid> },
    TurnChanged { session_id: Uuid, current_turn: Uuid, round: i32 },
    HPUpdated { character_id: Uuid, hp_current: i32, hp_max: i32 },
    EventLogCreated { event_id: Uuid, event_type: String, event_data: serde_json::Value, created_by: Uuid, created_at: DateTime<Utc> },
    AIResponse { response: String, request_type: String, tokens_used: Option<i32>, model: String },
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub user_id: Uuid,
    pub username: String,
    pub is_dm: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterInfo {
    pub id: Uuid,
    pub name: String,
    pub race: Option<String>,
    pub class: Option<String>,
    pub level: i32,
    pub hp_current: Option<i32>,
    pub hp_max: Option<i32>,
    pub ac: Option<i32>,
    pub speed: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiceResult {
    pub dice: String,
    pub result: i32,
    pub rolls: Vec<i32>,
    pub reason: Option<String>,
}

// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(pool): Extension<PgPool>,
    Extension(session_state): Extension<SessionState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, pool, session_state))
}

async fn handle_socket(socket: WebSocket, pool: PgPool, session_state: SessionState) {
    let (mut sender, mut receiver) = socket.split();
    
    // TODO: Extract user info from JWT token in WebSocket upgrade
    // For now, we'll use a placeholder user
    let user_id = Uuid::new_v4();
    let username = "Anonymous".to_string();
    let is_dm = false;
    
    let mut current_session: Option<Uuid> = None;
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        match handle_client_message(
                            client_msg,
                            &pool,
                            &session_state,
                            user_id,
                            &username,
                            is_dm,
                            &mut current_session,
                        ).await {
                            Ok(server_msg) => {
                                if let Err(e) = sender.send(Message::Text(serde_json::to_string(&server_msg).unwrap())).await {
                                    eprintln!("Failed to send message: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                let error_msg = ServerMessage::Error { message: e };
                                if let Err(e) = sender.send(Message::Text(serde_json::to_string(&error_msg).unwrap())).await {
                                    eprintln!("Failed to send error message: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse client message: {}", e);
                        let error_msg = ServerMessage::Error { message: "Invalid message format".to_string() };
                        if let Err(e) = sender.send(Message::Text(serde_json::to_string(&error_msg).unwrap())).await {
                            eprintln!("Failed to send error message: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                // Handle disconnect
                if let Some(session_id) = current_session {
                    leave_session(&session_state, session_id, user_id).await;
                }
                break;
            }
            _ => {}
        }
    }
}

async fn handle_client_message(
    msg: ClientMessage,
    pool: &PgPool,
    session_state: &SessionState,
    user_id: Uuid,
    username: &str,
    is_dm: bool,
    current_session: &mut Option<Uuid>,
) -> Result<ServerMessage, String> {
    match msg {
        ClientMessage::JoinSession { session_id } => {
            // Verify user has access to this session
            let has_access = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM sessions s 
                 INNER JOIN campaigns c ON s.campaign_id = c.id 
                 WHERE s.id = $1 AND (c.dm_id = $2 OR s.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2)))"
            )
            .bind(session_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

            if !has_access {
                return Err("Access denied to this session".to_string());
            }

            // Join the session
            join_session(session_state, session_id, user_id, username, is_dm, pool).await;
            *current_session = Some(session_id);

            // Get current players in session
            let players = get_session_players(session_state, session_id).await;

            Ok(ServerMessage::SessionJoined { session_id, players })
        }
        
        ClientMessage::LeaveSession { session_id } => {
            leave_session(session_state, session_id, user_id).await;
            *current_session = None;
            Ok(ServerMessage::PlayerLeft { player_id: user_id })
        }
        
        ClientMessage::DiceRoll { dice, reason } => {
            let result = roll_dice(&dice)?;
            let dice_result = DiceResult {
                dice: dice.clone(),
                result: result.total,
                rolls: result.rolls,
                reason,
            };
            
            // Broadcast to all players in the session
            if let Some(session_id) = current_session {
                broadcast_to_session(session_state, *session_id, &ServerMessage::DiceRolled {
                    player_id: user_id,
                    result: dice_result.clone(),
                }).await;
            }
            
            Ok(ServerMessage::DiceRolled {
                player_id: user_id,
                result: dice_result,
            })
        }
        
        ClientMessage::ChatMessage { message } => {
            let timestamp = Utc::now();
            let chat_msg = ServerMessage::ChatMessage {
                player_id: user_id,
                message,
                timestamp,
            };
            
            // Broadcast to all players in the session
            if let Some(session_id) = current_session {
                broadcast_to_session(session_state, *session_id, &chat_msg).await;
            }
            
            Ok(chat_msg)
        }
        
        ClientMessage::UpdateGameState { game_state } => {
            // Only DM can update game state
            if !is_dm {
                return Err("Only the DM can update game state".to_string());
            }
            
            // Update game state in database
            if let Some(session_id) = current_session {
                sqlx::query("UPDATE sessions SET game_state = $1, updated_at = $2 WHERE id = $3")
                    .bind(&game_state)
                    .bind(Utc::now())
                    .bind(*session_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to update game state: {}", e))?;
                
                // Broadcast to all players
                let update_msg = ServerMessage::GameStateUpdated { game_state };
                broadcast_to_session(session_state, *session_id, &update_msg).await;
                
                Ok(update_msg)
            } else {
                Err("Not in a session".to_string())
            }
        }
        
        ClientMessage::PlayerAction { action: _, data: _ } => {
            // Handle various player actions (movement, attacks, etc.)
            // This is a placeholder for future implementation
            Ok(ServerMessage::Error { message: "Player actions not yet implemented".to_string() })
        }
        
        ClientMessage::UpdateCharacter { character_id, updates: _ } => {
            // Check if user owns this character or is DM of the campaign
            let has_access = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM characters c 
                 INNER JOIN campaigns cam ON c.campaign_id = cam.id 
                 WHERE c.id = $1 AND (c.player_id = $2 OR cam.dm_id = $2))"
            )
            .bind(character_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

            if !has_access {
                return Err("Access denied to this character".to_string());
            }

            // Additional validation: Check if character belongs to current session's campaign
            if let Some(session_id) = current_session {
                if let Some(session_campaign_id) = get_session_campaign_id(session_state, *session_id).await {
                    let character_campaign_id = sqlx::query_scalar::<_, Option<Uuid>>(
                        "SELECT campaign_id FROM characters WHERE id = $1"
                    )
                    .bind(character_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?;

                    if character_campaign_id != Some(session_campaign_id) {
                        return Err("Character does not belong to current session's campaign".to_string());
                    }
                }
            }

            // Update character in database
            let now = Utc::now();
            let res = sqlx::query_as::<_, crate::models::Character>(
                "UPDATE characters SET updated_at = $1 WHERE id = $2 RETURNING *"
            )
            .bind(now)
            .bind(character_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to update character: {}", e))?;

            // Create character info for broadcast
            let character_info = CharacterInfo {
                id: res.id,
                name: res.name,
                race: res.race,
                class: res.class,
                level: res.level,
                hp_current: res.hp_current,
                hp_max: res.hp_max,
                ac: res.ac,
                speed: res.speed,
            };

            // Broadcast to all players in the session
            if let Some(session_id) = current_session {
                broadcast_to_session(session_state, *session_id, &ServerMessage::CharacterUpdated {
                    character: character_info.clone(),
                }).await;
            }

            Ok(ServerMessage::CharacterUpdated { character: character_info })
        }
        
        ClientMessage::UpdateInitiative { session_id, initiative_order } => {
            // Check if user is DM of this session's campaign
            let is_dm = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM sessions s 
                 INNER JOIN campaigns c ON s.campaign_id = c.id 
                 WHERE s.id = $1 AND c.dm_id = $2)"
            )
            .bind(session_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

            if !is_dm {
                return Err("Only the DM can update initiative".to_string());
            }

            // Get current game state
            let session = sqlx::query_as::<_, crate::models::Session>(
                "SELECT * FROM sessions WHERE id = $1"
            )
            .bind(session_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to fetch session: {}", e))?;

            // Parse current game state
            let mut game_state: crate::models::GameState = serde_json::from_value(session.game_state.clone())
                .unwrap_or_else(|_| crate::models::GameState {
                    initiative_order: Vec::new(),
                    current_turn: None,
                    round: 1,
                    combat_active: false,
                    conditions: Vec::new(),
                });

            // Update initiative order
            game_state.initiative_order = initiative_order.clone();
            game_state.combat_active = true;

            // Save updated game state
            let now = Utc::now();
            sqlx::query("UPDATE sessions SET game_state = $1, updated_at = $2 WHERE id = $3")
                .bind(serde_json::to_value(&game_state).unwrap())
                .bind(now)
                .bind(session_id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update game state: {}", e))?;

            // Broadcast to all players
            broadcast_to_session(session_state, session_id, &ServerMessage::InitiativeUpdated {
                session_id,
                initiative_order: initiative_order.clone(),
                current_turn: game_state.current_turn,
            }).await;

            Ok(ServerMessage::InitiativeUpdated {
                session_id,
                initiative_order,
                current_turn: game_state.current_turn,
            })
        }
        
        ClientMessage::NextTurn { session_id } => {
            // Check if user is DM of this session's campaign
            let is_dm = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM sessions s 
                 INNER JOIN campaigns c ON s.campaign_id = c.id 
                 WHERE s.id = $1 AND c.dm_id = $2)"
            )
            .bind(session_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

            if !is_dm {
                return Err("Only the DM can advance turns".to_string());
            }

            // Get current game state
            let session = sqlx::query_as::<_, crate::models::Session>(
                "SELECT * FROM sessions WHERE id = $1"
            )
            .bind(session_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to fetch session: {}", e))?;

            // Parse current game state
            let mut game_state: crate::models::GameState = serde_json::from_value(session.game_state.clone())
                .unwrap_or_else(|_| crate::models::GameState {
                    initiative_order: Vec::new(),
                    current_turn: None,
                    round: 1,
                    combat_active: false,
                    conditions: Vec::new(),
                });

            // Advance to next turn
            if !game_state.initiative_order.is_empty() {
                let current_index = game_state.initiative_order.iter()
                    .position(|entry| Some(entry.id) == game_state.current_turn)
                    .unwrap_or(0);
                
                let next_index = (current_index + 1) % game_state.initiative_order.len();
                game_state.current_turn = Some(game_state.initiative_order[next_index].id);
                
                // Increment round if we've gone through all entries
                if next_index == 0 {
                    game_state.round += 1;
                }
            }

            // Save updated game state
            let now = Utc::now();
            let round = game_state.round;
            sqlx::query("UPDATE sessions SET game_state = $1, updated_at = $2 WHERE id = $3")
                .bind(serde_json::to_value(&game_state).unwrap())
                .bind(now)
                .bind(session_id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update game state: {}", e))?;

            // Broadcast to all players
            if let Some(current_turn) = game_state.current_turn {
                broadcast_to_session(session_state, session_id, &ServerMessage::TurnChanged {
                    session_id,
                    current_turn,
                    round,
                }).await;

                Ok(ServerMessage::TurnChanged {
                    session_id,
                    current_turn,
                    round,
                })
            } else {
                Err("No initiative order set".to_string())
            }
        }
        
        ClientMessage::UpdateHP { character_id, hp_current, hp_max } => {
            // Check if user owns this character or is DM of the campaign
            let has_access = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM characters c 
                 INNER JOIN campaigns cam ON c.campaign_id = cam.id 
                 WHERE c.id = $1 AND (c.player_id = $2 OR cam.dm_id = $2))"
            )
            .bind(character_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

            if !has_access {
                return Err("Access denied to this character".to_string());
            }

            // Update character HP
            let now = Utc::now();
            let res = sqlx::query_as::<_, crate::models::Character>(
                "UPDATE characters SET hp_current = $1, hp_max = COALESCE($2, hp_max), updated_at = $3 WHERE id = $4 RETURNING *"
            )
            .bind(hp_current)
            .bind(hp_max)
            .bind(now)
            .bind(character_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to update character HP: {}", e))?;

            // Broadcast to all players in the session
            if let Some(session_id) = current_session {
                broadcast_to_session(session_state, *session_id, &ServerMessage::HPUpdated {
                    character_id,
                    hp_current: res.hp_current.unwrap_or(0),
                    hp_max: res.hp_max.unwrap_or(0),
                }).await;
            }

            Ok(ServerMessage::HPUpdated {
                character_id,
                hp_current: res.hp_current.unwrap_or(0),
                hp_max: res.hp_max.unwrap_or(0),
            })
        }
        
        ClientMessage::CreateEventLog { session_id, event_type, event_data } => {
            // Check if user has access to this session
            let has_access = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM sessions s 
                 INNER JOIN campaigns c ON s.campaign_id = c.id 
                 WHERE s.id = $1 AND (c.dm_id = $2 OR s.campaign_id IN (SELECT campaign_id FROM campaign_players WHERE player_id = $2)))"
            )
            .bind(session_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

            if !has_access {
                return Err("Access denied to this session".to_string());
            }

            // Create event log in database
            let event_id = Uuid::new_v4();
            let now = Utc::now();
            
            let event_log = sqlx::query_as::<_, crate::models::EventLog>(
                "INSERT INTO event_logs (id, session_id, event_type, event_data, created_by, created_at) 
                 VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
            )
            .bind(event_id)
            .bind(session_id)
            .bind(&event_type)
            .bind(&event_data)
            .bind(user_id)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to create event log: {}", e))?;

            // Broadcast to all players in the session
            let event_msg = ServerMessage::EventLogCreated {
                event_id: event_log.id,
                event_type: event_log.event_type,
                event_data: event_log.event_data,
                created_by: event_log.created_by.unwrap_or(user_id),
                created_at: event_log.created_at,
            };
            
            broadcast_to_session(session_state, session_id, &event_msg).await;

            Ok(event_msg)
        }
        
        ClientMessage::AIRequest { prompt, request_type, context: _ } => {
            // For now, return a mock AI response
            // TODO: Implement actual AI integration
            let response = match request_type.as_str() {
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

            // Log the AI request as an event if we're in a session
            if let Some(session_id) = current_session {
                let _ = sqlx::query(
                    "INSERT INTO event_logs (id, session_id, event_type, event_data, created_by, created_at) 
                     VALUES ($1, $2, $3, $4, $5, $6)"
                )
                .bind(Uuid::new_v4())
                .bind(*session_id)
                .bind("ai_request")
                .bind(serde_json::json!({
                    "prompt": prompt,
                    "request_type": request_type,
                    "response": response
                }))
                .bind(user_id)
                .bind(Utc::now())
                .execute(pool)
                .await;
            }

            let ai_response = ServerMessage::AIResponse {
                response,
                request_type,
                tokens_used: Some(150), // Mock value
                model: "gpt-4".to_string(),
            };

            // Broadcast AI response to all players in the session
            if let Some(session_id) = current_session {
                broadcast_to_session(session_state, *session_id, &ai_response).await;
            }

            Ok(ai_response)
        }
    }
}

async fn join_session(
    session_state: &SessionState,
    session_id: Uuid,
    user_id: Uuid,
    username: &str,
    is_dm: bool,
    pool: &PgPool,
) {
    // Fetch session info from database to get campaign_id
    let session_result = sqlx::query_as::<_, crate::models::Session>(
        "SELECT * FROM sessions WHERE id = $1"
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await;

    if let Ok(Some(session)) = session_result {
        let mut sessions = session_state.sessions.write().await;
        let session_info = sessions.entry(session_id).or_insert_with(|| SessionInfo {
            session_id,
            campaign_id: session.campaign_id,
            connections: Arc::new(RwLock::new(HashMap::new())),
        });
        
        // Use both session_id and campaign_id for logging
        println!("User {} joining session {} (campaign: {})", user_id, session_info.session_id, session_info.campaign_id);
        
        let mut connections = session_info.connections.write().await;
        connections.insert(user_id, ConnectionInfo {
            user_id,
            username: username.to_string(),
            is_dm,
        });
    }
}

async fn leave_session(session_state: &SessionState, session_id: Uuid, user_id: Uuid) {
    let mut sessions = session_state.sessions.write().await;
    if let Some(session_info) = sessions.get_mut(&session_id) {
        // Use session_id and campaign_id for logging
        println!("User {} leaving session {} (campaign: {})", user_id, session_info.session_id, session_info.campaign_id);
        
        let mut connections = session_info.connections.write().await;
        connections.remove(&user_id);
        
        // Check if session should be removed (no connections left)
        if connections.is_empty() {
            drop(connections); // Release the borrow before removing
            sessions.remove(&session_id);
        }
    }
}

async fn get_session_players(session_state: &SessionState, session_id: Uuid) -> Vec<PlayerInfo> {
    let sessions = session_state.sessions.read().await;
    if let Some(session_info) = sessions.get(&session_id) {
        // Use session_id and campaign_id for logging/debugging
        println!("Getting players for session {} (campaign: {})", session_info.session_id, session_info.campaign_id);
        
        let connections = session_info.connections.read().await;
        connections
            .values()
            .map(|conn| PlayerInfo {
                user_id: conn.user_id,
                username: conn.username.clone(),
                is_dm: conn.is_dm,
            })
            .collect()
    } else {
        Vec::new()
    }
}

async fn get_session_campaign_id(session_state: &SessionState, session_id: Uuid) -> Option<Uuid> {
    let sessions = session_state.sessions.read().await;
    sessions.get(&session_id).map(|session_info| session_info.campaign_id)
}

async fn broadcast_to_session(
    session_state: &SessionState,
    session_id: Uuid,
    message: &ServerMessage,
) {
    // This would broadcast to all connected clients in the session
    // For now, we'll just log the message with session info
    let sessions = session_state.sessions.read().await;
    if let Some(session_info) = sessions.get(&session_id) {
        println!("Broadcasting to session {} (campaign: {}): {:?}", 
                 session_info.session_id, session_info.campaign_id, message);
    } else {
        println!("Broadcasting to session {}: {:?}", session_id, message);
    }
}

// Dice rolling functionality
struct DiceRoll {
    total: i32,
    rolls: Vec<i32>,
}

fn roll_dice(dice: &str) -> Result<DiceRoll, String> {
    // Simple dice parser for common formats like "2d6+3", "1d20", etc.
    let parts: Vec<&str> = dice.split('+').collect();
    let dice_part = parts[0];
    let modifier = if parts.len() > 1 {
        parts[1].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    
    let dice_parts: Vec<&str> = dice_part.split('d').collect();
    if dice_parts.len() != 2 {
        return Err("Invalid dice format. Use format like '2d6+3'".to_string());
    }
    
    let count = dice_parts[0].parse::<i32>().map_err(|_| "Invalid dice count".to_string())?;
    let sides = dice_parts[1].parse::<i32>().map_err(|_| "Invalid dice sides".to_string())?;
    
    if count <= 0 || sides <= 0 {
        return Err("Dice count and sides must be positive".to_string());
    }
    
    let mut rolls = Vec::new();
    let mut total = 0;
    
    for _ in 0..count {
        let roll = (rand::random::<u32>() % sides as u32 + 1) as i32;
        rolls.push(roll);
        total += roll;
    }
    
    total += modifier;
    
    Ok(DiceRoll { total, rolls })
} 