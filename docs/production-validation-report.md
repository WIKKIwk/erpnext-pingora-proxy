# Production Validation Report

Validation date: 2026-06-01.

## Scope

This report covers the Pingora reverse proxy in front of a local ERPNext/Frappe bench.

Validated traffic paths:

- `/_pingora_health` served by Pingora
- `/` routed to ERPNext web upstream
- `/socket.io/?EIO=4&transport=polling` routed to ERPNext socket.io upstream

## Environment

See `docs/tested-matrix.md`.

## Results

| Check | Result |
| --- | --- |
| Rust tests | Passed |
| Release build | Passed |
| Pingora health endpoint | Passed |
| ERPNext web route through Pingora | Passed |
| Pingora response header | Passed |
| Socket.io polling route | Passed |
| HTTP load test | Passed |

Load test:

```text
Requests: 1000
Concurrency: 30
Failed requests: 0
Non-2xx responses: 0
Observed rate: 20044.50 requests/sec
```

## Production Readiness Statement

For the tested environment in `docs/tested-matrix.md`, this Pingora profile is production-ready after the production gate passes.

The claim is scoped to this configuration. A different domain, TLS stack, firewall, service manager, ERPNext version, or custom app set must run the same gate before being treated as covered.

## Next Validation Targets

- VPS validation with public DNS and TLS
- 24-hour health and load soak
- restart recovery test for Pingora service supervision
- comparison benchmark against nginx for the same ERPNext bench
