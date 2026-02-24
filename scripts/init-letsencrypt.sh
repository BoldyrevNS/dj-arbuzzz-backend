#!/bin/bash

# Initialize Let's Encrypt certificate for production
# Based on: https://github.com/wmnnd/nginx-certbot

if [ -z "$1" ]; then
  echo "Usage: ./scripts/init-letsencrypt.sh <domain> <email>"
  echo "Example: ./scripts/init-letsencrypt.sh example.com admin@example.com"
  exit 1
fi

if [ -z "$2" ]; then
  echo "Usage: ./scripts/init-letsencrypt.sh <domain> <email>"
  echo "Example: ./scripts/init-letsencrypt.sh example.com admin@example.com"
  exit 1
fi

DOMAIN=$1
EMAIL=$2
STAGING=${3:-0} # Set to 1 for testing

echo "🔐 Initializing Let's Encrypt for $DOMAIN..."

# Create required directories
mkdir -p certbot/conf
mkdir -p certbot/www
chmod 755 certbot/conf certbot/www

echo "### Checking directories..."
ls -la certbot/
echo ""

# Download recommended TLS parameters
if [ ! -e "certbot/conf/options-ssl-nginx.conf" ] || [ ! -e "certbot/conf/ssl-dhparams.pem" ]; then
  echo "### Downloading recommended TLS parameters..."
  mkdir -p certbot/conf
  curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf > certbot/conf/options-ssl-nginx.conf
  curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem > certbot/conf/ssl-dhparams.pem
  echo ""
fi

# Create dummy certificate for nginx to start
echo "### Creating dummy certificate for $DOMAIN..."
path="/etc/letsencrypt/live/$DOMAIN"
mkdir -p "certbot/conf/live/$DOMAIN"
docker-compose run --rm --entrypoint "\
  openssl req -x509 -nodes -newkey rsa:4096 -days 1\
    -keyout '$path/privkey.pem' \
    -out '$path/fullchain.pem' \
    -subj '/CN=localhost'" certbot
echo ""

# Create temporary HTTP-only nginx config for certificate verification
echo "### Creating temporary nginx configuration..."
cat > nginx/nginx.tmp.conf << 'EOF'
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    server {
        listen 80;
        server_name _;

        location /.well-known/acme-challenge/ {
            root /var/www/certbot;
            try_files \$uri =404;
        }

        location / {
            return 200 'Certbot verification server';
            add_header Content-Type text/plain;
        }
    }
}
EOF

# Backup current nginx config
if [ -f nginx/nginx.conf ]; then
    cp nginx/nginx.conf nginx/nginx.conf.backup
fi

cp nginx/nginx.tmp.conf nginx/nginx.conf
echo ""

# Create test file to verify volume mounting
echo "### Creating test challenge file..."
mkdir -p certbot/www/.well-known/acme-challenge
echo "test" > certbot/www/.well-known/acme-challenge/test
chmod -R 755 certbot/www
echo ""

# Stop all services to release ports
echo "### Stopping all services..."
docker-compose down
echo ""

# Start nginx with HTTP-only config
echo "### Starting nginx (standalone mode for certificate verification)..."
docker-compose up --no-deps --force-recreate -d nginx
echo ""

# Wait for nginx to start
echo "### Waiting for nginx to be ready..."
sleep 5

# Verify volume is mounted correctly
echo "### Verifying certbot volume..."
docker-compose exec nginx ls -la /var/www/certbot/.well-known/acme-challenge/ || echo "⚠ Challenge directory not found"
echo ""

# Test nginx is serving the test file
echo "### Testing challenge file access..."
if docker-compose exec -T nginx cat /var/www/certbot/.well-known/acme-challenge/test 2>/dev/null | grep -q "test"; then
    echo "✓ Test file is accessible inside container"
else
    echo "✗ Test file NOT accessible inside container"
    exit 1
fi

# Test from localhost
if docker-compose exec -T nginx wget -q -O- http://localhost/.well-known/acme-challenge/test 2>/dev/null | grep -q "test"; then
    echo "✓ Nginx is serving challenge files correctly"
else
    echo "✗ Nginx is NOT serving challenge files"
    docker-compose logs nginx | tail -20
    exit 1
fi
echo ""

# Delete dummy certificate
echo "### Deleting dummy certificate for $DOMAIN..."
docker-compose run --rm --entrypoint "\
  rm -Rf /etc/letsencrypt/live/$DOMAIN && \
  rm -Rf /etc/letsencrypt/archive/$DOMAIN && \
  rm -Rf /etc/letsencrypt/renewal/$DOMAIN.conf" certbot
echo ""

# Request Let's Encrypt certificate
echo "### Requesting Let's Encrypt certificate for $DOMAIN..."
staging_arg=""
if [ $STAGING != "0" ]; then
  staging_arg="--staging"
fi

docker-compose run --rm --entrypoint "\
  certbot certonly --webroot -w /var/www/certbot \
    $staging_arg \
    --email $EMAIL \
    --agree-tos \
    --no-eff-email \
    -d $DOMAIN" certbot
echo ""

# Check if certificate was obtained
if [ -d "certbot/conf/live/$DOMAIN" ]; then
    echo "✓ Certificate obtained successfully"
else
    echo "✗ Certificate was not obtained"
    echo "Check the logs above for errors"
    exit 1
fi

# Restore production nginx config or create it
echo "### Updating nginx to production configuration..."
export DOMAIN=$DOMAIN
envsubst '${DOMAIN}' < nginx/nginx.prod.conf > nginx/nginx.conf
echo ""

# Reload nginx with production config
echo "### Reloading nginx..."
docker-compose exec nginx nginx -s reload
echo ""

echo "✅ Let's Encrypt certificate obtained successfully!"
echo ""
echo "To enable automatic renewal, make sure certbot service is running:"
echo "  docker-compose --profile production up -d certbot"
