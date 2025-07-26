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

### Event Logs

#### Create Event Log
**POST** `/event-logs`

Create a new event log entry for a session.

**Request Body:**
```json
{
  "session_id": "uuid",
  "event_type": "combat_start",
  "event_data": {
    "location": "Castle Courtyard",
    "enemies": ["Goblin 1", "Goblin 2"],
    "initiative_order": [18, 15, 12, 8]
  }
}
```

**Response:**
```json
{
  "id": "uuid",
  "session_id": "uuid",
  "event_type": "combat_start",
  "event_data": {
    "location": "Castle Courtyard",
    "enemies": ["Goblin 1", "Goblin 2"],
    "initiative_order": [18, 15, 12, 8]
  },
  "created_by": "uuid",
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### List Event Logs
**GET** `/sessions/:session_id/event-logs`

Get all event logs for a specific session.

**Response:**
```json
[
  {
    "id": "uuid",
    "session_id": "uuid",
    "event_type": "session_start",
    "event_data": {
      "players": ["John", "Sarah", "Mike"]
    },
    "created_by": "uuid",
    "created_at": "2024-01-01T00:00:00Z"
  },
  {
    "id": "uuid",
    "session_id": "uuid",
    "event_type": "combat_start",
    "event_data": {
      "location": "Castle Courtyard",
      "enemies": ["Goblin 1", "Goblin 2"]
    },
    "created_by": "uuid",
    "created_at": "2024-01-01T00:05:00Z"
  }
]
```

#### Get Event Log
**GET** `/event-logs/:event_id`

Get a specific event log entry.

**Response:**
```json
{
  "id": "uuid",
  "session_id": "uuid",
  "event_type": "dice_roll",
  "event_data": {
    "player": "John",
    "dice": "2d20+5",
    "result": 18,
    "reason": "Attack roll"
  },
  "created_by": "uuid",
  "created_at": "2024-01-01T00:10:00Z"
}
```

### AI Integration

#### Generate AI Content
**POST** `/ai/generate`

Generate AI-powered content for D&D sessions.

**Request Body:**
```json
{
  "prompt": "Create a mysterious NPC for a tavern scene",
  "context": "The party is in a seedy tavern in Waterdeep",
  "session_id": "uuid",
  "request_type": "npc"
}
```

**Request Types:**
- `npc` - Generate NPC descriptions and stats
- `location` - Generate location descriptions
- `encounter` - Generate encounter scenarios
- `description` - Enhance existing descriptions
- `chat` - General DM assistance

**Response:**
```json
{
  "response": "Generated NPC: A mysterious figure with a weathered cloak and piercing eyes. They seem to know more than they let on...",
  "tokens_used": 150,
  "model": "gpt-4"
}
```

## WebSocket Events

### Client → Server Events

#### Join Session
```json
{
  "type": "JoinSession",
  "data": {
    "session_id": "uuid"
  }
}
```

#### Create Event Log
```json
{
  "type": "CreateEventLog",
  "data": {
    "session_id": "uuid",
    "event_type": "player_action",
    "event_data": {
      "player": "John",
      "action": "casts_fireball",
      "target": "Goblin 1",
      "damage": 28
    }
  }
}
```

#### AI Request
```json
{
  "type": "AIRequest",
  "data": {
    "prompt": "What should happen next in this encounter?",
    "request_type": "chat",
    "context": "The party is fighting goblins in a forest"
  }
}
```

### Server → Client Events

#### Event Log Created
```json
{
  "type": "EventLogCreated",
  "data": {
    "event_id": "uuid",
    "event_type": "player_action",
    "event_data": {
      "player": "John",
      "action": "casts_fireball",
      "target": "Goblin 1",
      "damage": 28
    },
    "created_by": "uuid",
    "created_at": "2024-01-01T00:15:00Z"
  }
}
```

#### AI Response
```json
{
  "type": "AIResponse",
  "data": {
    "response": "AI Assistant: The goblins seem nervous and might be open to negotiation.",
    "request_type": "chat",
    "tokens_used": 150,
    "model": "gpt-4"
  }
}
```

## Event Log Types

Common event types for session tracking:

- `session_start` - Session begins
- `session_end` - Session ends
- `player_join` - Player joins session
- `player_leave` - Player leaves session
- `combat_start` - Combat encounter begins
- `combat_end` - Combat encounter ends
- `dice_roll` - Player rolls dice
- `character_update` - Character stats updated
- `initiative_update` - Initiative order changed
- `turn_change` - Turn advances
- `hp_update` - Character HP changed
- `ai_request` - AI assistance requested
- `chat_message` - Player sends message
- `dm_note` - DM adds private note
- `player_action` - Player performs action 