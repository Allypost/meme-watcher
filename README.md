# Meme Watcher

Simple tool to index and display media files (images, videos, ...) from a folder.

## Running

Instructions to run each component are in their respective folders ([frontend](./frontend/), [backend](./backend/)).

A Caddyfile ([Caddyfile.prod](./Caddyfile.prod)) is also to get the app up and running faster. Change it to fit your needs.

## Architecture

Uses a SQLite database to keep all metadata with paths relative to the project folder.
This allows a bit more portability in case an instance is running on a remotely mounted directory or in case the watched folder needs to be moved around (eg. to a new drive or host).

For now it's split into a [rust backend](./backend/) which does all the work (indexing, generating thumbnails, processing files), and a [Next.JS frontend](./frontend/) which only displays data.

Instructions to run the components are in the respective folders.

## Development

Instructions for building/running the components are in their respective folders.
There is also a provided [mprocs](https://github.com/pvolok/mprocs) configuration file to run/monitor everything from one place.

If you use that file you will probably get an error for the Caddy part. To solve it in linux, simply run

```bash
sudo setcap CAP_NET_BIND_SERVICE=+eip "$(which caddy)"
```

That will allow Caddy to bind to "lower" ports (eg. ones below 1024) which is needed for HTTPS on port 443 and HTTP on 80.
