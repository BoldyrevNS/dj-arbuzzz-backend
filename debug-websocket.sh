#!/bin/bash

echo "=== WebSocket Debugging ==="
echo ""

echo "1. Checking nginx configuration..."
docker-compose exec nginx cat /etc/nginx/conf.d/app.conf | grep -A 10 "location /ws"
echo ""

echo "2. Testing WebSocket endpoint from nginx container..."
docker-compose exec nginx wget --spider --server-response http://backend:8080/api/v1/ws/ws 2>&1 | head -20
echo ""

echo "3. Checking if backend is listening..."
docker-compose exec backend netstat -tlnp 2>/dev/null || docker-compose exec backend ss -tlnp
echo ""

echo "4. Backend logs (last 50 lines, filtering WebSocket)..."
docker-compose logs --tail=50 backend | grep -i "websocket\|/ws\|connection"
echo ""

echo "5. Nginx access logs (last 20 lines)..."
docker-compose logs --tail=20 nginx | grep -v "certbot"
echo ""

echo "6. Testing WebSocket from host machine..."
echo "Trying to connect via websocat (if installed)..."
if command -v websocat &> /dev/null; then
    timeout 3 websocat wss://djarbuzzz-music.ru/ws -v 2>&1 || echo "Connection failed or timed out"
else
    echo "websocat not installed. Install with: brew install websocat"
    echo "Alternative: curl -i -N -H 'Connection: Upgrade' -H 'Upgrade: websocket' -H 'Sec-WebSocket-Version: 13' -H 'Sec-WebSocket-Key: test' https://djarbuzzz-music.ru/ws"
fi

echo ""
echo "=== End of diagnostics ==="
