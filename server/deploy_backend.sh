# Noop to authenticate sudo immediately
sudo true
# Change to the Rust project directory.
cd /home/albert/deepdecipher
# Build in release mode.
/home/albert/.cargo/bin/cargo build --release
# Copy binary to bin directory.
sudo cp target/release/server /usr/local/bin/deepdecipher/prod/server
# Restart the backend service to use the new binary.
sudo systemctl restart deepdecipher-backend
