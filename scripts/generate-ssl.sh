#!/bin/bash

# Generate self-signed SSL certificate for development

echo "🔐 Generating self-signed SSL certificate for development..."

# Create ssl directory if it doesn't exist
mkdir -p nginx/ssl

# Generate private key and certificate
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx/ssl/key.pem \
  -out nginx/ssl/cert.pem \
  -subj "/C=RU/ST=State/L=City/O=DJ-Arbuzzz/CN=localhost"

# Set proper permissions
chmod 600 nginx/ssl/key.pem
chmod 644 nginx/ssl/cert.pem

echo "✅ Self-signed certificate generated successfully!"
echo ""
echo "Certificate location: nginx/ssl/cert.pem"
echo "Private key location: nginx/ssl/key.pem"
echo ""
echo "⚠️  Warning: This is a self-signed certificate for development only."
echo "   Your browser will show a security warning. This is expected."
echo ""
echo "To use in production, run: ./scripts/init-letsencrypt.sh"
