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

# Update nginx config to use production template
echo "### Updating nginx configuration..."
export DOMAIN=$DOMAIN
envsubst '${DOMAIN}' < nginx/nginx.prod.conf > nginx/nginx.conf
echo ""

# Start nginx
echo "### Starting nginx..."
docker-compose up --force-recreate -d nginx
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

# Reload nginx
echo "### Reloading nginx..."
docker-compose exec nginx nginx -s reload
echo ""

echo "✅ Let's Encrypt certificate obtained successfully!"
echo ""
echo "To enable automatic renewal, make sure certbot service is running:"
echo "  docker-compose --profile production up -d certbot"
