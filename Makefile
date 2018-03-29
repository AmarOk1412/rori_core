all: build run

# TODO scripts to handle dependencies and install ring-daemon
fedora_deps="sudo dnf install -y dbus-devel"
ubuntu_deps="sudo apt-get install -y libdbus-1-dev"
dependencies:
	if [ -f "/etc/redhat-release" ]; then "$(fedora_deps)"; elif [ -f "/etc/debian_version" ]; then "$(ubuntu_deps)"; fi

clean-db:
	rm -rf rori.db

build:
	cargo build

run:
	RUST_BACKTRACE=1 RUST_LOG=info cargo run

test:
	RUST_TEST_THREADS=1 cargo test -- --nocapture
