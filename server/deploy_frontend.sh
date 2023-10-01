# Hack to get repo location. This breaks if the server directory is moved.
REPO_PATH=$(dirname $(dirname $(readlink -f $0)))
# Noop to authenticate sudo immediately.
sudo true
# Change to the repo directory.
cd $REPO_PATH
# Install required packages.
/usr/local/nodejs/bin/npm install
# Build frontend server.
/usr/local/nodejs/bin/npm run build
# Copy module to bin directory.
sudo cp -r . /usr/local/bin/deepdecipher/prod/frontend
# Restart frontend service to use new version.
sudo systemctl restart deepdecipher
