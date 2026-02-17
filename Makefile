IMAGE_NAME = datalake-hunter
CONTAINER_NAME = datalake-hunter-rs-container

build:
	docker build -t $(IMAGE_NAME) .

test: build
	docker compose run --rm app cargo test

shell:
	docker compose run --rm app /bin/bash

clean:
	docker rmi $(IMAGE_NAME)