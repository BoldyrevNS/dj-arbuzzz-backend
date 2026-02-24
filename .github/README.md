# GitHub Actions CI/CD

Этот проект использует GitHub Actions для автоматизации CI/CD процессов.

## 📋 Workflows

### 1. CI Pipeline (`.github/workflows/ci.yml`)

Автоматически запускается при:
- Push в ветки `main` или `develop`
- Открытии Pull Request

**Задачи:**
- ✅ **Backend (Rust)**
  - Проверка форматирования (`cargo fmt`)
  - Статический анализ (`cargo clippy`)
  - Сборка проекта
- ✅ **Frontend (Nuxt)**
  - Установка зависимостей (pnpm)
  - Линтинг (eslint)
  - Сборка проекта
- ✅ **Docker**
  - Валидация docker-compose

### 2. Deploy (`.github/workflows/deploy.yml`)

Запускается:
- Вручную через GitHub Actions UI
- При создании тега `v*.*.*`

**Задачи:**
- 🚀 Деплой приложения на production/staging сервер
- 🔄 Обновление Docker контейнеров
- ✅ Health check после деплоя

## 🔐 Настройка Secrets

Перейдите в `Settings > Secrets and variables > Actions` и добавьте следующие secrets:

### Для Deploy workflow:

| Secret | Описание | Пример |
|--------|----------|--------|
| `SSH_PRIVATE_KEY` | Приватный SSH ключ для доступа к серверу | `-----BEGIN OPENSSH PRIVATE KEY-----...` |
| `SERVER_HOST` | Хост сервера | `example.com` или `192.168.1.100` |
| `SERVER_USER` | Пользователь SSH | `deploy` или `ubuntu` |
| `DEPLOY_PATH` | Путь на сервере для деплоя | `/var/www/dj-arbuzzz` |

### Опционально (для уведомлений):

| Secret | Описание |
|--------|----------|
| `SLACK_WEBHOOK_URL` | Webhook для уведомлений в Slack |
| `TELEGRAM_BOT_TOKEN` | Токен Telegram бота |
| `TELEGRAM_CHAT_ID` | ID чата для уведомлений |

## 🚀 Использование

### Запуск CI

CI запускается автоматически при каждом push или PR. Проверить статус можно во вкладке "Actions".

### Деплой на сервер

1. **Через GitHub UI:**
   - Перейдите во вкладку "Actions"
   - Выберите "Deploy to Production"
   - Нажмите "Run workflow"
   - Выберите environment (production/staging)
   - Нажмите "Run workflow"

2. **Автоматически при создании релиза:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

## 🔧 Настройка сервера для деплоя

### 1. Создание SSH ключа

```bash
# На локальной машине
ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/github_deploy

# Копируем публичный ключ на сервер
ssh-copy-id -i ~/.ssh/github_deploy.pub user@your-server.com

# Добавляем приватный ключ в GitHub Secrets
cat ~/.ssh/github_deploy
```

### 2. Подготовка сервера

```bash
# Подключаемся к серверу
ssh user@your-server.com

# Создаем директорию для деплоя
sudo mkdir -p /var/www/dj-arbuzzz
sudo chown $USER:$USER /var/www/dj-arbuzzz

# Устанавливаем Docker и Docker Compose
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Устанавливаем Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Перелогиниваемся для применения группы docker
exit
ssh user@your-server.com

# Проверяем установку
docker --version
docker-compose --version
```

### 3. Настройка окружения на сервере

```bash
cd /var/www/dj-arbuzzz

# Создаем .env файл с production настройками
nano .env
```

```env
# Production .env
POSTGRES_USER=djarbuzzz_prod
POSTGRES_PASSWORD=super_secure_password_here
POSTGRES_DB=djarbuzzz_production
DATABASE_URL=postgresql://djarbuzzz_prod:super_secure_password_here@postgres:5432/djarbuzzz_production

REDIS_URL=redis://redis:6379

JWT_SECRET=your-very-secure-jwt-secret-min-32-chars
NUXT_API_SECRET=your-secure-nuxt-secret

DOMAIN=your-domain.com
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USERNAME=your-email@example.com
SMTP_PASSWORD=your-smtp-password
SMTP_FROM=noreply@your-domain.com
```

### 4. Инициализация SSL сертификатов

```bash
# Будет выполнено на сервере через GitHub Actions при первом деплое
# Или вручную:
chmod +x scripts/init-letsencrypt.sh
./scripts/init-letsencrypt.sh your-domain.com admin@your-domain.com
```

## 📊 Badges

Добавьте badges в README.md:

```markdown
![CI](https://github.com/username/dj-arbuzzz-backend/workflows/CI/badge.svg)
![Security Scan](https://github.com/username/dj-arbuzzz-backend/workflows/Security%20Scan/badge.svg)
```

## 🐛 Отладка

### Просмотр логов workflow

1. Перейдите во вкладку "Actions"
2. Выберите нужный workflow run
3. Кликните на job для просмотра подробных логов

### Локальное тестирование

Используйте [act](https://github.com/nektos/act) для локального запуска GitHub Actions:

```bash
# Установка act
brew install act  # macOS
# или
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Запуск CI локально
act -j backend
act -j frontend
act -j docker
```

### Частые проблемы

**1. Ошибка "Permission denied" при деплое**
- Проверьте, что SSH ключ добавлен на сервер
- Убедитесь, что пользователь имеет доступ к директории деплоя

**2. Docker образы не собираются**
- Проверьте Dockerfile синтаксис локально
- Убедитесь, что все зависимости доступны

**3. Тесты падают в CI**
- Проверьте версии зависимостей
- Убедитесь, что сервисы (postgres, redis) запустились
- Проверьте переменные окружения

## 🔄 Обновление workflows

После изменения workflow файлов:

```bash
git add .github/workflows/
git commit -m "Update GitHub Actions workflows"
git push origin main
```

Изменения автоматически применятся при следующем запуске.

## 📚 Дополнительные ресурсы

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Docker Build Push Action](https://github.com/docker/build-push-action)
- [SSH Deploy Action](https://github.com/appleboy/ssh-action)
