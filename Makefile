
include .env

UID = $(shell id -u)
DCRUN = docker-compose run --rm --user $(UID)

.DEFAULT_GOAL := help

## init:			terraform init
init: .env
	docker-compose run --rm envvars ensure --tags terraform
	docker-compose run -u $(UID) --rm --workdir /app/tf terraform init

## init-upgrade:		terraform init -upgrade
init-upgrade: .env
	docker-compose run -u $(UID) --rm envvars ensure --tags terraform
	docker-compose run --rm --workdir /app/tf terraform init -migrate-state

## fmt: 			terraform fmt -recursive
fmt: .env
	docker-compose run --rm --workdir /app/tf terraform fmt -recursive

## plan:			terraform plan
plan: .env
	docker-compose run --rm envvars ensure --tags terraform
	docker-compose run --rm --workdir /app/tf terraform plan

## apply:			terraform apply
apply: .env
	docker-compose run --rm envvars ensure --tags terraform
	docker-compose run --rm --workdir /app/tf terraform apply

## destroy:		terraform destroy
destroy: .env
	docker-compose run --rm envvars ensure --tags terraform
	docker-compose run --rm --workdir /app/tf terraform destroy

## .env:			creates .env file with the envvar keys populated
.env:
	touch .env
	docker-compose run --rm envvars envfile --overwrite

## help:			show this help
help:
	@sed -ne '/@sed/!s/## //p' $(MAKEFILE_LIST)

