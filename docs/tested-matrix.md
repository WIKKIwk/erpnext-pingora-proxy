# Tested Matrix

Last validated: 2026-06-01, Asia/Tashkent.

| Component | Tested Version / Value |
| --- | --- |
| OS | Fedora release 44 (Forty Four) |
| Kernel | 7.0.10-201.fc44.x86_64 |
| ERPNext | 16.20.0, version-16, `ff46d20` |
| Frappe | 16.19.0, version-16, `ba18090` |
| Bench mode | Local bench, no Docker |
| ERPNext web upstream | `127.0.0.1:8000` |
| ERPNext socket.io upstream | `127.0.0.1:9000` |
| Pingora listen | `127.0.0.1:8088` |
| Pingora crate | 0.8.0 |
| Rust | rustc 1.95.0 |
| Cargo | cargo 1.95.0 |
| Node.js | v22.22.2 |
| Python | 3.14.5 |
| Site host | `erpnext.localhost` |

## Gate Result

Production gate: passed.

Command:

```bash
PINGORA_GATE_REQUESTS=1000 PINGORA_GATE_CONCURRENCY=30 ./scripts/production-gate.sh
```

Observed result:

```text
OK   cargo test
OK   release build
OK   health endpoint
OK   ERPNext web route
OK   server header
OK   socket.io polling route
OK   load test 1000 requests concurrency=30 rps=20044.50
Pingora ERPNext production gate passed.
```
