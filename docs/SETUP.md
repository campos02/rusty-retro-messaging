# Setup
R²M requires [Rust](https://www.rust-lang.org/tools/install), [MariaDB](https://mariadb.org/) or [MySQL](https://www.mysql.com/),
and a reverse proxy.

## Ports
Besides the standard HTTPS port, R²M also requires ports 1863 and 1864 to be open and/or forwarded for the MSN clients to work. IPv4 is also required.

## Configuration
After cloning, first run `cp .env.example .env` and edit the .env file to your liking.

## Reverse proxy
In order to set up HTTPS a reverse proxy is required. Below is an example configuration
for Nginx (also including the [website](https://github.com/campos02/r2m-website)), but other software
like Caddy or Apache can be used instead.

### Nginx example
```
server {
    listen 80;
    listen [::]:80;

    server_name r2m.${SERVER_NAME} www.r2m.${SERVER_NAME};
    server_tokens off;

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://r2m.${SERVER_NAME}$request_uri;
    }
}

server {
    listen 443 ssl;
    listen [::]:443 ssl;

    server_name r2m.${SERVER_NAME} www.r2m.${SERVER_NAME};
    server_tokens off;

    #SSL config...

    location ~ ^(/_r2m|/rdr/pprdr.asp|/login.srf|/RST.srf) {
        proxy_pass http://localhost:3000;
    }

    location / {
        proxy_pass http://localhost:4321;
        proxy_set_header Origin http://$host;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## Database
Setting up the database is done with `cargo sqlx database setup`, which will create it
and run all migrations.

## Running
The server can be run with `cargo run` and installed with `cargo install --path .`, which will
place a `rusty-retro-messaging` executable inside your `~/cargo/bin` directory.