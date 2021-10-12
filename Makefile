PP_BACKEND_API_PATH=$(PWD)/pp_backend_api
PP_BACKEND_API_DOCKER_IMAGE_NAME=pp-backend-api-i
PP_BACKEND_API_DOCKER_CONTAINER_NAME=pp-backend-api-c

PP_STORAGE_DOCKER_VOLUME=pp-storage
PP_STORAGE_DOCKER_CONTAINER_NAME=pp-storage-c
PP_STORAGE_PASSWORD=test01
PP_STORAGE_HOSTNAME=localhost
PP_STORAGE_DATABASE=personalproject

PP_LIB_PATH=$(PWD)/pp_lib

CLI_01_PATH=$(PWD)/cli_01
CLI_02_PATH=$(PWD)/cli_02

TASK_PRODUCER=$(PWD)/task_producer
TASK_CONSUMER=$(PWD)/task_consumer

PP_QUEUE_DOCKER_CONTAINER_NAME=pp-queue-c

DOCKER_PP_NETWORK=dockerppnet

# colors
RED=\033[1;31m
GRN=\033[1;32m
YEL=\033[1;33m
MAG=\033[1;35m
CYN=\033[1;36m
NC=\033[0m

# logging stuff
TIMESTAMP := $$(date "+%Y-%m-%d %H:%M:%S")
LOG_PREFIX=$(CYN)$(TIMESTAMP)$(NC) $(RED)RPP$(NC)


version:
	@echo "$(LOG_PREFIX) $(YEL)Check rustc details...$(NC)"
	which rustc
	rustc --version
	@echo "$(LOG_PREFIX) $(YEL)Check rustup details...$(NC)"
	rustup --version
	@echo "$(LOG_PREFIX) $(YEL)Check cargo details...$(NC)"
	which cargo
	cargo version
	@echo "$(LOG_PREFIX) $(YEL)Check Python details...$(NC)"
	which python
	python -V
	@echo "$(LOG_PREFIX) $(YEL)Check pip details...$(NC)"
	which pip
	pip --version
	@echo "$(LOG_PREFIX) $(YEL)Check Docker details...$(NC)"
	which docker
	docker version
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

setup-python-dependencies:
	@echo "$(LOG_PREFIX) $(YEL)Install Python dependencies...$(NC)"
	pip install -r $(PWD)/bin/requirements.txt
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

setup-rust-env:
	@echo "$(LOG_PREFIX) $(YEL)Setup Rust environment...$(NC)"
	rustup component add rustfmt
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

create-docker-network:
	@echo "$(LOG_PREFIX) $(YEL)Setup Docker network...$(NC)"
	docker network ls
	docker network create \
		--driver=bridge \
		$(DOCKER_PP_NETWORK)
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

docker-ps:
	@echo "$(LOG_PREFIX) $(YEL)See Docker containers...$(NC)"
	docker ps --filter network=$(DOCKER_PP_NETWORK)
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

build-pp-backend-api:
	@echo "$(LOG_PREFIX) $(YEL)Build Docker image for the Rust Backend API...$(NC)"
	docker build \
		--tag $(PP_BACKEND_API_DOCKER_IMAGE_NAME) \
		--file backend.Dockerfile .
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

# cd $(PP_BACKEND_API_PATH) && RUST_BACKTRACE=1 cargo run
# troubleshoot:
# `docker run -it pp-pp_backend_api-i sh`
run-pp-backend-api:
	@echo "$(LOG_PREFIX) $(YEL)Run Docker container for the Rust Backend API...$(NC)"
	-docker stop $(PP_BACKEND_API_DOCKER_CONTAINER_NAME)
	-docker rm $(PP_BACKEND_API_DOCKER_CONTAINER_NAME)
	docker run -d \
		--name=$(PP_BACKEND_API_DOCKER_CONTAINER_NAME) \
		-p 3000:3000 \
		-e DOCKER_DB_HOST=$(PP_STORAGE_DOCKER_CONTAINER_NAME) \
		--net=$(DOCKER_PP_NETWORK) \
		$(PP_BACKEND_API_DOCKER_IMAGE_NAME)
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

backend-logs:
	@echo "$(LOG_PREFIX) $(YEL)Show the Backend API logs...$(NC)"
	docker logs $(PP_BACKEND_API_DOCKER_CONTAINER_NAME)
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

create-db-docker-volume:
	@echo "$(LOG_PREFIX) $(YEL)Setup Docker volume for the Postgres DB...$(NC)"
	docker volume ls
	docker volume create $(PP_STORAGE_DOCKER_VOLUME)
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

run-pp-storage:
	@echo "$(LOG_PREFIX) $(YEL)Run Docker container for the Postgres DB...$(NC)"
	docker run -d \
		--name=$(PP_STORAGE_DOCKER_CONTAINER_NAME) \
		-p 5432:5432 \
		-v $(PP_STORAGE_DOCKER_VOLUME):/var/lib/postgresql/data \
		-e POSTGRES_PASSWORD=$(PP_STORAGE_PASSWORD) \
		--net=$(DOCKER_PP_NETWORK) \
		postgres:12.6-alpine
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

test-pp-storage-connection:
	@echo "$(LOG_PREFIX) $(YEL)Test the DB connection...$(NC)"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "SELECT datname, datcollate, datctype FROM pg_database"
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

# sudo apt install postgresql-client-common postgresql-client-12
connect-pp-storage:
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME)

create-db:
	@echo "$(LOG_PREFIX) $(YEL)Drop the DB...$(NC)"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "DROP DATABASE IF EXISTS $(PP_STORAGE_DATABASE)"
	@echo "$(LOG_PREFIX) $(YEL)Create the DB...$(NC)"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "CREATE DATABASE $(PP_STORAGE_DATABASE);"
	@echo "$(LOG_PREFIX) $(YEL)Configure the DB...$(NC)"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "ALTER DATABASE $(PP_STORAGE_DATABASE) SET log_statement = 'all';"
	@echo "$(LOG_PREFIX) $(YEL)Setup schema and tables for the DB...$(NC)"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-d $(PP_STORAGE_DATABASE) \
		-a \
		-f $(PWD)/pp_storage/schema_handmade.sql
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

logs-pp-storage:
	docker logs -f $(PP_STORAGE_DOCKER_CONTAINER_NAME)

# to block the Rust linter and show the changes to apply, add: `-- --check`
# TODO: rust linter - https://github.com/rust-lang/rust-clippy
lint:
	@echo "$(LOG_PREFIX) $(YEL)Lint (format and check syntax errors) Rust land...$(NC)"
	@for RUST_PROJECT in \
		$(PP_BACKEND_API_PATH) \
		$(PP_LIB_PATH) \
		$(CLI_01_PATH) \
		$(CLI_02_PATH) \
		$(TASK_PRODUCER) \
		$(TASK_CONSUMER) \
	; do \
		echo "Processing project: $${RUST_PROJECT}" && \
		cd $${RUST_PROJECT} && \
		cargo fmt --all && \
		echo "DONE processing project: $${RUST_PROJECT}" \
		|| exit 1 ; \
	done
	@echo "$(LOG_PREFIX) $(YEL)Lint and format Python land...$(NC)"
	@for PY_FILE in \
		bin/http_integration_tests.py \
		bin/amqp_setup.py \
	; do \
		echo "Processing file: $${PY_FILE}" && \
		black $${PY_FILE} && \
		pylint \
			--disable=C0114,C0115,C0116,W1203,W0511 \
			$${PY_FILE} && \
		echo "DONE with file: $${PY_FILE}" \
		|| exit 1 ; \
	done
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

all-rust-tests:
	@echo "$(LOG_PREFIX) $(YEL)Run all Rust tests...$(NC)"
	cd $(PP_LIB_PATH) && RUST_BACKTRACE=1 cargo test --no-fail-fast
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

http-integration-test:
	@echo "$(LOG_PREFIX) $(YEL)Run HTTP integration tests for the Backend REST API...$(NC)"
	python bin/http_integration_tests.py
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

queue-test:
	@echo "$(LOG_PREFIX) $(YEL)Run the queue (AMQP) integration tests for the Rust code...$(NC)"
	cd $(PP_LIB_PATH) && \
		RUST_BACKTRACE=1 cargo test queue_tests::test_queue -- --exact
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

db-test:
	@echo "$(LOG_PREFIX) $(YEL)Run the DB integration tests for the Rust code...$(NC)"
	cd $(PP_LIB_PATH) && \
		RUST_BACKTRACE=1 cargo test test_crud -- --show-output
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

cli-run-01:
	@echo "$(LOG_PREFIX) $(YEL)Invoke the CLI_O1 in various ways...$(NC)"
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- -h
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --id 1 --call-type http
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --work-code apiLvMrVyULP3 --call-type http
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --id 1 --call-type db
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --work-code apiLvMrVyULP3 --call-type db
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

cli-run-02:
	@echo "$(LOG_PREFIX) $(YEL)Invoke the CLI_O2 for the TCP socket and handmade HTTP request...$(NC)"
	cd $(CLI_02_PATH) && RUST_BACKTRACE=1 cargo run
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

# management dashboard: 
# - URL: http://localhost:15672/
# - credentials: guest, guest
#
# https://www.rabbitmq.com/tutorials/tutorial-one-python.html
# TODO https://www.rabbitmq.com/confirms.html (message acknowledgments)
# TODO https://www.rabbitmq.com/production-checklist.html
# TODO https://www.rabbitmq.com/queues.html
#
run-pp-queue:
	@echo "$(LOG_PREFIX) $(YEL)Run the RabbitMQ AMQP Docker container...$(NC)"
	docker run -d \
		--name $(PP_QUEUE_DOCKER_CONTAINER_NAME) \
		-p 5672:5672 \
		-p 15672:15672 \
		--net=$(DOCKER_PP_NETWORK) \
		rabbitmq:3.9.5-management-alpine
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

setup-pp-queue:
	@echo "$(LOG_PREFIX) $(YEL)Configure the AMQP RabbitMQ queue...$(NC)"
	python bin/amqp_setup.py
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

# run-task-producer:
# 	cd $(TASK_PRODUCER) && RUST_BACKTRACE=1 cargo run
#
# run-task-consumer:
# 	cd $(TASK_CONSUMER) && RUST_BACKTRACE=1 cargo run

build-task-producer:
	@echo "$(LOG_PREFIX) $(YEL)Build the Task Producer...$(NC)"
	cd $(TASK_PRODUCER) && \
		cargo build --release
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

run-task-producer: build-task-producer
	cd $(TASK_PRODUCER) && \
		./target/release/task_producer

build-task-consumer:
	@echo "$(LOG_PREFIX) $(YEL)Build the Task Consumer...$(NC)"
	cd $(TASK_CONSUMER) && \
		cargo build --release
	@echo "$(LOG_PREFIX) $(GRN)DONE$(NC)"

run-task-consumer: build-task-consumer
	cd $(TASK_CONSUMER) && \
		./target/release/task_consumer
