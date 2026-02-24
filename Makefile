.PHONY: help setup dev build test lint clean deploy docker-build docker-up docker-down ssl-dev ssl-prod

# Default target
.DEFAULT_GOAL := help

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
RESET := \033[0m

help: ## Показать это сообщение помощи
	@echo "$(CYAN)DJ Arbuzzz - Доступные команды:$(RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""

# Setup
setup: ## Настройка проекта (установка зависимостей)
	@echo "$(CYAN)Настройка проекта...$(RESET)"
	@[ -f .env ] || (cp .env.example .env && echo "$(GREEN)✓ Создан .env файл$(RESET)")
	@cd server && cargo check && echo "$(GREEN)✓ Backend зависимости установлены$(RESET)"
	@cd client && pnpm install && echo "$(GREEN)✓ Frontend зависимости установлены$(RESET)"
	@chmod +x scripts/*.sh
	@echo "$(GREEN)✓ Проект настроен!$(RESET)"

# Development
dev-backend: ## Запустить backend в режиме разработки
	@echo "$(CYAN)Запуск backend...$(RESET)"
	@cd server && cargo watch -x run

dev-frontend: ## Запустить frontend в режиме разработки
	@echo "$(CYAN)Запуск frontend...$(RESET)"
	@cd client && pnpm dev

dev-infra: ## Запустить только инфраструктуру (postgres, redis)
	@echo "$(CYAN)Запуск инфраструктуры...$(RESET)"
	@docker-compose up -d postgres redis
	@echo "$(GREEN)✓ Инфраструктура запущена$(RESET)"

dev: ## Запустить весь проект в режиме разработки
	@echo "$(CYAN)Запуск всего проекта...$(RESET)"
	@$(MAKE) ssl-dev
	@docker-compose up -d
	@echo "$(GREEN)✓ Проект запущен на https://localhost$(RESET)"

# Build
build-backend: ## Собрать backend
	@echo "$(CYAN)Сборка backend...$(RESET)"
	@cd server && cargo build --release
	@echo "$(GREEN)✓ Backend собран$(RESET)"

build-frontend: ## Собрать frontend
	@echo "$(CYAN)Сборка frontend...$(RESET)"
	@cd client && pnpm run build
	@echo "$(GREEN)✓ Frontend собран$(RESET)"

build: build-backend build-frontend ## Собрать весь проект


# Linting
lint-backend: ## Проверить код backend (fmt + clippy)
	@echo "$(CYAN)Проверка backend...$(RESET)"
	@cd server && cargo fmt --check && cargo clippy -- -D warnings
	@echo "$(GREEN)✓ Backend код в порядке$(RESET)"

lint-frontend: ## Проверить код frontend (eslint)
	@echo "$(CYAN)Проверка frontend...$(RESET)"
	@cd client && pnpm run lint
	@echo "$(GREEN)✓ Frontend код в порядке$(RESET)"

lint: lint-backend lint-frontend ## Проверить весь код

fix-backend: ## Исправить форматирование backend
	@echo "$(CYAN)Исправление backend...$(RESET)"
	@cd server && cargo fmt

fix-frontend: ## Исправить форматирование frontend
	@echo "$(CYAN)Исправление frontend...$(RESET)"
	@cd client && pnpm run lint:fix

fix: fix-backend fix-frontend ## Исправить форматирование

# Docker
docker-build: ## Собрать Docker образы
	@echo "$(CYAN)Сборка Docker образов...$(RESET)"
	@docker-compose build
	@echo "$(GREEN)✓ Образы собраны$(RESET)"

docker-up: ## Запустить Docker контейнеры
	@echo "$(CYAN)Запуск контейнеров...$(RESET)"
	@docker-compose up -d
	@echo "$(GREEN)✓ Контейнеры запущены$(RESET)"
	@$(MAKE) docker-status

docker-down: ## Остановить Docker контейнеры
	@echo "$(CYAN)Остановка контейнеров...$(RESET)"
	@docker-compose down
	@echo "$(GREEN)✓ Контейнеры остановлены$(RESET)"

docker-restart: docker-down docker-up ## Перезапустить Docker контейнеры

docker-logs: ## Показать логи Docker контейнеров
	@docker-compose logs -f

docker-status: ## Показать статус Docker контейнеров
	@echo "$(CYAN)Статус контейнеров:$(RESET)"
	@docker-compose ps

docker-clean: ## Удалить Docker контейнеры и volumes
	@echo "$(YELLOW)Удаление контейнеров и данных...$(RESET)"
	@docker-compose down -v
	@echo "$(GREEN)✓ Очистка завершена$(RESET)"

# SSL
ssl-dev: ## Создать самоподписанные SSL сертификаты для разработки
	@echo "$(CYAN)Генерация SSL сертификатов...$(RESET)"
	@./scripts/generate-ssl.sh

ssl-prod: ## Настроить Let's Encrypt для production (требуются аргументы: domain, email)
	@echo "$(CYAN)Настройка Let's Encrypt...$(RESET)"
	@if [ -z "$(domain)" ] || [ -z "$(email)" ]; then \
		echo "$(RED)Ошибка: Укажите domain и email$(RESET)"; \
		echo "Использование: make ssl-prod domain=example.com email=admin@example.com"; \
		exit 1; \
	fi
	@./scripts/init-letsencrypt.sh $(domain) $(email)

# Database
db-migrate: ## Запустить миграции базы данных
	@echo "$(CYAN)Запуск миграций...$(RESET)"
	@cd server && diesel migration run
	@echo "$(GREEN)✓ Миграции выполнены$(RESET)"

db-reset: ## Сбросить базу данных
	@echo "$(YELLOW)Сброс базы данных...$(RESET)"
	@cd server && diesel database reset
	@echo "$(GREEN)✓ База данных сброшена$(RESET)"

# Deployment
deploy-check: ## Проверка готовности к деплою
	@echo "$(CYAN)Проверка проекта...$(RESET)"
	@$(MAKE) lint
	@$(MAKE) docker-build
	@echo "$(GREEN)✓ Проект готов к деплою$(RESET)"

deploy: deploy-check ## Деплой на production (через GitHub Actions)
	@echo "$(CYAN)Запуск деплоя...$(RESET)"
	@echo "$(YELLOW)Убедитесь, что все изменения закоммичены и запушены$(RESET)"
	@echo "Создайте тег: git tag v1.0.0 && git push origin v1.0.0"

# Cleaning
clean-backend: ## Очистить артефакты сборки backend
	@echo "$(CYAN)Очистка backend...$(RESET)"
	@cd server && cargo clean

clean-frontend: ## Очистить артефакты сборки frontend
	@echo "$(CYAN)Очистка frontend...$(RESET)"
	@cd client && rm -rf node_modules .output .nuxt

clean: clean-backend clean-frontend ## Очистить все артефакты сборки
	@rm -rf nginx/ssl/*.pem
	@rm -rf certbot/
	@echo "$(GREEN)✓ Очистка завершена$(RESET)"

# Info
info: ## Показать информацию о проекте
	@echo "$(CYAN)Информация о проекте:$(RESET)"
	@echo ""
	@echo "$(GREEN)Backend:$(RESET)"
	@cd server && cargo --version
	@echo ""
	@echo "$(GREEN)Frontend:$(RESET)"
	@cd client && node --version && pnpm --version
	@echo ""
	@echo "$(GREEN)Docker:$(RESET)"
	@docker --version
	@docker-compose --version
	@echo ""
