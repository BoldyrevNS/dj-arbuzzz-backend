# DJ Arbuzzz

![CI](https://github.com/YOUR_USERNAME/dj-arbuzzz-backend/workflows/CI/badge.svg)
![Security Scan](https://github.com/YOUR_USERNAME/dj-arbuzzz-backend/workflows/Security%20Scan/badge.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> DJ радио-платформа с Rust backend и Nuxt frontend

## ✨ Возможности

- 🎵 Потоковое радио вещание
- 🔐 Аутентификация и авторизация
- 📻 Управление треками и плейлистами
- 🎛️ Панель управления DJ
- 🐳 Docker контейнеризация
- 🔒 HTTPS по умолчанию
- 🚀 CI/CD через GitHub Actions

## 🛠️ Технологии

**Backend:**
- Rust 1.84
- Axum (web framework)
- Diesel (ORM)
- PostgreSQL 17
- Redis 7

**Frontend:**
- Nuxt 4
- Vue 3
- TypeScript
- Pinia

**Инфраструктура:**
- Docker & Docker Compose
- Nginx (reverse proxy)
- Let's Encrypt (SSL)

## 🚀 Быстрый старт

### Использование Make (рекомендуется)

```bash
# Показать все доступные команды
make help

# Быстрая настройка и запуск
make setup
make dev
```

### Ручная установка

#### Требования

- Docker 20.10+
- Docker Compose 2.0+
- (Опционально) Rust 1.84+
- (Опционально) Node.js 22+ и pnpm 9+

## 🚀 Запуск за 3 шага

### 1. Настройка окружения

```bash
cp .env.example .env
```

### 2. Генерация SSL сертификатов (для разработки)

```bash
chmod +x scripts/generate-ssl.sh
./scripts/generate-ssl.sh
```

### 3. Запуск всего стека

```bash
docker-compose up -d
```

## ✅ Готово!

Приложение доступно на:
- **Frontend**: https://localhost
- **Backend API**: https://localhost/api

⚠️ Браузер покажет предупреждение о самоподписанном сертификате - это нормально для разработки.

## 📋 Полная документация

См. [DOCKER.md](DOCKER.md) для подробной информации о:
- Production настройке с Let's Encrypt
- Управлении сервисами
- Отладке и troubleshooting

## 🔧 Полезные команды

```bash
# Просмотр логов
docker-compose logs -f
# или
make docker-logs

# Остановка
docker-compose down
# или
make docker-down

# Перезапуск с пересборкой
docker-compose up -d --build
# или
make docker-restart

# Проверка статуса
docker-compose ps
# или
make docker-status
```

## 💻 Разработка

### Локальная разработка

```bash
# Запустить только инфраструктуру
make dev-infra

# В одном терминале: backend
make dev-backend

# В другом терминале: frontend
make dev-frontend
```

### Линтинг и форматирование

```bash
# Проверить код
make lint

# Исправить автоматически
make fix
```

## 🔒 CI/CD

Проект использует GitHub Actions для автоматизации:

- **CI Pipeline** - автоматические линтинг и проверки форматирования при каждом PR
- **Security Scan** - еженедельное сканирование безопасности
- **Auto Deploy** - автоматический деплой при создании тега

Подробнее см. [.github/README.md](.github/README.md)

### Создание релиза

```bash
# 1. Убедитесь, что все изменения закоммичены
git status

# 2. Создайте тег версии
git tag v1.0.0

# 3. Запушьте тег
git push origin v1.0.0

# GitHub Actions автоматически:
# - Запустит проверки кода
# - Опубликует релиз
# - Задеплоит на production (если настроено)
```

## 🚀 Production деплой

### Полная пошаговая инструкция

📖 **[DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)** - Подробная инструкция по деплою с нуля

**Что нужно:**
- ✅ Сервер (Ubuntu 22.04+ / Debian 11+)
- ✅ Домен
- ✅ SSH доступ

**Время деплоя:** ~20-30 минут

### Быстрый деплой

Если все зависимости установлены:

```bash
# На сервере
sudo apt update && sudo apt install -y docker.io docker-compose
git clone https://github.com/YOUR_USERNAME/dj-arbuzzz-backend.git
cd dj-arbuzzz-backend
cp .env.example .env
nano .env  # Настройте переменные окружения

# Настройка SSL
make ssl-prod domain=your-domain.com email=admin@your-domain.com

# Запуск
docker-compose up -d
```

## 📚 Полная документация

- **[DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)** - 🚀 Полная пошаговая инструкция по деплою
- **[QUICK_DEPLOY.md](QUICK_DEPLOY.md)** - ⚡ Быстрая шпаргалка для деплоя
- [DOCKER.md](DOCKER.md) - подробная информация о Docker
- [.github/README.md](.github/README.md) - CI/CD и автодеплой через GitHub Actions
- [nginx/README.md](nginx/README.md) - конфигурация Nginx
- [CONTRIBUTING.md](CONTRIBUTING.md) - гайд для контрибьюторов

## 🤝 Contributing

1. Fork проекта
2. Создайте feature branch (`git checkout -b feature/amazing-feature`)
3. Закоммитьте изменения (`git commit -m 'feat: add amazing feature'`)
4. Запушьте в branch (`git push origin feature/amazing-feature`)
5. Откройте Pull Request

### Commit Convention

Используйте [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - новая функциональность
- `fix:` - исправление ошибки
- `docs:` - изменения в документации
- `style:` - форматирование кода
- `refactor:` - рефакторинг
- `test:` - добавление тестов
- `chore:` - обновление зависимостей, настройки

## 📝 Лицензия

MIT

## 👥 Авторы

- [@YOUR_USERNAME](https://github.com/YOUR_USERNAME)

