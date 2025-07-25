# YoDA API Documentation

## Overview

YoDA (Your D&D Assistant) is a comprehensive D&D campaign management system with real-time features. This API provides endpoints for user authentication, campaign management, session handling, character management, and real-time game features.

**Base URL**: `http://localhost:3000`  
**Content-Type**: `application/json`

## Authentication

All protected endpoints require a JWT token in the Authorization header:
```
Authorization: Bearer <your-jwt-token>
```

## Error Responses

All endpoints may return the following error responses:
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Missing or invalid JWT token
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists
- `500 Internal Server Error` - Server error

## Endpoints

### Authentication

#### Register User
**POST** `/auth/register`

Create a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "username": "username",
  "password": "password123"
}
```

**Response:**
- `201 Created` - User registered successfully
- `409 Conflict` - Email or username already exists

#### Login
**POST** `/auth/login`

Authenticate user and receive JWT token.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Campaign Management

#### Create Campaign
**POST** `/campaigns`

Create a new campaign (DM only).

**Request Body:**
```json
{
  "name": "Lost Mine of Phandelver",
  "description": "A classic D&D adventure",
  "settings": {
    "theme": "dark",
    "difficulty": "medium"
  }
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "Lost Mine of Phandelver",
  "description": "A classic D&D adventure",
  "dm_id": "uuid",
  "settings": {
    "theme": "dark",
    "difficulty": "medium"
  },
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### List Campaigns
**GET** `/campaigns`

Get all campaigns the user has access to (as DM or player).

**Response:**
```json
[
  {
    "id": "uuid",
    "name": "Lost Mine of Phandelver",
    "description": "A classic D&D adventure",
    "dm_id": "uuid",
    "settings": {},
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Get Campaign
**GET** `/campaigns/{campaign_id}`

Get a specific campaign.

**Response:**
```json
{
  "id": "uuid",
  "name": "Lost Mine of Phandelver",
  "description": "A classic D&D adventure",
  "dm_id": "uuid",
  "settings": {},
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Update Campaign
**PUT** `/campaigns/{campaign_id}`

Update campaign details (DM only).

**Request Body:**
```json
{
  "name": "Updated Campaign Name",
  "description": "Updated description",
  "settings": {
    "theme": "light",
    "difficulty": "hard"
  }
}
```

**Response:** Same as Get Campaign

#### Delete Campaign
**DELETE** `/campaigns/{campaign_id}`

Delete a campaign (DM only).

**Response:**
- `200 OK` - Campaign deleted successfully

### Session Management

#### Create Session
**POST** `/sessions`

Create a new session for a campaign.

**Request Body:**
```json
{
  "campaign_id": "uuid",
  "name": "Session 1: Goblin Ambush",
  "description": "The party encounters goblins on the road"
}
```

**Response:**
```json
{
  "id": "uuid",
  "campaign_id": "uuid",
  "name": "Session 1: Goblin Ambush",
  "description": null,
  "status": "planned",
  "started_at": null,
  "ended_at": null,
  "game_state": {},
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### List Sessions
**GET** `/sessions`

Get all sessions the user has access to.

**Response:**
```json
[
  {
    "id": "uuid",
    "campaign_id": "uuid",
    "name": "Session 1: Goblin Ambush",
    "description": null,
    "status": "planned",
    "started_at": null,
    "ended_at": null,
    "game_state": {},
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Get Session
**GET** `/sessions/{session_id}`

Get a specific session.

**Response:** Same as Create Session response

#### Update Session
**PUT** `/sessions/{session_id}`

Update session details (DM only).

**Request Body:**
```json
{
  "name": "Updated Session Name",
  "status": "active",
  "game_state": {
    "initiative_order": [],
    "current_turn": null,
    "round": 1,
    "combat_active": false,
    "conditions": []
  }
}
```

**Response:** Same as Create Session response

#### Start Session
**POST** `/sessions/{session_id}/start`

Start a session (DM only).

**Response:** Same as Create Session response with `status: "active"` and `started_at` set

#### End Session
**POST** `/sessions/{session_id}/end`

End a session (DM only).

**Response:** Same as Create Session response with `status: "ended"` and `ended_at` set

### Character Management

#### Create Character
**POST** `/characters`

Create a new character.

**Request Body:**
```json
{
  "campaign_id": "uuid",
  "name": "Thorin Stonebeard",
  "race": "Dwarf",
  "class": "Fighter",
  "level": 5,
  "hp_max": 45,
  "ac": 18,
  "speed": 30,
  "stats": {
    "strength": 16,
    "dexterity": 14,
    "constitution": 16,
    "intelligence": 10,
    "wisdom": 12,
    "charisma": 8
  },
  "inventory": [],
  "spells": [],
  "features": []
}
```

**Response:**
```json
{
  "id": "uuid",
  "campaign_id": "uuid",
  "player_id": "uuid",
  "name": "Thorin Stonebeard",
  "race": "Dwarf",
  "class": "Fighter",
  "level": 5,
  "hp_current": 45,
  "hp_max": 45,
  "ac": 18,
  "speed": 30,
  "stats": {
    "strength": 16,
    "dexterity": 14,
    "constitution": 16,
    "intelligence": 10,
    "wisdom": 12,
    "charisma": 8
  },
  "inventory": [],
  "spells": [],
  "features": [],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### List Characters
**GET** `/characters`

Get all characters the user has access to.

**Response:**
```json
[
  {
    "id": "uuid",
    "campaign_id": "uuid",
    "player_id": "uuid",
    "name": "Thorin Stonebeard",
    "race": "Dwarf",
    "class": "Fighter",
    "level": 5,
    "hp_current": 45,
    "hp_max": 45,
    "ac": 18,
    "speed": 30,
    "stats": {},
    "inventory": [],
    "spells": [],
    "features": [],
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Get Character
**GET** `/characters/{character_id}`

Get a specific character.

**Response:** Same as Create Character response

#### Update Character
**PUT** `/characters/{character_id}`

Update character details (owner or DM only).

**Request Body:**
```json
{
  "name": "Updated Character Name",
  "level": 6,
  "hp_current": 50,
  "hp_max": 50,
  "ac": 19,
  "stats": {
    "strength": 18,
    "dexterity": 14,
    "constitution": 16,
    "intelligence": 10,
    "wisdom": 12,
    "charisma": 8
  }
}
```

**Response:** Same as Create Character response

#### Delete Character
**DELETE** `/characters/{character_id}`

Delete a character (owner or DM only).

**Response:**
- `200 OK` - Character deleted successfully

#### Update Character HP
**PUT** `/characters/{character_id}/hp`

Update character's current and maximum HP.

**Request Body:**
```json
{
  "hp_current": 35,
  "hp_max": 50
}
```

**Response:** Same as Create Character response

### Game State Management

#### Update Initiative
**PUT** `/initiative`

Update initiative order and combat state (DM only).

**Request Body:**
```json
{
  "session_id": "uuid",
  "initiative_order": [
    {
      "id": "uuid",
      "name": "Player 1",
      "initiative": 18,
      "is_player": true,
      "character_id": "uuid",
      "user_id": "uuid",
      "hp_current": 25,
      "hp_max": 25,
      "ac": 16
    },
    {
      "id": "uuid",
      "name": "Goblin 1",
      "initiative": 15,
      "is_player": false,
      "character_id": null,
      "user_id": null,
      "hp_current": 7,
      "hp_max": 7,
      "ac": 15
    }
  ],
  "current_turn": "uuid",
  "round": 1,
  "combat_active": true
}
```

**Response:**
- `200 OK` - Initiative updated successfully

### WebSocket Endpoints

#### WebSocket Connection
**GET** `/ws`

Establish a WebSocket connection for real-time features.

**Query Parameters:**
- `token` - JWT token for authentication

#### WebSocket Messages

**Client to Server Messages:**

1. **Join Session**
```json
{
  "type": "JoinSession",
  "data": {
    "session_id": "uuid"
  }
}
```

2. **Leave Session**
```json
{
  "type": "LeaveSession",
  "data": {
    "session_id": "uuid"
  }
}
```

3. **Dice Roll**
```json
{
  "type": "DiceRoll",
  "data": {
    "dice": "2d6+3",
    "reason": "Attack roll"
  }
}
```

4. **Chat Message**
```json
{
  "type": "ChatMessage",
  "data": {
    "message": "Hello, everyone!"
  }
}
```

5. **Update Character**
```json
{
  "type": "UpdateCharacter",
  "data": {
    "character_id": "uuid",
    "updates": {
      "hp_current": 35,
      "ac": 19
    }
  }
}
```

6. **Update Initiative**
```json
{
  "type": "UpdateInitiative",
  "data": {
    "session_id": "uuid",
    "initiative_order": [
      {
        "id": "uuid",
        "name": "Player 1",
        "initiative": 18,
        "is_player": true,
        "character_id": "uuid",
        "user_id": "uuid",
        "hp_current": 25,
        "hp_max": 25,
        "ac": 16
      }
    ]
  }
}
```

7. **Next Turn**
```json
{
  "type": "NextTurn",
  "data": {
    "session_id": "uuid"
  }
}
```

8. **Update HP**
```json
{
  "type": "UpdateHP",
  "data": {
    "character_id": "uuid",
    "hp_current": 35,
    "hp_max": 50
  }
}
```

**Server to Client Messages:**

1. **Session Joined**
```json
{
  "type": "SessionJoined",
  "data": {
    "session_id": "uuid",
    "players": [
      {
        "user_id": "uuid",
        "username": "player1",
        "is_dm": false
      }
    ]
  }
}
```

2. **Player Joined**
```json
{
  "type": "PlayerJoined",
  "data": {
    "player": {
      "user_id": "uuid",
      "username": "player2",
      "is_dm": false
    }
  }
}
```

3. **Player Left**
```json
{
  "type": "PlayerLeft",
  "data": {
    "player_id": "uuid"
  }
}
```

4. **Dice Rolled**
```json
{
  "type": "DiceRolled",
  "data": {
    "player_id": "uuid",
    "result": {
      "dice": "2d6+3",
      "result": 12,
      "rolls": [4, 5],
      "reason": "Attack roll"
    }
  }
}
```

5. **Chat Message**
```json
{
  "type": "ChatMessage",
  "data": {
    "player_id": "uuid",
    "message": "Hello, everyone!",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

6. **Character Updated**
```json
{
  "type": "CharacterUpdated",
  "data": {
    "character": {
      "id": "uuid",
      "name": "Thorin Stonebeard",
      "race": "Dwarf",
      "class": "Fighter",
      "level": 5,
      "hp_current": 35,
      "hp_max": 45,
      "ac": 19,
      "speed": 30
    }
  }
}
```

7. **Initiative Updated**
```json
{
  "type": "InitiativeUpdated",
  "data": {
    "session_id": "uuid",
    "initiative_order": [
      {
        "id": "uuid",
        "name": "Player 1",
        "initiative": 18,
        "is_player": true,
        "character_id": "uuid",
        "user_id": "uuid",
        "hp_current": 25,
        "hp_max": 25,
        "ac": 16
      }
    ],
    "current_turn": "uuid"
  }
}
```

8. **Turn Changed**
```json
{
  "type": "TurnChanged",
  "data": {
    "session_id": "uuid",
    "current_turn": "uuid",
    "round": 2
  }
}
```

9. **HP Updated**
```json
{
  "type": "HPUpdated",
  "data": {
    "character_id": "uuid",
    "hp_current": 35,
    "hp_max": 50
  }
}
```

10. **Error**
```json
{
  "type": "Error",
  "data": {
    "message": "Access denied to this character"
  }
}
```

## Data Models

### User
```json
{
  "id": "uuid",
  "email": "user@example.com",
  "username": "username",
  "password_hash": "hashed_password",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Campaign
```json
{
  "id": "uuid",
  "name": "Campaign Name",
  "description": "Campaign description",
  "dm_id": "uuid",
  "settings": {},
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Session
```json
{
  "id": "uuid",
  "campaign_id": "uuid",
  "name": "Session Name",
  "status": "planned|active|ended",
  "started_at": "2024-01-01T00:00:00Z",
  "ended_at": "2024-01-01T00:00:00Z",
  "game_state": {},
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Character
```json
{
  "id": "uuid",
  "campaign_id": "uuid",
  "player_id": "uuid",
  "name": "Character Name",
  "race": "Dwarf",
  "class": "Fighter",
  "level": 5,
  "hp_current": 45,
  "hp_max": 45,
  "ac": 18,
  "speed": 30,
  "stats": {},
  "inventory": [],
  "spells": [],
  "features": [],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Game State
```json
{
  "initiative_order": [
    {
      "id": "uuid",
      "name": "Player 1",
      "initiative": 18,
      "is_player": true,
      "character_id": "uuid",
      "user_id": "uuid",
      "hp_current": 25,
      "hp_max": 25,
      "ac": 16
    }
  ],
  "current_turn": "uuid",
  "round": 1,
  "combat_active": true,
  "conditions": [
    {
      "target_id": "uuid",
      "condition_type": "poisoned",
      "duration": 3,
      "description": "Poisoned by goblin arrow",
      "applied_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## Authentication & Authorization

### JWT Token Structure
```json
{
  "sub": "user-uuid",
  "exp": 1704067200
}
```

### Access Control Rules

1. **Campaign Access**
   - DM can access all campaigns they created
   - Players can access campaigns they're added to

2. **Session Access**
   - DM can access all sessions in their campaigns
   - Players can access sessions in campaigns they're part of

3. **Character Access**
   - Character owner can access their characters
   - DM can access all characters in their campaigns

4. **Game State Access**
   - Only DM can update initiative and game state
   - All participants can view game state

## Error Handling

### Common Error Responses

**400 Bad Request**
```json
{
  "error": "Invalid request data",
  "details": "Field 'email' is required"
}
```

**401 Unauthorized**
```json
{
  "error": "Invalid or missing JWT token"
}
```

**403 Forbidden**
```json
{
  "error": "Access denied to this campaign"
}
```

**404 Not Found**
```json
{
  "error": "Campaign not found"
}
```

**409 Conflict**
```json
{
  "error": "Email or username already exists"
}
```

**500 Internal Server Error**
```json
{
  "error": "Failed to create campaign"
}
```

## Rate Limiting

Currently, no rate limiting is implemented. Consider implementing rate limiting for production use.

## WebSocket Connection Limits

- Maximum concurrent connections per user: 5
- Maximum connections per session: 20
- Connection timeout: 30 minutes

## Development Setup

1. **Database Setup**
   ```bash
   # Start PostgreSQL and Redis
   docker-compose up -d
   
   # Run migrations
   sqlx migrate run --database-url "postgresql://dnd_user:dnd_pass@localhost/dnd_dm_assistant"
   ```

2. **Environment Variables**
   ```bash
   DATABASE_URL=postgresql://dnd_user:dnd_pass@localhost/dnd_dm_assistant
   REDIS_URL=redis://localhost:6379
   JWT_SECRET=your-secret-key-here
   ```

3. **Start Server**
   ```bash
   cargo run
   ```

## Testing

Run the test suite:
```bash
cargo test
```

All endpoints have comprehensive test coverage including:
- Happy path scenarios
- Error conditions
- Authorization checks
- Database integration
- WebSocket functionality

## Production Considerations

1. **Security**
   - Use strong JWT secrets
   - Implement rate limiting
   - Add CORS configuration
   - Use HTTPS in production

2. **Performance**
   - Add database connection pooling
   - Implement caching for frequently accessed data
   - Consider Redis for session storage

3. **Monitoring**
   - Add logging for all endpoints
   - Monitor WebSocket connections
   - Track API usage metrics

4. **Backup**
   - Regular database backups
   - Version control for migrations
   - Environment configuration management 