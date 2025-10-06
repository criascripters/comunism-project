.PHONY: help setup-dev setup-prod dev-up dev-down dev-restart prod-up prod-down prod-restart logs logs-backend logs-frontend logs-db clean rebuild-dev rebuild-prod status

# Default target
help:
	@echo "==================================="
	@echo "Comunism Project - Makefile Commands"
	@echo "==================================="
	@echo ""
	@echo "Setup Commands:"
	@echo "  make setup-dev          - Setup development environment"
	@echo "  make setup-prod         - Setup production environment"
	@echo ""
	@echo "Development Commands:"
	@echo "  make dev-up             - Start development environment"
	@echo "  make dev-down           - Stop development environment"
	@echo "  make dev-restart        - Restart development environment"
	@echo "  make rebuild-dev        - Rebuild and start development environment"
	@echo ""
	@echo "Production Commands:"
	@echo "  make prod-up            - Start production environment"
	@echo "  make prod-down          - Stop production environment"
	@echo "  make prod-restart       - Restart production environment"
	@echo "  make rebuild-prod       - Rebuild and start production environment"
	@echo ""
	@echo "Monitoring Commands:"
	@echo "  make status             - Show running containers status"
	@echo "  make logs               - Show all logs (development)"
	@echo "  make logs-backend       - Show backend logs (development)"
	@echo "  make logs-frontend      - Show frontend logs (development)"
	@echo "  make logs-db            - Show database logs (development)"
	@echo "  make logs-prod          - Show all logs (production)"
	@echo ""
	@echo "Maintenance Commands:"
	@echo "  make clean              - Stop all containers and remove volumes"
	@echo "  make clean-all          - Clean everything including images"
	@echo ""

# Setup Commands
setup-dev:
	@echo "Setting up development environment..."
	@if not exist "backend\.env.development" copy "backend\.env.development.example" "backend\.env.development" 2>nul || echo "backend/.env.development already exists"
	@if not exist "frontend\.env.development" copy "frontend\.env.development.example" "frontend\.env.development" 2>nul || echo "frontend/.env.development already exists"
	@echo "Development environment setup complete!"
	@echo "Edit the .env.development files if needed, then run: make dev-up"

setup-prod:
	@echo "Setting up production environment..."
	@if not exist "backend\.env.production" copy "backend\.env.production.example" "backend\.env.production" 2>nul || echo "backend/.env.production already exists"
	@if not exist "frontend\.env.production" copy "frontend\.env.production.example" "frontend\.env.production" 2>nul || echo "frontend/.env.production already exists"
	@echo "Production environment setup complete!"
	@echo "IMPORTANT: Edit the .env.production files with secure values!"
	@echo "Then run: make prod-up"

# Development Commands
dev-up:
	@echo "Starting development environment..."
	docker-compose -f docker/docker-compose.dev.yml up -d
	@echo ""
	@echo "Development environment is starting!"
	@echo "Frontend: http://localhost:3000"
	@echo "Backend:  http://localhost:5122"
	@echo "Database: localhost:5432"
	@echo ""
	@echo "To see logs: make logs"

dev-down:
	@echo "Stopping development environment..."
	docker-compose -f docker/docker-compose.dev.yml down
	@echo "Development environment stopped!"

dev-restart:
	@echo "Restarting development environment..."
	docker-compose -f docker/docker-compose.dev.yml restart
	@echo "Development environment restarted!"

rebuild-dev:
	@echo "Rebuilding development environment..."
	docker-compose -f docker/docker-compose.dev.yml down
	docker-compose -f docker/docker-compose.dev.yml build --no-cache
	docker-compose -f docker/docker-compose.dev.yml up -d
	@echo "Development environment rebuilt and started!"

# Production Commands
prod-up:
	@echo "Starting production environment..."
	docker-compose -f docker/docker-compose.prod.yml up -d
	@echo ""
	@echo "Production environment is starting!"
	@echo "Frontend: http://localhost:3000"
	@echo "Backend:  http://localhost:5122"
	@echo ""
	@echo "To see logs: make logs-prod"

prod-down:
	@echo "Stopping production environment..."
	docker-compose -f docker/docker-compose.prod.yml down
	@echo "Production environment stopped!"

prod-restart:
	@echo "Restarting production environment..."
	docker-compose -f docker/docker-compose.prod.yml restart
	@echo "Production environment restarted!"

rebuild-prod:
	@echo "Rebuilding production environment..."
	docker-compose -f docker/docker-compose.prod.yml down
	docker-compose -f docker/docker-compose.prod.yml build --no-cache
	docker-compose -f docker/docker-compose.prod.yml up -d
	@echo "Production environment rebuilt and started!"

# Logs Commands
logs:
	docker-compose -f docker/docker-compose.dev.yml logs -f

logs-backend:
	docker-compose -f docker/docker-compose.dev.yml logs -f backend

logs-frontend:
	docker-compose -f docker/docker-compose.dev.yml logs -f frontend

logs-db:
	docker-compose -f docker/docker-compose.dev.yml logs -f db

logs-prod:
	docker-compose -f docker/docker-compose.prod.yml logs -f

# Monitoring Commands
status:
	@echo "==================================="
	@echo "Container Status"
	@echo "==================================="
	@docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
	@echo ""

# Maintenance Commands
clean:
	@echo "Cleaning up development environment..."
	docker-compose -f docker/docker-compose.dev.yml down -v
	@echo "Cleaning up production environment..."
	docker-compose -f docker/docker-compose.prod.yml down -v
	@echo "All environments cleaned!"

clean-all: clean
	@echo "Removing all Docker images..."
	docker system prune -a -f
	@echo "All Docker resources cleaned!"
