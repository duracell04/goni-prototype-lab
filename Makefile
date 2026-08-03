SHELL := /usr/bin/env bash

COMPOSE_FILE := deploy/docker-compose.yml

.PHONY: up down logs doctor smoke smoke-local demo bench canonical rust test lint

up:
	docker compose -f $(COMPOSE_FILE) up -d

down:
	docker compose -f $(COMPOSE_FILE) down

logs:
	docker compose -f $(COMPOSE_FILE) logs -f

doctor:
	bash scripts/doctor.sh

smoke:
	bash scripts/smoke_test.sh

smoke-local:
	bash scripts/run_smoke_local.sh

demo:
	bash scripts/demo.sh

bench:
	python goni-lab/goni_lab.py bench --scenario goni-lab/scenarios/mixed.json

canonical:
	python scripts/validate_canonical_basis.py

rust:
	cargo test --manifest-path software/kernel/Cargo.toml --workspace --all-features

test: canonical bench rust

lint:
	python scripts/validate_canonical_basis.py
	bash scripts/txt_lint.sh
	cargo fmt --manifest-path software/kernel/Cargo.toml --all -- --check
