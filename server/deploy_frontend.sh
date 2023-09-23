# Noop to authenticate sudo.
sudo true
cd /home/albert/deepdecipher/frontend
/usr/local/nodejs/bin/npm install
/usr/local/nodejs/bin/npm run build
sudo systemctl restart deepdecipher
