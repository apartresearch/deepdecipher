# Hack to get repo location. This breaks if the server directory is moved.
REPO_PATH=$(dirname $(dirname $(readlink -f $0)))
# Noop to authenticate sudo immediately.
sudo true
# Change to the repo directory.
cd $REPO_PATH
# Update service files.
sudo cp --backup=numbered -f server/deepdecipher.service /etc/systemd/system/deepdecipher.service
sudo cp --backup=numbered -f server/deepdecipher-backend.service /etc/systemd/system/deepdecipher-backend.service
# Reload services.
sudo systemctl daemon-reload
# Restart services.
sudo systemctl restart deepdecipher
sudo systemctl restart deepdecipher-backend

