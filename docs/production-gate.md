# Production Gate

Run the production gate against a running ERPNext bench:

```bash
PINGORA_LISTEN=127.0.0.1:8088 \
PINGORA_WEB_UPSTREAM=127.0.0.1:8000 \
PINGORA_SOCKETIO_UPSTREAM=127.0.0.1:9000 \
PINGORA_SITE_HOST=erpnext.localhost \
./scripts/production-gate.sh
```

The gate verifies:

- Rust tests
- release build
- Pingora health endpoint
- ERPNext web route through Pingora
- Pingora response header
- `/socket.io` polling route
- HTTP load test with ApacheBench when `ab` is installed

Load parameters:

```bash
PINGORA_GATE_REQUESTS=1000 PINGORA_GATE_CONCURRENCY=50 ./scripts/production-gate.sh
```

Passing this gate means the proxy is a production-ready candidate for the tested ERPNext bench. It is not a guarantee for every deployment; TLS, firewall, service supervision, log rotation, and real traffic soak testing still need to be validated in the target environment.
