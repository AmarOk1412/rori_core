all: keys build run

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

keys:
	mkdir -p keys
	openssl req -x509 -newkey rsa:4096 -nodes -keyout keys/localhost.key -out keys/localhost.crt -days 3650
	openssl pkcs12 -export -out keys/api.p12 -inkey keys/localhost.key -in keys/localhost.crt

test_keys:
	# Used for test/
	rm -rf test_keys
	mkdir -p test_keys
	openssl req -x509 -newkey rsa:4096 -nodes -keyout test_keys/localhost.key -out test_keys/localhost.crt -days 3650  -subj "/C=CA/ST=Quebec/L=Montreal/O=Enconn/CN=enconn.fr"
	openssl pkcs12 -export -out test_keys/api.p12 -inkey test_keys/localhost.key -in test_keys/localhost.crt -passout pass:

docker:
	docker build -t rori_core -f docker/ubuntu_17 .

docker-run:
	docker run -it --rm --privileged -e DISPLAY=$(DISPLAY) --net=host rori_core
