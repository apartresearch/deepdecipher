# Noop to authenticate sudo immediately
sudo true
# Change to the frontend directory in the repo
cd /home/albert/deepdecipher
# Update Caddyfile
sudo cp --backup=numbered -f server/Caddyfile /etc/caddy/Caddyfile
# Reload Caddy with new configuration
caddy reload --config /etc/caddy/Caddyfile
