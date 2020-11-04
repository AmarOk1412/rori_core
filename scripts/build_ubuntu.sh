sudo apt-get update -y
# Install ring-daemon
sudo apt-get install gnupg2 dirmngr software-properties-common -y
sudo sh -c "echo 'deb https://dl.jami.net/nightly/ubuntu_20.04/ ring main' > /etc/apt/sources.list.d/jami-nightly-main.list"
sudo apt-key adv --keyserver pgp.mit.edu --recv-keys A295D773307D25A33AE72F2F64CD5FA175348F84
sudo add-apt-repository universe
sudo apt-get update -y && apt-get install ring -y

# Install Rust
sudo apt-get install curl sudo -y
curl -sSf https://static.rust-lang.org/rustup.sh | sh

# Install rori_core dependencies
sudo apt-get install sqlite3 libsqlite3-dev libncurses5-dev libssl-dev -y
sudo apt-get install python libpython3.6-dev libsqlite3-dev libdbus-1-dev dbus-x11 -y # for cargo build

sudo apt-get install build-essential -y

# Install rori_modules dependencies
sudo apt-get -y install python3-pip
pip3 install wikipedia feedparser appdirs --user
