# Nginx Configuration

Эта директория содержит конфигурационные файлы для Nginx reverse proxy.

## Файлы

- **nginx.conf** - конфигурация для разработки с самоподписанными сертификатами
- **nginx.prod.conf** - шаблон для production с Let's Encrypt
- **ssl/** - директория для SSL сертификатов

## Использование

### Разработка

1. Сгенерируйте самоподписанный сертификат:
   ```bash
   ../scripts/generate-ssl.sh
   ```

2. Запустите Docker Compose:
   ```bash
   docker-compose up -d
   ```

3. Приложение доступно на:
   - https://localhost (frontend)
   - https://localhost/api (backend API)

### Production

1. Настройте домен в `.env`:
   ```env
   DOMAIN=your-domain.com
   ```

2. Инициализируйте Let's Encrypt:
   ```bash
   ../scripts/init-letsencrypt.sh your-domain.com admin@your-domain.com
   ```

3. Запустите с Certbot для автообновления:
   ```bash
   docker-compose --profile production up -d
   ```

## Особенности конфигурации

- ✅ HTTP/2 и TLS 1.2/1.3
- ✅ Gzip compression для статических ресурсов
- ✅ WebSocket поддержка для radio stream
- ✅ Security headers (HSTS, X-Frame-Options, etc.)
- ✅ Автоматическое перенаправление HTTP → HTTPS
- ✅ Proxy pass для frontend и backend
- ✅ Client max body size: 100MB

## Проксирование

Nginx проксирует следующие пути:

- `/` → `frontend:3000` (Nuxt.js frontend)
- `/api` → `backend:8080` (Rust backend API)
- `/ws` → `backend:8080` (WebSocket для radio)

## Проверка конфигурации

```bash
# Проверить синтаксис
docker-compose exec nginx nginx -t

# Перезагрузить конфигурацию
docker-compose exec nginx nginx -s reload

# Просмотр логов
docker-compose logs -f nginx
```

## Отладка

### Проверка SSL сертификатов

```bash
# Информация о сертификате
openssl x509 -in nginx/ssl/cert.pem -text -noout

# Проверка на сервере
openssl s_client -connect localhost:443
```

### Проверка проксирования

```bash
# Проверка frontend
curl -k https://localhost

# Проверка backend API
curl -k https://localhost/api/health
```

## Кастомизация

Чтобы изменить конфигурацию nginx:

1. Отредактируйте `nginx.conf`
2. Проверьте синтаксис: `docker-compose exec nginx nginx -t`
3. Перезагрузите: `docker-compose exec nginx nginx -s reload`

Для production используйте `nginx.prod.conf` как шаблон.
