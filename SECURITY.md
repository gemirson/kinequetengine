# Security Policy

## Supported Versions
| Version | Supported |
|---------|-----------|
| 6.0.x   | Yes       |

## Reporting a Vulnerability
Email: security@kinequetengine.dev (placeholder)

## Security Measures
- API key authentication required for all endpoints
- Per-tenant rate limiting (50 concurrent requests)
- Input validation on all endpoints
- TLS termination via reverse proxy (nginx/envoy)
- cargo-audit runs in CI

## Running Security Audit
```bash
cargo install cargo-audit
cargo audit
```
