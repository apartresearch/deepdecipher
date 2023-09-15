sudo cp --backup=numbered -f deepdecipher.service /etc/systemd/system/deepdecipher.service;
sudo cp --backup=numbered -f deepdecipher-backend.service /etc/systemd/system/deepdecipher-backend.service;
sudo systemctl daemon-reload;
sudo systemctl restart deepdecipher;
sudo systemctl restart deepdecipher-backend;

