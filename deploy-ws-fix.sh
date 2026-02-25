#!/bin/bash

echo "=== Deploying WebSocket fixes ==="
echo ""

# Commit and push changes
echo "1. Committing changes..."
git add -A
git commit -m "fix: WebSocket paths and SSL config in nginx.conf"
git push

echo ""
echo "2. On your server, run these commands:"
echo ""
cat << 'EOF'
cd /var/www/dj-arbuzzz

# Pull latest changes
echo "Pulling latest changes..."
git pull

# Rebuild backend with WebSocket logging
echo "Rebuilding backend..."
docker-compose build backend

# Rebuild frontend with updated WebSocket connection
echo "Rebuilding frontend..."
docker-compose build frontend

# Restart all services to apply changes
echo "Restarting services..."
docker-compose restart backend frontend nginx

# Wait for startup
sleep 5

# Show logs
echo ""
echo "=== Backend logs (last 30 lines) ==="
docker-compose logs --tail=30 backend

echo ""
echo "=== Nginx logs (last 20 lines) ==="
docker-compose logs --tail=20 nginx

echo ""
echo "=== Service status ==="
docker-compose ps

echo ""
echo "=== Testing WebSocket endpoint from nginx container ==="
docker-compose exec nginx wget -O- --spider --server-response http://backend:8080/api/v1/ws/ws 2>&1 | head -15

echo ""
echo "✅ Deployment complete!"
echo ""
echo "Open browser console at https://djarbuzzz-music.ru"
echo "You should see:"
echo "  [WebSocket] Connecting to: wss://djarbuzzz-music.ru/ws"
echo "  [WebSocket] Connected"
echo ""
echo "Check backend logs for:"
echo "  WebSocket connection attempt"
echo "  WebSocket connection established"

EOF

