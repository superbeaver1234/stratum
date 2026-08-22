# Security

## Secrets

Do not commit RPC passwords, API keys, private keys, seed phrases, NiceHash credentials, exchange credentials or bearer tokens.

Use environment variables or Docker secrets. A vault-compatible provider may be added later.

## Network boundaries

- Blockchain daemon RPC interfaces belong on localhost/private networks only.
- PostgreSQL, Prometheus and Grafana are private by default.
- Public exposure is limited to the Stratum listener and explicitly authenticated control API.
- RPC credentials must never be included in structured logs, metrics labels, panic text or error telemetry.

## Untrusted input

Stratum clients, upstream pools, daemon RPC responses and exchange APIs are untrusted boundaries. Apply message size limits, strict decoding, timeouts, bounded queues and typed validation.

## Safety behavior

Any uncertainty that can invalidate native-chain work or profitability inputs must disable the affected route and prefer BTC SAFE MODE.
