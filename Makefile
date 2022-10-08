
include .env

UID = $(shell id -u)
DCRUN = docker-compose run --rm --user $(UID)
AWS_PROFILE=harrison-sandpit

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

publish:
	docker build -f ./src/6_lambda_rust/Dockerfile-x86_64 -t 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-x86_64:latest .
	docker build -f ./src/6_lambda_rust/Dockerfile-arm64 -t 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-arm64:latest .
	docker push 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-x86_64
	docker push 929097612753.dkr.ecr.ap-southeast-2.amazonaws.com/index-tarballs-rust-arm64

enqueue:
	for QUEUE in python-arm64 python-x86_64 rust-arm64 rust-x86_64; do aws sqs send-message-batch --queue-url https://sqs.ap-southeast-2.amazonaws.com/929097612753/index-tarballs-$$QUEUE --entries file://enqueue.json --output text; done;
	
purge:
	for QUEUE in python-arm64 python-x86_64 rust-arm64 rust-x86_64; do aws sqs purge-queue --queue-url https://sqs.ap-southeast-2.amazonaws.com/929097612753/index-tarballs-$$QUEUE; done;
