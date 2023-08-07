# DefGuard Proxy

This service is meant to serve as a proxy for a subset of functionalities of DefGuard core which require public access.
It provides a public REST API and communicates with core over gRPC.

## Development

Clone repository:
```bash
git@github.com:DefGuard/client.git
```

Initialize `proto` submodule:
```bash
git submodule update --init --recursive
```

To run API server:
```bash
cargo run
```
