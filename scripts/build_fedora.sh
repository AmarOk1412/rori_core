sudo dnf update -y

# Install ring-daemon
sudo dnf install dnf-plugins-core -y
sudo dnf config-manager --add-repo https://dl.ring.cx/ring-nightly/fedora_$(rpm -E %fedora)/ring-nightly.repo
sudo dnf install ring -y # TODO replace by ring-daemon but doesn't seems to work
sudo dnf install qt5-devel qt5-qtdeclarative qt5-qtdeclarative-devel qt5-qtbase qt5-qtbase-devel qt5-qtgraphicaleffects -y

# Install Rust
sudo dnf install rust cargo -y

# Install rori_core dependencies
sudo dnf install sqlite dbus-devel ncurses-devel openssl-devel -y
sudo dnf install python python3-devel sqlite-devel -y # for cargo build

# Install rori_modules dependencies
sudo dnf -y install python3-pip
pip3 install wikipedia --user
pip3 install geocoder --user
