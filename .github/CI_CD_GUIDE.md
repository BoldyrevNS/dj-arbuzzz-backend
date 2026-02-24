# CI/CD Quick Reference

## 📁 Созданные файлы

### GitHub Actions Workflows

#### `.github/workflows/ci.yml` - Continuous Integration
**Триггеры:**
- Push в `main`, `develop`
- Pull Request

**Что делает:**
- ✅ Проверяет Rust код (fmt, clippy, build)
- ✅ Проверяет Nuxt код (lint, build)
- ✅ Валидирует Docker Compose
- ✅ Запускается быстро без внешних сервисов

**Как использовать:**
```bash
# Автоматически запускается при push или PR
git push origin feature-branch
```

#### `.github/workflows/deploy.yml` - Production Deployment
**Триггеры:**
- Ручной запуск (workflow_dispatch)
- Push тега `v*.*.*`

**Что делает:**
- 🚀 Деплоит на production/staging сервер
- 📂 Копирует файлы через rsync
- 🐳 Запускает docker-compose на сервере
- ✅ Проверяет health check

**Как использовать:**
```bash
# Через GitHub UI:
# Actions > Deploy to Production > Run workflow

# Или через тег (автоматически):
git tag v1.0.0 && git push origin v1.0.0
```

**Требует настройки Secrets:**
- `SSH_PRIVATE_KEY` - SSH ключ для доступа к серверу
- `SERVER_HOST` - Хост сервера (example.com)
- `SERVER_USER` - SSH пользователь
- `DEPLOY_PATH` - Путь на сервере (/var/www/app)

#### `.github/workflows/security.yml` - Security Scanning
**Триггеры:**
- Push в `main`, `develop`
- Pull Request
- Каждый понедельник в 00:00 UTC

**Что делает:**
- 🔍 Сканирует Rust зависимости (cargo-audit)
- 🔍 Сканирует npm зависимости (pnpm audit)
- 🔍 Сканирует Docker образы (Trivy)
- 📊 Загружает результаты в GitHub Security

**Как использовать:**
```bash
# Автоматически запускается
# Смотреть результаты: Security > Code scanning alerts
```

#### `.github/workflows/release.yml` - Release Management
**Триггеры:**
- Push тега `v*.*.*`

**Что делает:**
- 📝 Создает GitHub Release
- 🔖 Генерирует changelog
- 📦 Прикрепляет ссылки на Docker образы

**Как использовать:**
```bash
git tag v1.0.0
git push origin v1.0.0
# Смотреть: Releases
```

### Configuration Files

#### `.github/dependabot.yml` - Dependency Updates
**Что делает:**
- 🔄 Автоматически обновляет зависимости
- 📅 Еженедельно (понедельник в 09:00)
- 📦 Rust Cargo, npm, Docker, GitHub Actions

**Группы обновлений:**
- Production dependencies (minor + patch)
- Development dependencies (minor + patch)

#### `.github/release-changelog-config.json` - Release Notes Config
Конфигурация для автоматической генерации changelog в релизах.

**Категории:**
- 🚀 Features
- 🐛 Bug Fixes
- 📚 Documentation
- 🔧 Maintenance
- 🔐 Security

### Templates

#### `.github/pull_request_template.md`
Шаблон для Pull Request с чеклистом проверок.

#### `.github/ISSUE_TEMPLATE/bug_report.yml`
Форма для сообщения об ошибках.

#### `.github/ISSUE_TEMPLATE/feature_request.yml`
Форма для предложения новых функций.

#### `.github/ISSUE_TEMPLATE/config.yml`
Конфигурация issue templates.

## 🔐 GitHub Secrets

### Настройка: Settings > Secrets and variables > Actions

**Обязательные для Deploy:**
```
SSH_PRIVATE_KEY    - Приватный SSH ключ
SERVER_HOST        - example.com
SERVER_USER        - deploy
DEPLOY_PATH        - /var/www/dj-arbuzzz
```

**Опциональные:**
```
SLACK_WEBHOOK_URL     - Уведомления в Slack
TELEGRAM_BOT_TOKEN    - Telegram бот
TELEGRAM_CHAT_ID      - ID чата
```

## 🚀 Workflow Examples

### Типичный workflow разработки

```bash
# 1. Создать ветку
git checkout -b feature/new-feature

# 2. Внести изменения
# ... код ...

# 3. Закоммитить
git add .
git commit -m "feat: add new feature"

# 4. Запушить
git push origin feature/new-feature

# 5. Создать PR на GitHub
# CI автоматически запустится

# 6. После одобрения -> merge в main
# CI снова запустится

# 7. Создать релиз
git tag v1.0.0
git push origin v1.0.0

# Автоматически:
# - Создастся GitHub Release
# - Задеплоится на production (если настроено)
```

### Hotfix workflow

```bash
# 1. Создать hotfix ветку
git checkout -b hotfix/critical-bug

# 2. Исправить и закоммитить
git commit -m "fix: resolve critical bug"

# 3. Создать PR с меткой hotfix
# CI проверит

# 4. После merge - создать патч релиз
git tag v1.0.1
git push origin v1.0.1
```

## 📊 Monitoring

### Просмотр статуса workflows

```bash
# Через GitHub UI:
# Repository > Actions

# Или через GitHub CLI:
gh run list
gh run view <run-id>
gh run watch
```

### Просмотр логов

```bash
# Через GitHub CLI:
gh run view --log

# Или скачать артефакты:
gh run download <run-id>
```

## 🔧 Troubleshooting

### CI fails

**Backend build fails:**
```bash
# Проверить локально:
cd server
cargo build
cargo clippy
```

**Frontend lint fails:**
```bash
cd client
pnpm run lint:fix
git add . && git commit --amend
```

**Docker validation fails:**
```bash
# Проверить локально:
docker-compose config
```

### Deploy fails

**SSH connection failed:**
```bash
# Проверить ключ:
ssh -i ~/.ssh/key user@server

# Обновить Secret SSH_PRIVATE_KEY
```

**Docker compose fails on server:**
```bash
# SSH на сервер:
ssh user@server
cd /var/www/app
docker-compose logs
```

## 📱 Badges для README

```markdown
![CI](https://github.com/username/repo/workflows/CI/badge.svg)
![Security](https://github.com/username/repo/workflows/Security%20Scan/badge.svg)
```

## 🔗 Полезные ссылки

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [GitHub CLI](https://cli.github.com/)
- [Dependabot](https://docs.github.com/en/code-security/dependabot)

## 📝 Checklist для нового проекта

- [ ] Заменить `YOUR_USERNAME` в файлах
- [ ] Настроить GitHub Secrets для deploy
- [ ] Включить GitHub Actions в Settings > Actions
- [ ] Включить Dependabot в Settings > Security
- [ ] Настроить Branch Protection для `main`
- [ ] Добавить badges в README.md
- [ ] Протестировать CI на test branch
- [ ] Настроить notifications (опционально)
