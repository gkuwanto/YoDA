# YoDA
Yet another Dungeonmaster Assistant

A real-time, AI-powered Dungeon Master assistant tool built with Rust backend (Axum + Socketioxide) and Vue 3 frontend. This application helps DMs manage D&D sessions with AI-enhanced features, real-time collaboration, and comprehensive campaign management.

## Project Overview

This is a web-based platform that provides:
- Real-time session management with WebSocket support
- AI/RAG-powered content generation and suggestions
- Campaign and character management
- Interactive battle maps and initiative tracking
- Knowledge base integration for rules and lore

## Tech Stack

### Backend
- **Language**: Rust
- **Web Framework**: Axum
- **Real-time**: Socketioxide (Socket.IO for Axum)
- **Database**: PostgreSQL 15+ with SQLx
- **Cache**: Redis (with redis-rs)
- **Vector DB**: Qdrant (for RAG functionality)
- **AI Integration**: OpenAI API / Anthropic Claude API
- **Authentication**: JWT with jsonwebtoken crate
- **Storage**: S3-compatible object storage (AWS S3 or MinIO)
- **Async Runtime**: Tokio

### Frontend
- **Framework**: Vue 3 with Composition API
- **Build Tool**: Vite
- **State Management**: Pinia
- **UI Framework**: Tailwind CSS + Headless UI
- **Real-time**: Socket.io-client
- **HTTP Client**: Axios
- **Rich Text Editor**: TipTap
- **Type Safety**: TypeScript

## Project Structure

```
dnd-dm-assistant/
├── backend/
│   ├── src/
│   │   ├── main.rs
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   └── settings.rs
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── campaign.rs
│   │   │   ├── session.rs
│   │   │   ├── character.rs
│   │   │   └── game_state.rs
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── campaign.rs
│   │   │   ├── session.rs
│   │   │   ├── ai.rs
│   │   │   ├── knowledge.rs
│   │   │   └── character.rs
│   │   ├── socket/
│   │   │   ├── mod.rs
│   │   │   ├── handlers.rs
│   │   │   └── events.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── ai_service.rs
│   │   │   ├── rag_service.rs
│   │   │   ├── auth_service.rs
│   │   │   └── game_service.rs
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   └── cors.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   └── migrations/
│   │   ├── utils/
│   │   │   ├── mod.rs
│   │   │   ├── errors.rs
│   │   │   └── extractors.rs
│   │   └── state.rs
│   ├── Cargo.toml
│   ├── .env.example
│   └── Dockerfile
├── frontend/
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   ├── Sidebar.vue
│   │   │   │   ├── Header.vue
│   │   │   │   └── MainLayout.vue
│   │   │   ├── session/
│   │   │   │   ├── EventLog.vue
│   │   │   │   ├── SessionControls.vue
│   │   │   │   └── PlayerList.vue
│   │   │   ├── ai/
│   │   │   │   ├── AIPanel.vue
│   │   │   │   ├── MonsterStats.vue
│   │   │   │   └── AIChat.vue
│   │   │   ├── character/
│   │   │   │   ├── CharacterCard.vue
│   │   │   │   ├── CharacterList.vue
│   │   │   │   └── CharacterSheet.vue
│   │   │   ├── tools/
│   │   │   │   ├── DiceRoller.vue
│   │   │   │   ├── InitiativeTracker.vue
│   │   │   │   └── QuickTools.vue
│   │   │   └── knowledge/
│   │   │       ├── KnowledgeBase.vue
│   │   │       └── DocumentViewer.vue
│   │   ├── views/
│   │   │   ├── Dashboard.vue
│   │   │   ├── Campaign.vue
│   │   │   ├── Session.vue
│   │   │   └── Login.vue
│   │   ├── stores/
│   │   │   ├── auth.ts
│   │   │   ├── campaign.ts
│   │   │   ├── session.ts
│   │   │   └── websocket.ts
│   │   ├── services/
│   │   │   ├── api.ts
│   │   │   ├── auth.service.ts
│   │   │   ├── campaign.service.ts
│   │   │   └── ai.service.ts
│   │   ├── composables/
│   │   │   ├── useWebSocket.ts
│   │   │   ├── useAI.ts
│   │   │   └── useDice.ts
│   │   ├── types/
│   │   │   └── index.ts
│   │   └── styles/
│   │       └── main.css
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── Dockerfile
├── docker-compose.yml
├── .gitignore
└── README.md
```

## Core Features to Implement

### Phase 1: Foundation (Weeks 1-2)
1. **Backend Setup**
   - Basic Axum server with health check endpoint
   - PostgreSQL connection with SQLx
   - Redis connection setup
   - JWT authentication middleware
   - Basic user registration/login endpoints
   - Socketioxide integration with Axum

2. **Frontend Setup**
   - Vue 3 project with Vite
   - Tailwind CSS configuration
   - Basic routing with Vue Router
   - Pinia store setup
   - Login/Register views
   - Socket.io-client setup

### Phase 2: Core Functionality (Weeks 3-4)
1. **Campaign Management**
   - CRUD operations for campaigns
   - Player invitation system
   - Campaign settings and metadata

2. **Session Management**
   - Create/join sessions
   - Basic session state management
   - Event logging system

3. **WebSocket Integration**
   - Socketioxide namespaces for sessions
   - Real-time event broadcasting
   - Room management for campaigns/sessions
   - Connection state handling

### Phase 3: Game Features (Weeks 5-6)
1. **Character Management**
   - Character sheet CRUD
   - HP and status tracking
   - Character assignment to players

2. **Game State**
   - Initiative tracker
   - Combat state management
   - Condition/effect tracking

3. **UI Components**
   - Event log display
   - Character cards
   - Initiative order display
   - Dice roller with animations

### Phase 4: AI Integration (Weeks 7-8)
1. **AI Service Setup**
   - OpenAI/Claude API integration
   - Prompt templates for different AI tasks
   - Response caching with Redis

2. **RAG Implementation**
   - Qdrant vector database setup
   - Document ingestion pipeline
   - Semantic search functionality

3. **AI Features**
   - NPC generation
   - Location/quest generation
   - Rules clarification
   - Combat suggestions

### Phase 5: Advanced Features (Weeks 9-10)
1. **Knowledge Base**
   - Document upload and processing
   - Custom campaign lore management
   - Quick reference system

2. **Battle Maps**
   - Basic map display
   - Token management
   - Fog of war

3. **Advanced Tools**
   - Automated combat resolution
   - Session recording/replay
   - Export functionality

## Backend Implementation Details

### Cargo.toml Dependencies
```toml
[package]
name = "dnd-dm-assistant"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["macros", "ws"] }
axum-extra = { version = "0.9", features = ["typed-header"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# WebSocket
socketioxide = { version = "0.10", features = ["state"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Authentication
jsonwebtoken = "9"
argon2 = "0.5"

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
thiserror = "1.0"

# Redis
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# AI/HTTP
reqwest = { version = "0.11", features = ["json"] }

# Vector DB
qdrant-client = "1.7"
```

### Main.rs Structure
```rust
use axum::{
    routing::{get, post},
    Router,
};
use socketioxide::SocketIo;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")?;
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    
    // Redis connection
    let redis_client = redis::Client::open(std::env::var("REDIS_URL")?)?;
    let redis_conn = redis_client.get_connection_manager().await?;
    
    // Initialize Socket.IO
    let (socket_layer, io) = SocketIo::new_layer();
    
    // Setup socket handlers
    io.ns("/", socket::handlers::on_connect);
    
    // Shared application state
    let app_state = Arc::new(AppState {
        db: db_pool,
        redis: redis_conn,
        io: io.clone(),
    });
    
    // Build application
    let app = Router::new()
        // API routes
        .nest("/api", api_routes(app_state.clone()))
        // Health check
        .route("/health", get(health_check))
        // Socket.IO layer
        .layer(socket_layer)
        // CORS
        .layer(CorsLayer::permissive());
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn api_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Auth routes
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        
        // Campaign routes
        .route("/campaigns", get(handlers::campaign::list))
        .route("/campaigns", post(handlers::campaign::create))
        .route("/campaigns/:id", get(handlers::campaign::get))
        .route("/campaigns/:id", put(handlers::campaign::update))
        .route("/campaigns/:id", delete(handlers::campaign::delete))
        
        // Session routes
        .route("/sessions", get(handlers::session::list))
        .route("/sessions", post(handlers::session::create))
        .route("/sessions/:id", get(handlers::session::get))
        .route("/sessions/:id/start", post(handlers::session::start))
        .route("/sessions/:id/end", post(handlers::session::end))
        
        // AI routes
        .route("/ai/generate/npc", post(handlers::ai::generate_npc))
        .route("/ai/generate/location", post(handlers::ai::generate_location))
        .route("/ai/chat", post(handlers::ai::chat))
        
        .with_state(state)
}
```

### Socket Event Structure
```rust
// socket/events.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientEvent {
    JoinSession { session_id: Uuid },
    LeaveSession { session_id: Uuid },
    DiceRoll { dice: String, reason: Option<String> },
    UpdateInitiative { order: Vec<InitiativeEntry> },
    CharacterUpdate { character_id: Uuid, updates: CharacterUpdate },
    ChatMessage { message: String },
    AIRequest { prompt: String, context: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerEvent {
    SessionJoined { session_id: Uuid, players: Vec<Player> },
    PlayerJoined { player: Player },
    PlayerLeft { player_id: Uuid },
    DiceRolled { player_id: Uuid, result: DiceResult },
    InitiativeUpdated { order: Vec<InitiativeEntry> },
    CharacterUpdated { character: Character },
    ChatMessage { player_id: Uuid, message: String, timestamp: DateTime<Utc> },
    AISuggestion { suggestion: String },
    Error { message: String },
}
```

## API Endpoints

### Authentication
```rust
POST   /api/auth/register     // User registration
POST   /api/auth/login        // User login  
POST   /api/auth/refresh      // Refresh JWT token
POST   /api/auth/logout       // Logout user
GET    /api/auth/me           // Get current user
```

### Campaigns
```rust
GET    /api/campaigns         // List user's campaigns
POST   /api/campaigns         // Create new campaign
GET    /api/campaigns/:id     // Get campaign details
PUT    /api/campaigns/:id     // Update campaign
DELETE /api/campaigns/:id     // Delete campaign
POST   /api/campaigns/:id/invite   // Invite players
GET    /api/campaigns/:id/players  // Get campaign players
```

### Sessions
```rust
GET    /api/sessions          // List sessions
POST   /api/sessions          // Create session
GET    /api/sessions/:id      // Get session details
PUT    /api/sessions/:id      // Update session
POST   /api/sessions/:id/start    // Start session
POST   /api/sessions/:id/end      // End session
GET    /api/sessions/:id/events   // Get event log
POST   /api/sessions/:id/events   // Add event
```

### Characters
```rust
GET    /api/characters        // List characters
POST   /api/characters        // Create character
GET    /api/characters/:id    // Get character details
PUT    /api/characters/:id    // Update character
DELETE /api/characters/:id    // Delete character
PUT    /api/characters/:id/hp // Update HP
```

### AI Features
```rust
POST   /api/ai/generate/npc       // Generate NPC
POST   /api/ai/generate/location  // Generate location
POST   /api/ai/generate/quest     // Generate quest
POST   /api/ai/generate/encounter // Generate encounter
POST   /api/ai/enhance/description // Enhance descriptions
POST   /api/ai/chat               // AI chat assistance
```

### Knowledge Base
```rust
GET    /api/knowledge/search      // Search knowledge base
POST   /api/knowledge/documents   // Upload document
GET    /api/knowledge/documents/:id // Get document
DELETE /api/knowledge/documents/:id // Delete document
```

## Socket.IO Events

### Client → Server
```javascript
// Join a session
socket.emit('session:join', { session_id: 'uuid' })

// Roll dice
socket.emit('dice:roll', { dice: '2d20+5', reason: 'Attack roll' })

// Update initiative
socket.emit('initiative:update', { order: [...] })

// Update character
socket.emit('character:update', { character_id: 'uuid', hp_current: 15 })

// Send chat message
socket.emit('chat:message', { message: 'Hello!' })

// Request AI assistance
socket.emit('ai:request', { prompt: 'Generate a tavern description' })
```

### Server → Client
```javascript
// Player joined session
socket.on('player:joined', { player: {...} })

// Dice roll result
socket.on('dice:rolled', { player_id: 'uuid', result: {...} })

// Initiative updated
socket.on('initiative:updated', { order: [...] })

// Character updated
socket.on('character:updated', { character: {...} })

// Chat message received
socket.on('chat:message', { player_id: 'uuid', message: '...', timestamp: '...' })

// AI suggestion
socket.on('ai:suggestion', { suggestion: '...' })
```

## Database Schema

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Campaigns table
CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    dm_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Campaign players junction table
CREATE TABLE campaign_players (
    campaign_id UUID REFERENCES campaigns(id) ON DELETE CASCADE,
    player_id UUID REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (campaign_id, player_id)
);

-- Sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'planned',
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    game_state JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Characters table
CREATE TABLE characters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    player_id UUID REFERENCES users(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    race VARCHAR(100),
    class VARCHAR(100),
    level INTEGER DEFAULT 1,
    hp_current INTEGER,
    hp_max INTEGER,
    ac INTEGER,
    stats JSONB DEFAULT '{}',
    inventory JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Event logs table
CREATE TABLE event_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB NOT NULL,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Knowledge documents table
CREATE TABLE knowledge_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID REFERENCES campaigns(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    document_type VARCHAR(50),
    tags TEXT[],
    metadata JSONB DEFAULT '{}',
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_sessions_campaign_id ON sessions(campaign_id);
CREATE INDEX idx_characters_campaign_id ON characters(campaign_id);
CREATE INDEX idx_event_logs_session_id ON event_logs(session_id);
CREATE INDEX idx_knowledge_documents_campaign_id ON knowledge_documents(campaign_id);
CREATE INDEX idx_knowledge_documents_tags ON knowledge_documents USING GIN(tags);
```

## Frontend Implementation Notes

### Pinia Store Structure
```typescript
// stores/session.ts
export const useSessionStore = defineStore('session', () => {
  const currentSession = ref<Session | null>(null)
  const eventLog = ref<EventLogEntry[]>([])
  const players = ref<Player[]>([])
  const characters = ref<Character[]>([])
  
  // Socket.IO connection
  const socket = ref<Socket | null>(null)
  
  // Actions
  const joinSession = async (sessionId: string) => {
    // Connect to socket and join session
  }
  
  const rollDice = async (dice: string, reason?: string) => {
    socket.value?.emit('dice:roll', { dice, reason })
  }
  
  return {
    currentSession,
    eventLog,
    players,
    characters,
    joinSession,
    rollDice
  }
})
```

### Socket.IO Composable
```typescript
// composables/useWebSocket.ts
export const useWebSocket = () => {
  const socket = ref<Socket | null>(null)
  const connected = ref(false)
  
  const connect = (sessionId: string) => {
    socket.value = io('http://localhost:3000', {
      auth: {
        token: localStorage.getItem('auth_token')
      }
    })
    
    socket.value.on('connect', () => {
      connected.value = true
      socket.value?.emit('session:join', { session_id: sessionId })
    })
    
    // Setup event handlers
    setupEventHandlers(socket.value)
  }
  
  return {
    socket: readonly(socket),
    connected: readonly(connected),
    connect
  }
}
```

## Development Setup

### Prerequisites
- Rust 1.75+
- Node.js 18+
- PostgreSQL 15+
- Redis 7+
- Qdrant (optional, for RAG features)

### Backend Setup
```bash
cd backend
cp .env.example .env
# Edit .env with your configuration

# Install dependencies and run
cargo build
cargo run
```

### Frontend Setup
```bash
cd frontend
npm install
npm run dev
```

### Docker Setup
```bash
docker-compose up -d
```

## Environment Variables

### Backend (.env)
```env
# Server
PORT=3000
RUST_LOG=debug

# Database
DATABASE_URL=postgresql://user:password@localhost/dnd_dm_assistant

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key
JWT_EXPIRY=7d

# AI Services
OPENAI_API_KEY=your-openai-key
ANTHROPIC_API_KEY=your-anthropic-key

# Vector DB
QDRANT_URL=http://localhost:6333

# S3/MinIO
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=your-access-key
S3_SECRET_KEY=your-secret-key
S3_BUCKET=dnd-assets
```

### Frontend (.env)
```env
VITE_API_URL=http://localhost:3000
VITE_WS_URL=http://localhost:3000
```

## Key Implementation Priorities

1. **Start with Core Auth & Session Management** - Get basic user flows working
2. **Implement Real-time Features Early** - Socket.IO integration is critical
3. **Build Modular AI Service** - Easy to swap between OpenAI/Claude/Local models
4. **Focus on DM Experience** - UI should be optimized for quick actions during gameplay
5. **Performance Matters** - Cache frequently accessed data (monster stats, spell descriptions)
6. **Mobile Responsive** - DMs often use tablets during sessions

## Testing Strategy

- Unit tests for business logic (Rust: `cargo test`)
- Integration tests for API endpoints
- E2E tests for critical user flows (Cypress/Playwright)
- Load testing for Socket.IO connections
- AI prompt testing framework