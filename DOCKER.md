# DJ Arbuzzz Docker Setup

Этот проект включает в себя Docker-конфигурацию для запуска полного стека приложения DJ Arbuzzz.

## Компоненты

- **PostgreSQL 17** - основная база данных
- **Redis 7** - кеш и очереди (с persistence)
- **Rust Backend** - API сервер на Axum
- **Nuxt Frontend** - веб-интерфейс на Nuxt 4
- **Nginx** - обратный прокси с HTTPS
- **Diesel Migrations** - автоматические миграции БД
- **Certbot** - автоматическое обновление SSL сертификатов (опционально)

## Быстрый старт

### 1. Копирование переменных окружения

```bash
cp .env.example .env
```

Отредактируйте `.env` файл и укажите свои значения для production.

### 2. Запуск всего кластера

```bash
docker-compose up -d
```

Эта команда:
- Поднимет PostgreSQL и Redis
- Запустит миграции Diesel
- Сборка и запуск Rust backend
- Сборка и запуск Nuxt frontend
- Запустит Nginx с HTTPS

### 3. Генерация SSL сертификатов для разработки

```bash
chmod +x scripts/generate-ssl.sh
./scripts/generate-ssl.sh
```

### 4. Проверка статуса

```bash
docker-compose ps
```

### 4. Просмотр логов

```bash
# Все сервисы
docker-compose logs -f

# Конкретный сервис
docker-compose logs -f backend
docker-compose logs -f frontend
```

## Доступ к сервисам

- **Frontend (HTTPS)**: https://localhost
- **Frontend (HTTP)**: http://localhost (перенаправляет на HTTPS)
- **Backend API**: https://localhost/api
- **PostgreSQL**: localhost:5432
- **Redis**: localhost:6379

⚠️ **Важно**: При использовании самоподписанного сертификата браузер покажет предупреждение безопасности. Это нормально для разработки.

## Управление

### Остановка всех сервисов

```bash
docker-compose down
```

### Остановка с удалением volumes (очистка данных)

```bash
docker-compose down -v
```

### Пересборка образов

```bash
docker-compose build --no-cache
docker-compose up -d
```

### Выполнение миграций вручную

```bash
docker-compose run --rm migrations
```

### Масштабирование сервисов

```bash
docker-compose up -d --scale backend=3
```

## Volumes

- `postgres_data` - данные PostgreSQL
- `redis_data` - данные Redis
- `songs_data` - музыкальные файлы

## Настройка для production

1. **Измените все секреты** в `.env`:
   - `JWT_SECRET`
   - `NUXT_API_SECRET`
   - `POSTGRES_PASSWORD`

2. **Настройте SMTP** для отправки email

3. **Настройте Let's Encrypt для SSL**:
   ```bash
   chmod +x scripts/init-letsencrypt.sh
   ./scripts/init-letsencrypt.sh your-domain.com admin@your-domain.com
   ```

4. **Запустите Certbot для авто-обновления сертификатов**:
   ```bash
   docker-compose --profile production up -d certbot
   ```

5. **Настройте бэкапы** для PostgreSQL

## HTTPS Конфигурация

### Разработка (самоподписанный сертификат)

```bash
# Генерация сертификата
./scripts/generate-ssl.sh

# Запуск
docker-compose up -d
```

### Production (Let's Encrypt)

```bash
# Инициализация Let's Encrypt
./scripts/init-letsencrypt.sh example.com admin@example.com

# Тестовый режим (для проверки)
./scripts/init-letsencrypt.sh example.com admin@example.com 1

# Запуск с автообновлением
docker-compose --profile production up -d
```

### Обновление конфигурации Nginx

Если нужно изменить nginx конфигурацию:

```bash
# Редактировать файл
nano nginx/nginx.conf

# Проверить конфигурацию
docker-compose exec nginx nginx -t

# Перезагрузить nginx
docker-compose exec nginx nginx -s reload
```

## Разработка

Для разработки рекомендуется использовать локальный запуск с hot-reload:

```bash
# Backend
cd server
cargo watch -x run

# Frontend
cd client
pnpm dev
```

И поднимать только инфраструктуру (без nginx):

```bash
docker-compose up -d postgres redis
```

Или использовать полный стек с nginx:

```bash
# Генерируем SSL сертификаты
./scripts/generate-ssl.sh

# Запускаем всё
docker-compose up -d

# Приложение доступно на https://localhost
```

## Отладка

### Подключение к контейнеру

```bash
docker-compose exec backend sh
docker-compose exec frontend sh
docker-compose exec postgres psql -U djarbuzzz -d djarbuzzz_db
```

### Проверка health-check

```bash
docker-compose exec backend curl http://localhost:8080/health
```

## Требования

- Docker Engine 20.10+
- Docker Compose 2.0+
- Минимум 4GB RAM
- Минимум 10GB свободного места

## Устранение неполадок

### Backend не стартует

1. Проверьте логи: `docker-compose logs backend`
2. Убедитесь, что миграции выполнены: `docker-compose logs migrations`
3. Проверьте DATABASE_URL в `.env`

### Frontend недоступен

1. Проверьте `NUXT_PUBLIC_API_BASE` в `.env`
2. Убедитесь, что backend запущен и здоров
3. Проверьте логи: `docker-compose logs frontend`

### Ошибки базы данных

1. Сбросьте базу: `docker-compose down -v && docker-compose up -d`
2. Проверьте подключение: `docker-compose exec postgres pg_isready`

### Проблемы с HTTPS/SSL

1. **Сертификат не найден**: Запустите `./scripts/generate-ssl.sh`
2. **Nginx не стартует**: Проверьте конфигурацию: `docker-compose logs nginx`
3. **Браузер не доверяет сертификату**: Это нормально для самоподписанных сертификатов. Добавьте исключение в браузере
4. **Let's Encrypt ошибки**: Проверьте, что домен правильно указан и доступен из интернета

## Лицензия

MIT
