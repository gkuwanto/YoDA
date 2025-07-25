# YoDA Docker Setup

This document provides instructions for running YoDA (Your D&D Assistant) using Docker Compose.

## Prerequisites

- Docker
- Docker Compose

## Quick Start

### Production Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd YoDA
   ```

2. **Start all services**
   ```bash
   docker-compose up -d
   ```

3. **Check service status**
   ```bash
   docker-compose ps
   ```

4. **View logs**
   ```bash
   # All services
   docker-compose logs -f
   
   # Specific service
   docker-compose logs -f backend
   ```

### Development Setup

1. **Start development environment**
   ```bash
   docker-compose -f docker-compose.dev.yml up -d
   ```

2. **Run with hot reloading**
   ```bash
   docker-compose -f docker-compose.dev.yml up
   ```

## Services

### Backend API
- **Port**: 3000
- **Health Check**: `http://localhost:3000/health`
- **API Documentation**: Available at `http://localhost:3000/docs`
- **WebSocket**: `ws://localhost:3000/ws`

### PostgreSQL Database
- **Port**: 5432
- **Database**: `dnd_dm_assistant`
- **User**: `dnd_user`
- **Password**: `dnd_pass`

### Redis Cache
- **Port**: 6379
- **Purpose**: Session storage and caching

## Environment Variables

### Backend Service
```bash
DATABASE_URL=postgresql://dnd_user:dnd_pass@postgres:5432/dnd_dm_assistant
REDIS_URL=redis://redis:6379
JWT_SECRET=your-secret-key-here-change-in-production
RUST_LOG=info
```

### Development Environment
```bash
DATABASE_URL=postgresql://dnd_user:dnd_pass@postgres:5432/dnd_dm_assistant
REDIS_URL=redis://redis:6379
JWT_SECRET=your-secret-key-here-change-in-production
RUST_LOG=debug
RUST_BACKTRACE=1
```

## Database Migrations

Migrations are automatically applied when the PostgreSQL container starts. The migrations are located in `backend/migrations/`.

### Manual Migration (if needed)
```bash
# Connect to the backend container
docker-compose exec backend bash

# Run migrations
sqlx migrate run --database-url "postgresql://dnd_user:dnd_pass@postgres:5432/dnd_dm_assistant"
```

## Testing

### Run tests in Docker
```bash
# Production build
docker-compose exec backend cargo test

# Development environment
docker-compose -f docker-compose.dev.yml exec backend cargo test
```

### Run tests locally (if Rust is installed)
```bash
cd backend
cargo test
```

## API Testing

### Health Check
```bash
curl http://localhost:3000/health
```

### Register User
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "username": "testuser",
    "password": "password123"
  }'
```

### Login
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

## WebSocket Testing

### Using wscat (install with `npm install -g wscat`)
```bash
# Connect to WebSocket
wscat -c "ws://localhost:3000/ws?token=YOUR_JWT_TOKEN"

# Send a message
{"type": "JoinSession", "data": {"session_id": "session-uuid"}}
```

## Development Workflow

### 1. Start Development Environment
```bash
docker-compose -f docker-compose.dev.yml up -d
```

### 2. Make Code Changes
Edit files in the `backend/` directory. Changes will be reflected automatically.

### 3. View Logs
```bash
docker-compose -f docker-compose.dev.yml logs -f backend
```

### 4. Run Tests
```bash
docker-compose -f docker-compose.dev.yml exec backend cargo test
```

### 5. Database Operations
```bash
# Connect to database
docker-compose exec postgres psql -U dnd_user -d dnd_dm_assistant

# Run migrations
docker-compose exec backend sqlx migrate run
```

## Production Deployment

### 1. Environment Configuration
Create a `.env` file for production:
```bash
# .env
JWT_SECRET=your-super-secure-jwt-secret-here
DATABASE_URL=postgresql://dnd_user:dnd_pass@postgres:5432/dnd_dm_assistant
REDIS_URL=redis://redis:6379
RUST_LOG=info
```

### 2. Build and Deploy
```bash
# Build production images
docker-compose build

# Start services
docker-compose up -d

# Check health
docker-compose ps
```

### 3. SSL/HTTPS Setup
For production, consider using a reverse proxy (nginx) with SSL certificates.

## Troubleshooting

### Common Issues

#### 1. Port Already in Use
```bash
# Check what's using the port
lsof -i :3000

# Stop conflicting services
sudo systemctl stop <service-name>
```

#### 2. Database Connection Issues
```bash
# Check database logs
docker-compose logs postgres

# Restart database
docker-compose restart postgres
```

#### 3. Backend Build Failures
```bash
# Clean and rebuild
docker-compose down
docker-compose build --no-cache backend
docker-compose up -d
```

#### 4. Permission Issues
```bash
# Fix volume permissions
sudo chown -R $USER:$USER ./backend
```

### Debug Commands

#### Check Service Status
```bash
docker-compose ps
docker-compose logs
```

#### Access Container Shell
```bash
# Backend
docker-compose exec backend bash

# Database
docker-compose exec postgres psql -U dnd_user -d dnd_dm_assistant

# Redis
docker-compose exec redis redis-cli
```

#### Monitor Resources
```bash
# Container resource usage
docker stats

# Disk usage
docker system df
```

## Performance Optimization

### 1. Database Optimization
```bash
# Increase PostgreSQL shared buffers
POSTGRES_SHARED_BUFFERS=256MB
```

### 2. Redis Optimization
```bash
# Configure Redis memory limits
redis-cli CONFIG SET maxmemory 256mb
redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

### 3. Backend Optimization
```bash
# Set Rust optimization flags
RUSTFLAGS="-C target-cpu=native"
```

## Security Considerations

### 1. JWT Secret
- Use a strong, random JWT secret in production
- Rotate secrets regularly
- Store secrets in environment variables, not in code

### 2. Database Security
- Change default passwords
- Use SSL connections in production
- Regular backups

### 3. Network Security
- Use internal Docker networks
- Expose only necessary ports
- Implement rate limiting

## Backup and Recovery

### Database Backup
```bash
# Create backup
docker-compose exec postgres pg_dump -U dnd_user dnd_dm_assistant > backup.sql

# Restore backup
docker-compose exec -T postgres psql -U dnd_user dnd_dm_assistant < backup.sql
```

### Volume Backup
```bash
# Backup volumes
docker run --rm -v yoda_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres_backup.tar.gz -C /data .

# Restore volumes
docker run --rm -v yoda_postgres_data:/data -v $(pwd):/backup alpine tar xzf /backup/postgres_backup.tar.gz -C /data
```

## Monitoring

### Health Checks
All services include health checks:
- Backend: `GET /health`
- PostgreSQL: `pg_isready`
- Redis: `redis-cli ping`

### Log Monitoring
```bash
# Follow logs
docker-compose logs -f

# Search logs
docker-compose logs | grep ERROR
```

## Scaling

### Horizontal Scaling
```bash
# Scale backend service
docker-compose up -d --scale backend=3
```

### Load Balancing
Consider using nginx or traefik for load balancing multiple backend instances.

## Cleanup

### Remove All Services
```bash
docker-compose down -v
```

### Remove Images
```bash
docker-compose down --rmi all
```

### Complete Cleanup
```bash
docker system prune -a --volumes
```

## Support

For issues and questions:
1. Check the logs: `docker-compose logs`
2. Verify environment variables
3. Check service health: `docker-compose ps`
4. Review this documentation
5. Check the main README.md for additional information 