# 🚀 Быстрая шпаргалка по деплою

## Перед началом

- [ ] Ubuntu 22.04 на сервере
- [ ] Домен указывает на IP сервера
- [ ] Порты 80, 443, 22 открыты

## 1. Установка Docker (на сервере)

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

Перелогиниться!

## 2. Клонирование проекта

```bash
sudo mkdir -p /var/www/dj-arbuzzz
sudo chown $USER:$USER /var/www/dj-arbuzzz
cd /var/www/dj-arbuzzz
git clone YOUR_REPO_URL .
chmod +x scripts/*.sh
```

## 3. Настройка .env

```bash
cp .env.example .env
nano .env
```

**Обязательно изменить:**
- `DOMAIN=your-domain.com`
- `POSTGRES_PASSWORD=` (сгенерировать: `openssl rand -base64 32`)
- `JWT_SECRET=` (сгенерировать: `openssl rand -base64 48`)
- `NUXT_API_SECRET=` (сгенерировать: `openssl rand -base64 32`)
- `NUXT_PUBLIC_API_BASE=https://your-domain.com/api`
- SMTP настройки (если нужна почта)

## 4. SSL сертификаты

```bash
./scripts/init-letsencrypt.sh your-domain.com admin@your-domain.com
```

## 5. Запуск

```bash
docker-compose up -d
docker-compose --profile production up -d certbot
```

## 6. Проверка

```bash
docker-compose ps
docker-compose logs -f
curl https://your-domain.com
```

## GitHub Actions (опционально)

**Settings > Secrets > Add:**
- `SSH_PRIVATE_KEY` - содержимое `~/.ssh/deploy_key`
- `SERVER_HOST` - ваш домен или IP
- `SERVER_USER` - имя пользователя SSH
- `DEPLOY_PATH` - `/var/www/dj-arbuzzz`

**Деплой:**
```bash
git tag v1.0.0
git push origin v1.0.0
```

## Полезные команды

```bash
# Логи
docker-compose logs -f
docker-compose logs -f backend

# Перезапуск
docker-compose restart
docker-compose restart backend

# Обновление
cd /var/www/dj-arbuzzz
git pull
docker-compose up -d --build

# Остановка
docker-compose down

# Бэкап БД
docker exec dj-arbuzzz-postgres pg_dump -U djarbuzzz_prod djarbuzzz_production > backup.sql
```

## Troubleshooting

**Nginx не стартует:**
```bash
docker-compose logs nginx
ls -la nginx/ssl/
```

**Backend не подключается к БД:**
```bash
docker-compose exec postgres pg_isready
docker-compose logs migrations
```

**Let's Encrypt ошибка:**
```bash
# Проверить DNS
dig your-domain.com
# Попробовать staging
./scripts/init-letsencrypt.sh your-domain.com your@email.com 1
```

---

📖 **Подробная инструкция:** [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)
