all: build run

dependencies:
	if [ -f "/etc/redhat-release" ]; then "./scripts/build_fedora.sh"; elif [ -f "/etc/debian_version" ]; then "./scripts/build_ubuntu.sh"; fi

clean-db:
	rm -rf rori.db

build:
	cargo build

run:
	RUST_BACKTRACE=1 RUST_LOG=info cargo run

test:
	RUST_TEST_THREADS=1 RUST_BACKTRACE=1 cargo test -- --nocapture

docker:
	docker build -t rori_core -f docker/ubuntu_17 .

docker-run:
	docker run -it --rm --privileged -e DISPLAY=$(DISPLAY) --net=host rori_core
