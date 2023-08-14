 <p align="center">
    <img src="docs/header.png" alt="defguard">
 </p>

This service is meant to serve as a proxy for a subset of functionalities of [defguard](https://github.com/DefGuard/defguard) core which require public access.
It provides a public REST API and communicates with core over [gRPC](https://github.com/DefGuard/proto).

To learn more about the system see our [documentation](https://defguard.gitbook.io).

## Quick start

If you already have your defguard instance running you can set up a proxy by following our [deployment guide](https://defguard.gitbook.io/defguard/features/setting-up-your-instance/docker-compose).

## Documentation

See the [documentation](https://defguard.gitbook.io) for more information.

## Community and Support

Find us on Matrix: [#defguard:teonite.com](https://matrix.to/#/#defguard:teonite.com)

## Contribution

Please review the [Contributing guide](https://defguard.gitbook.io/defguard/for-developers/contributing) for information on how to get started contributing to the project. You might also find our [environment setup guide](https://defguard.gitbook.io/defguard/for-developers/dev-env-setup) handy.


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

To run webapp dev server:
```bash
cd web/
pnpm install
pnpm run dev
```
