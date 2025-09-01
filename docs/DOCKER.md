# Setup
To facilitate a Docker setup, this repository comes with a compose file. A reverse proxy is not defined but it is required,
as MSN uses HTTPS in its authentication process.

## Ports
Besides the standard HTTPS port, R²M also requires ports 1863 and 1864 to be open and/or forwarded for the MSN clients to work. IPv4 is also required.

## Configuration
After cloning, first run `cp .env.example .env` to create a .env file and edit it to your liking.

## Reverse proxy
`compose.yaml` can then be included or extended in order put the HTTP endpoints behind a reverse proxy of your choice.
Below is an example configuration for Nginx, which also includes the [website](https://github.com/campos02/r2m-website).

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
        proxy_pass http://r2m:3000;
    }

    location / {
        proxy_pass http://r2m-website:4321;
        proxy_set_header Origin http://$host;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## Running
After everything is set up use `docker compose up -d` to run R²M.