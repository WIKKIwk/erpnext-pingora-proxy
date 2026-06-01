# ERPNext Pingora Proxy

Experimental Pingora reverse proxy profile for a local or production-style ERPNext/Frappe bench.

This project routes ERPNext traffic like this:

```text
client -> Pingora
  /socket.io -> ERPNext realtime/socket.io upstream
  everything else -> ERPNext web upstream
```

It is intended as a second reverse-proxy option beside the common nginx setup. It is not an official ERPNext, Frappe, Pingora, or Cloudflare project.

## Features

- Pingora-based HTTP reverse proxy
- ERPNext site host forwarding
- `/socket.io` routing to the realtime service
- `X-Forwarded-*`, `X-Real-IP`, and `X-Frappe-Site-Name` upstream headers
- Health endpoint at `/_pingora_health`
- Configurable via environment variables

## Requirements

- Rust toolchain
- Running ERPNext/Frappe bench
- ERPNext web server, usually on `127.0.0.1:8000`
- ERPNext realtime/socket.io server, usually on `127.0.0.1:9000`

## Configuration

Copy the example env file:

```bash
cp .env.example .env
```

Default values:

```bash
PINGORA_LISTEN=127.0.0.1:8088
PINGORA_WEB_UPSTREAM=127.0.0.1:8000
PINGORA_SOCKETIO_UPSTREAM=127.0.0.1:9000
PINGORA_SITE_HOST=erpnext.localhost
PINGORA_FORWARDED_PROTO=http
RUST_LOG=info
```

## Run

Development:

```bash
RUST_LOG=info cargo run
```

Release build:

```bash
cargo build --release
```

With the helper script:

```bash
scripts/start-prod.sh
scripts/stop-prod.sh
```

Then open:

```text
http://127.0.0.1:8088
```

Health check:

```bash
curl http://127.0.0.1:8088/_pingora_health
```

## Production Gate

Run the repeatable production gate:

```bash
./scripts/production-gate.sh
```

See `docs/production-gate.md` for load-test knobs and release criteria.

## Notes

This is experimental. For public production use, add and verify TLS, domain configuration, service supervision, log rotation, firewall rules, and ERPNext production process management.

Pingora is an open-source project by Cloudflare. See the Pingora project and its license before adopting this in production.
