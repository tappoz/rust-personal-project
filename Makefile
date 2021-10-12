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

version:
	which rustc
	rustc --version
	rustup --version
	which python
	python -V

setup-python-dependencies:
	pip install -r $(PWD)/bin/requirements.txt

create-docker-network:
	docker network ls
	docker network create \
		--driver=bridge \
		$(DOCKER_PP_NETWORK)

docker-ps:
	docker ps --filter network=$(DOCKER_PP_NETWORK)

build-pp-backend-api:
		docker build --tag $(PP_BACKEND_API_DOCKER_IMAGE_NAME) --file backend.Dockerfile .

# cd $(PP_BACKEND_API_PATH) && RUST_BACKTRACE=1 cargo run
# troubleshoot:
# `docker run -it pp-pp_backend_api-i sh`
run-pp-backend-api:
	-docker stop $(PP_BACKEND_API_DOCKER_CONTAINER_NAME)
	-docker rm $(PP_BACKEND_API_DOCKER_CONTAINER_NAME)
	docker run -d \
		--name=$(PP_BACKEND_API_DOCKER_CONTAINER_NAME) \
		-p 3000:3000 \
		-e DOCKER_DB_HOST=$(PP_STORAGE_DOCKER_CONTAINER_NAME) \
		--net=$(DOCKER_PP_NETWORK) \
		$(PP_BACKEND_API_DOCKER_IMAGE_NAME)

create-db-docker-volume:
	docker volume ls
	docker volume create $(PP_STORAGE_DOCKER_VOLUME)

run-pp-storage:
	docker run -d \
		--name=$(PP_STORAGE_DOCKER_CONTAINER_NAME) \
		-p 5432:5432 \
		-v $(PP_STORAGE_DOCKER_VOLUME):/var/lib/postgresql/data \
		-e POSTGRES_PASSWORD=$(PP_STORAGE_PASSWORD) \
		--net=$(DOCKER_PP_NETWORK) \
		postgres:12.6-alpine

test-pp-storage-connection:
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "SELECT datname, datcollate, datctype FROM pg_database"

# sudo apt install postgresql-client-common postgresql-client-12
connect-pp-storage:
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql -U postgres -h $(PP_STORAGE_HOSTNAME)

# SELECT * 
# FROM pg_database
create-db:
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "DROP DATABASE IF EXISTS $(PP_STORAGE_DATABASE)"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "CREATE DATABASE $(PP_STORAGE_DATABASE);"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-c "ALTER DATABASE $(PP_STORAGE_DATABASE) SET log_statement = 'all';"
	PGPASSWORD=$(PP_STORAGE_PASSWORD) psql \
		-U postgres \
		-h $(PP_STORAGE_HOSTNAME) \
		-d $(PP_STORAGE_DATABASE) \
		-a \
		-f $(PWD)/pp_storage/schema_handmade.sql

logs-pp-storage:
	docker logs -f $(PP_STORAGE_DOCKER_CONTAINER_NAME)

# to block the Rust linter and show the changes to apply, add: `-- --check`
# TODO: rust linter - https://github.com/rust-lang/rust-clippy
lint:
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

integration-test:
	cd $(PP_LIB_PATH) && RUST_BACKTRACE=1 cargo test --no-fail-fast
	python bin/http_integration_tests.py

queue-test:
	cd $(PP_LIB_PATH) && \
		RUST_BACKTRACE=1 cargo test queue_tests::test_queue -- --exact

db-test:
	cd $(PP_LIB_PATH) && \
		RUST_BACKTRACE=1 cargo test test_crud -- --show-output

cli-run-01:
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- -h
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --id 1 --call-type http
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --work-code apiLvMrVyULP3 --call-type http
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --id 1 --call-type db
	cd $(CLI_01_PATH) && RUST_BACKTRACE=1 cargo run -- --work-code apiLvMrVyULP3 --call-type db

cli-run-02:
	cd $(CLI_02_PATH) && RUST_BACKTRACE=1 cargo run

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
	docker run -d \
		--name $(PP_QUEUE_DOCKER_CONTAINER_NAME) \
		-p 5672:5672 \
		-p 15672:15672 \
		--net=$(DOCKER_PP_NETWORK) \
		rabbitmq:3.9.5-management-alpine

setup-pp-queue:
	python bin/amqp_setup.py

# run-task-producer:
# 	cd $(TASK_PRODUCER) && RUST_BACKTRACE=1 cargo run
#
# run-task-consumer:
# 	cd $(TASK_CONSUMER) && RUST_BACKTRACE=1 cargo run

build-task-producer:
	cd $(TASK_PRODUCER) && \
		cargo build --release

run-task-producer: build-task-producer
	cd $(TASK_PRODUCER) && \
		./target/release/task_producer

build-task-consumer:
	cd $(TASK_CONSUMER) && \
		cargo build --release

run-task-consumer: build-task-consumer
	cd $(TASK_CONSUMER) && \
		./target/release/task_consumer
