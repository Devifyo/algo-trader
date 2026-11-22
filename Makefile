# algo-trader — operator entrypoints
COMPOSE := docker compose -f infrastructure/docker/docker-compose.yml --env-file .env

.DEFAULT_GOAL := help
.PHONY: help bootstrap build up up-infra up-core up-app down restart logs ps \
        health kill-switch unkill check-rust check-config db-shell redis-cli \
        nats-top backup clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	 awk 'BEGIN{FS=":.*?## "}{printf "  \033[36m%-16s\033[0m %s\n",$$1,$$2}'

bootstrap: ## One-time setup: keys, vault, build
	./scripts/bootstrap.sh

build: ## Build all images
	$(COMPOSE) build

up-infra: ## Start data plane only (postgres, redis, nats, vault)
	$(COMPOSE) up -d postgres redis nats vault

up-core: ## Start trading core (engine + ai + news + portfolio)
	$(COMPOSE) up -d trading-engine ai-engine news-service portfolio-service

up-app: ## Start dashboard + monitoring
	$(COMPOSE) up -d dashboard prometheus grafana loki

up: ## Start everything
	$(COMPOSE) up -d

down: ## Stop everything (keep volumes)
	$(COMPOSE) down

restart: ## Restart all services
	$(COMPOSE) restart

logs: ## Tail logs (make logs S=trading-engine)
	$(COMPOSE) logs -f --tail=200 $(S)

ps: ## List containers
	$(COMPOSE) ps

health: ## Run cross-service health check
	./scripts/healthcheck.sh

kill-switch: ## EMERGENCY: flatten + halt
	./scripts/kill-switch.sh on

unkill: ## Clear the kill switch (resume entries)
	./scripts/kill-switch.sh off

check-config: ## Validate docker-compose
	$(COMPOSE) config -q && echo "compose OK"

check-rust: ## cargo check the engine in a builder container
	docker run --rm -v $(PWD)/services/trading-engine-rust:/app -w /app rust:1-slim \
	  sh -c "cargo check --workspace"

db-shell: ## psql into TimescaleDB
	$(COMPOSE) exec postgres psql -U algotrader -d algotrader

redis-cli: ## redis-cli
	$(COMPOSE) exec redis redis-cli

nats-top: ## NATS monitoring
	@echo "open http://127.0.0.1:$${NATS_MONITOR_PORT:-8134}"

backup: ## Backup the database
	./scripts/backup.sh

clean: ## Stop and REMOVE volumes (DESTROYS DATA)
	$(COMPOSE) down -v
