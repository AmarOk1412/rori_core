sudo dnf update -y

# Install ring-daemon
sudo dnf install dnf-plugins-core -y
sudo dnf config-manager --add-repo https://dl.ring.cx/ring-nightly/fedora_27/ring-nightly.repo
sudo dnf install ring -y # TODO replace by ring-daemon but doesn't seems to work

# Install Rust
sudo dnf install rust cargo -y

# Install rori_core dependencies
sudo dnf install sqlite dbus-devel ncurses-devel openssl-devel -y
sudo dnf install python python3-devel sqlite-devel -y # for cargo build
