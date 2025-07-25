use axum::{routing::{get, post, put, delete}, Router, http::StatusCode, response::IntoResponse, extract::Extension};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
mod models;
mod handlers;
mod middleware;
mod socket;
use middleware::{jwt_auth, AuthUser};
use socket::{SessionState, ws_handler};

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn protected_route(Extension(user): Extension<AuthUser>) -> impl IntoResponse {
    (StatusCode::OK, format!("User ID: {}", user.0))
}

async fn api_docs() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>YoDA API Documentation</title>
    <style>
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }
        h2 { color: #34495e; margin-top: 30px; border-left: 4px solid #3498db; padding-left: 15px; }
        h3 { color: #2c3e50; margin-top: 25px; }
        .endpoint { background: #f8f9fa; border: 1px solid #e9ecef; border-radius: 5px; padding: 15px; margin: 10px 0; }
        .method { display: inline-block; padding: 4px 8px; border-radius: 3px; color: white; font-weight: bold; font-size: 12px; }
        .get { background: #28a745; }
        .post { background: #007bff; }
        .put { background: #ffc107; color: #212529; }
        .delete { background: #dc3545; }
        .url { font-family: 'Courier New', monospace; background: #e9ecef; padding: 5px; border-radius: 3px; }
        .description { margin: 10px 0; color: #6c757d; }
        .auth { background: #fff3cd; border: 1px solid #ffeaa7; padding: 10px; border-radius: 5px; margin: 10px 0; }
        .example { background: #f8f9fa; border-left: 4px solid #007bff; padding: 10px; margin: 10px 0; font-family: 'Courier New', monospace; }
        .websocket { background: #e3f2fd; border: 1px solid #bbdefb; padding: 15px; border-radius: 5px; margin: 10px 0; }
        .status { display: inline-block; padding: 2px 6px; border-radius: 3px; font-size: 11px; font-weight: bold; }
        .success { background: #d4edda; color: #155724; }
        .error { background: #f8d7da; color: #721c24; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎲 YoDA API Documentation</h1>
        <p><strong>Yet another Dungeonmaster Assistant</strong> - Real-time D&D session management API</p>
        
        <h2>🔐 Authentication</h2>
        <div class="auth">
            <strong>JWT Authentication Required:</strong> Most endpoints require a valid JWT token in the Authorization header:<br>
            <code>Authorization: Bearer &lt;your-jwt-token&gt;</code>
        </div>

        <h2>📊 Health Check</h2>
        <div class="endpoint">
            <span class="method get">GET</span>
            <span class="url">/health</span>
            <div class="description">Check if the server is running</div>
            <div class="example">
                <strong>Response:</strong> 200 OK<br>
                <code>OK</code>
            </div>
        </div>

        <h2>🔑 Authentication Endpoints</h2>
        
        <div class="endpoint">
            <span class="method post">POST</span>
            <span class="url">/auth/register</span>
            <div class="description">Register a new user account</div>
            <div class="example">
                <strong>Request Body:</strong><br>
                <code>{
  "username": "dungeonmaster",
  "email": "dm@example.com",
  "password": "securepassword"
}</code><br><br>
                <strong>Response:</strong> <span class="status success">201 Created</span><br>
                <code>{
  "user": {
    "id": "uuid",
    "username": "dungeonmaster",
    "email": "dm@example.com"
  },
  "token": "jwt-token"
}</code>
            </div>
        </div>

        <div class="endpoint">
            <span class="method post">POST</span>
            <span class="url">/auth/login</span>
            <div class="description">Login with username/email and password</div>
            <div class="example">
                <strong>Request Body:</strong><br>
                <code>{
  "username": "dungeonmaster",
  "password": "securepassword"
}</code><br><br>
                <strong>Response:</strong> <span class="status success">200 OK</span><br>
                <code>{
  "user": {
    "id": "uuid",
    "username": "dungeonmaster",
    "email": "dm@example.com"
  },
  "token": "jwt-token"
}</code>
            </div>
        </div>

        <h2>🏰 Campaign Management</h2>
        
        <div class="endpoint">
            <span class="method get">GET</span>
            <span class="url">/campaigns</span>
            <div class="description">List all campaigns for the authenticated user</div>
            <div class="example">
                <strong>Response:</strong> <span class="status success">200 OK</span><br>
                <code>[
  {
    "id": "uuid",
    "name": "Lost Mine of Phandelver",
    "description": "A classic D&D adventure",
    "dm_id": "uuid",
    "created_at": "2025-07-25T23:41:33Z"
  }
]</code>
            </div>
        </div>

        <div class="endpoint">
            <span class="method post">POST</span>
            <span class="url">/campaigns</span>
            <div class="description">Create a new campaign</div>
            <div class="example">
                <strong>Request Body:</strong><br>
                <code>{
  "name": "Lost Mine of Phandelver",
  "description": "A classic D&D adventure"
}</code><br><br>
                <strong>Response:</strong> <span class="status success">201 Created</span>
            </div>
        </div>

        <div class="endpoint">
            <span class="method get">GET</span>
            <span class="url">/campaigns/:id</span>
            <div class="description">Get campaign details</div>
        </div>

        <div class="endpoint">
            <span class="method put">PUT</span>
            <span class="url">/campaigns/:id</span>
            <div class="description">Update campaign details</div>
        </div>

        <div class="endpoint">
            <span class="method delete">DELETE</span>
            <span class="url">/campaigns/:id</span>
            <div class="description">Delete a campaign</div>
        </div>

        <h2>🎮 Session Management</h2>
        
        <div class="endpoint">
            <span class="method get">GET</span>
            <span class="url">/sessions</span>
            <div class="description">List all sessions for campaigns the user has access to</div>
        </div>

        <div class="endpoint">
            <span class="method post">POST</span>
            <span class="url">/sessions</span>
            <div class="description">Create a new session</div>
            <div class="example">
                <strong>Request Body:</strong><br>
                <code>{
  "campaign_id": "uuid",
  "name": "Session 1: Goblin Ambush",
  "description": "The party encounters goblins on the road"
}</code>
            </div>
        </div>

        <div class="endpoint">
            <span class="method get">GET</span>
            <span class="url">/sessions/:id</span>
            <div class="description">Get session details</div>
        </div>

        <div class="endpoint">
            <span class="method put">PUT</span>
            <span class="url">/sessions/:id</span>
            <div class="description">Update session details</div>
        </div>

        <div class="endpoint">
            <span class="method post">POST</span>
            <span class="url">/sessions/:id/start</span>
            <div class="description">Start a session (DM only)</div>
        </div>

        <div class="endpoint">
            <span class="method post">POST</span>
            <span class="url">/sessions/:id/end</span>
            <div class="description">End a session (DM only)</div>
        </div>

        <h2>👤 Character Management</h2>
        
        <div class="endpoint">
            <span class="method get">GET</span>
            <span class="url">/characters</span>
            <div class="description">List characters for campaigns the user has access to</div>
        </div>

        <div class="endpoint">
            <span class="method post">POST</span>
            <span class="url">/characters</span>
            <div class="description">Create a new character</div>
            <div class="example">
                <strong>Request Body:</strong><br>
                <code>{
  "campaign_id": "uuid",
  "name": "Gandalf the Grey",
  "race": "Human",
  "class": "Wizard",
  "level": 5,
  "hp_current": 25,
  "hp_max": 25,
  "ac": 15,
  "speed": 30
}</code>
            </div>
        </div>

        <div class="endpoint">
            <span class="method get">GET</span>
            <span class="url">/characters/:id</span>
            <div class="description">Get character details</div>
        </div>

        <div class="endpoint">
            <span class="method put">PUT</span>
            <span class="url">/characters/:id</span>
            <div class="description">Update character details</div>
        </div>

        <div class="endpoint">
            <span class="method delete">DELETE</span>
            <span class="url">/characters/:id</span>
            <div class="description">Delete a character</div>
        </div>

        <div class="endpoint">
            <span class="method put">PUT</span>
            <span class="url">/characters/:id/hp</span>
            <div class="description">Update character HP</div>
            <div class="example">
                <strong>Request Body:</strong><br>
                <code>{
  "hp_current": 20,
  "hp_max": 25
}</code>
            </div>
        </div>

        <h2>⚔️ Game State Management</h2>
        
        <div class="endpoint">
            <span class="method put">PUT</span>
            <span class="url">/initiative</span>
            <div class="description">Update initiative order (DM only)</div>
            <div class="example">
                <strong>Request Body:</strong><br>
                <code>{
  "session_id": "uuid",
  "initiative_order": [
    {
      "id": "uuid",
      "name": "Goblin 1",
      "initiative": 18,
      "is_player": false,
      "hp_current": 7,
      "hp_max": 7,
      "ac": 15
    }
  ]
}</code>
            </div>
        </div>

        <h2>🔌 WebSocket Real-time Communication</h2>
        
        <div class="websocket">
            <h3>WebSocket Endpoint</h3>
            <div class="url">ws://localhost:3000/ws</div>
            <div class="description">Real-time communication for game sessions</div>
            
            <h4>Connection</h4>
            <p>Connect to the WebSocket endpoint with JWT authentication:</p>
            <div class="example">
                <code>const socket = new WebSocket('ws://localhost:3000/ws');</code>
            </div>

            <h4>Client Messages</h4>
            <ul>
                <li><strong>JoinSession:</strong> Join a game session</li>
                <li><strong>LeaveSession:</strong> Leave a game session</li>
                <li><strong>DiceRoll:</strong> Roll dice with optional reason</li>
                <li><strong>ChatMessage:</strong> Send chat message</li>
                <li><strong>UpdateGameState:</strong> Update game state (DM only)</li>
                <li><strong>UpdateCharacter:</strong> Update character details</li>
                <li><strong>UpdateInitiative:</strong> Update initiative order (DM only)</li>
                <li><strong>NextTurn:</strong> Advance to next turn (DM only)</li>
                <li><strong>UpdateHP:</strong> Update character HP</li>
            </ul>

            <h4>Server Messages</h4>
            <ul>
                <li><strong>SessionJoined:</strong> Confirmation of session join</li>
                <li><strong>PlayerJoined:</strong> New player joined session</li>
                <li><strong>PlayerLeft:</strong> Player left session</li>
                <li><strong>DiceRolled:</strong> Dice roll result</li>
                <li><strong>ChatMessage:</strong> Chat message from player</li>
                <li><strong>GameStateUpdated:</strong> Game state changed</li>
                <li><strong>CharacterUpdated:</strong> Character details updated</li>
                <li><strong>InitiativeUpdated:</strong> Initiative order updated</li>
                <li><strong>TurnChanged:</strong> Turn advanced</li>
                <li><strong>HPUpdated:</strong> Character HP updated</li>
                <li><strong>Error:</strong> Error message</li>
            </ul>
        </div>

        <h2>📝 Error Responses</h2>
        <div class="endpoint">
            <div class="description">All endpoints may return these error responses:</div>
            <div class="example">
                <strong>401 Unauthorized:</strong> Invalid or missing JWT token<br>
                <strong>403 Forbidden:</strong> Insufficient permissions<br>
                <strong>404 Not Found:</strong> Resource not found<br>
                <strong>422 Unprocessable Entity:</strong> Invalid request data<br>
                <strong>500 Internal Server Error:</strong> Server error
            </div>
        </div>

        <h2>🔧 Development</h2>
        <p>For development and testing, you can use tools like:</p>
        <ul>
            <li><strong>Postman</strong> or <strong>Insomnia</strong> for API testing</li>
            <li><strong>WebSocket Client</strong> browser extension for WebSocket testing</li>
            <li><strong>curl</strong> for command-line testing</li>
        </ul>

        <div style="margin-top: 40px; padding: 20px; background: #f8f9fa; border-radius: 5px; text-align: center;">
            <p><strong>🎲 YoDA Backend API v1.0</strong></p>
            <p>Built with Rust, Axum, PostgreSQL, and Redis</p>
        </div>
    </div>
</body>
</html>
    "#;
    
    (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "text/html")], html)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Set up PostgreSQL connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    println!("Connected to Postgres");

    // Redis connection will be set up when needed for caching

    // Create shared session state for WebSocket connections
    let session_state = SessionState {
        sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // Build our application with a health check route
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/auth/register", axum::routing::post(handlers::register))
        .route("/auth/login", axum::routing::post(handlers::login))
        .route("/protected", get(protected_route).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Campaign routes (protected)
        .route("/campaigns", get(handlers::list_campaigns).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns", post(handlers::create_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns/:id", get(handlers::get_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns/:id", put(handlers::update_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/campaigns/:id", delete(handlers::delete_campaign).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Session routes (protected)
        .route("/sessions", get(handlers::list_sessions).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions", post(handlers::create_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id", get(handlers::get_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id", put(handlers::update_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id/start", post(handlers::start_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/sessions/:id/end", post(handlers::end_session).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Character routes (protected)
        .route("/characters", get(handlers::list_characters).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters", post(handlers::create_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id", get(handlers::get_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id", put(handlers::update_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id", delete(handlers::delete_character).route_layer(axum::middleware::from_fn(jwt_auth)))
        .route("/characters/:id/hp", put(handlers::update_character_hp).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Game state routes (protected)
        .route("/initiative", put(handlers::update_initiative).route_layer(axum::middleware::from_fn(jwt_auth)))
        // Health check endpoint
        .route("/health", get(health_check))
        .route("/docs", get(api_docs))
        .layer(Extension(pool))
        .layer(Extension(session_state));

    println!("🚀 YoDA Backend Server starting on http://0.0.0.0:3000");
    println!("📚 API Documentation available at http://localhost:3000/docs");
    println!("🔌 WebSocket endpoint available at ws://localhost:3000/ws");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}