# Noop to authenticate sudo immediately
sudo true
# Change to the frontend directory in the repo
cd /home/albert/deepdecipher/frontend
# Install required packages
/usr/local/nodejs/bin/npm install
# Build frontend server
/usr/local/nodejs/bin/npm run build
# Copy module to bin directory
sudo cp -r . /usr/local/bin/deepdecipher/prod/frontend
# Restart frontend service to use new version
sudo systemctl restart deepdecipher
