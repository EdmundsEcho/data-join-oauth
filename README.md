# Projects

## bin oauth

Part of the suite of micro-services.

### To run the bin locally - bare metal

set -x RUST_ENV Development; set -x RUST_LOG 'oauth=trace,hyper=info'; cargo watch -x 'run --release -p oauth';

### To run within docker-compose

Github generates a compiled package

- set the image source location to
- login to ghcr.io with docker
- docker-compose: add the following to the manifest

```
  #
  # 📥 to force a fresh pull,
  #    call docker-compose pull auth-service
  #
  auth-service:
    hostname: auth-service
    image: ghcr.io/lucivia/auth-img-dev:latest
    ports:
     - "3099:3099"
```

### Configure where the service interacts with tnc system

There are three configuration environments:

- `RUST_ENV=production` -> `config/default.toml`
- `RUST_ENV=development` -> `config/development.toml`
- `RUST_ENV=test` -> `config/test.toml`

To update where the service is expected to interact with the back-end and where redirect the user-agent:

- update the appropriate configuration file

### Working with a docker container

- Copy the current settings into the host (local machine)

```
> docker cp <container-id>:/app/config/development.toml <where-on-your-computer>
```

- Once updated, copy the file to the "live" container, and hit the auth-service/reload endpoint (development only)

```
> docker cp axum_config/development.toml <container-id>:/app/config/development.toml
> curl http://localhost:3099/reload
```

- The docker-compose logs will echo the newly configured status of the service

_last updated: June 19th, 2022_
