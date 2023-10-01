# Hack to get repo location. This breaks if the server directory is moved.
REPO_PATH=$(dirname $(dirname $(readlink -f $0)))
# Noop to authenticate sudo immediately.
sudo true
# Change to the repo directory.
cd $REPO_PATH
# Update Caddyfile.
sudo cp --backup=numbered -f server/Caddyfile /etc/caddy/Caddyfile
# Reload Caddy with new configuration.
caddy reload --config /etc/caddy/Caddyfile
