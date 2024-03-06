# Hack to get repo location. This breaks if the server directory is moved.
REPO_PATH=$(dirname $(dirname $(readlink -f $0)))
# Noop to authenticate sudo immediately.
sudo true
# Change to the repo directory.
cd $REPO_PATH
# Build in release mode.
cargo build --release
# Stop service in order to edit the bin directory.
sudo systemctl stop deepdecipher-backend
# Copy binary to bin directory.
sudo cp target/release/server /usr/local/bin/deepdecipher/prod/server

# Change to frontend directory.
cd frontend
# Install required packages.
npm install
# Build frontend server.
/usr/local/nodejs/bin/npm run build
# Copy module to bin directory.
sudo cp -r . /usr/local/bin/deepdecipher/prod/frontend
# Go back to repo directory.
cd ..

# Update Caddyfile.
sudo cp --backup=numbered -f server/Caddyfile /etc/caddy/Caddyfile
# Reload Caddy with new configuration.
caddy reload --config /etc/caddy/Caddyfile

# Update service files.
sudo cp --backup=numbered -f server/deepdecipher.service /etc/systemd/system/deepdecipher.service
sudo cp --backup=numbered -f server/deepdecipher-backend.service /etc/systemd/system/deepdecipher-backend.service
# Reload services.
sudo systemctl daemon-reload
# Restart services.
sudo systemctl restart deepdecipher
sudo systemctl restart deepdecipher-backend