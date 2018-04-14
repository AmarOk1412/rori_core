all: keys build run

fedora_deps="sudo dnf update \
						 && sudo dnf config-manager --add-repo https://dl.ring.cx/ring-nightly/fedora_27/ring-nightly.repo \
						 && sudo dnf install -y ring dbus-devel openssl dnf-plugins-core rust cargo sqlite ncurses-devel openssl-devel python python3-devel sqlite-devel"
ubuntu_deps="sudo apt-get install -y libdbus-1-dev openssl"
dependencies:
	if [ -f "/etc/redhat-release" ]; then "$(fedora_deps)"; elif [ -f "/etc/debian_version" ]; then "$(ubuntu_deps)"; fi

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

docker:
	docker build -t rori_core -f docker/ubuntu_17 .

docker-run:
	docker run -it --rm --privileged -e DISPLAY=$(DISPLAY) --net=host rori_core
