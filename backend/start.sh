#!/bin/bash
set -e

echo "🚀 Starting YoDA Backend..."

# Wait for database to be ready
echo "⏳ Waiting for database to be ready..."
until sqlx migrate info --database-url "$DATABASE_URL" > /dev/null 2>&1; do
  echo "Database not ready, waiting..."
  sleep 2
done

# Run database migrations
echo "📊 Running database migrations..."
sqlx migrate run --database-url "$DATABASE_URL"

# Start the backend server
echo "🎯 Starting backend server..."
exec /usr/local/bin/backend 