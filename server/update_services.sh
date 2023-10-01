# Noop to authenticate sudo immediately
sudo true
# Change directory to the repo root
cd /home/albert/deepdecipher
sudo cp --backup=numbered -f server/deepdecipher.service /etc/systemd/system/deepdecipher.service
sudo cp --backup=numbered -f server/deepdecipher-backend.service /etc/systemd/system/deepdecipher-backend.service
sudo systemctl daemon-reload
sudo systemctl restart deepdecipher
sudo systemctl restart deepdecipher-backend

