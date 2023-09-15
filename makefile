# Makefile

# Define the name of your Docker Compose YAML file
COMPOSE_FILE = docker-compose.yml

# Default target
all: up

up:
	docker-compose -f $(COMPOSE_FILE) up -d

down:
	docker-compose -f $(COMPOSE_FILE) down

stop:
	docker-compose -f $(COMPOSE_FILE) stop

logs:
	docker-compose -f $(COMPOSE_FILE) logs -f

test:
	cargo test

.PHONY: all up down stop logs test
