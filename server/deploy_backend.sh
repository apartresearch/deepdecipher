# Hack to get repo location. This breaks if the server directory is moved.
REPO_PATH=$(dirname $(dirname $(readlink -f $0)))
# Noop to authenticate sudo immediately.
sudo true
# Change to the repo directory.
cd $REPO_PATH
# Build in release mode.
/home/albert/.cargo/bin/cargo build --release
# Copy binary to bin directory.
sudo cp target/release/server /usr/local/bin/deepdecipher/prod/server
# Restart the backend service to use the new binary.
sudo systemctl restart deepdecipher-backend
