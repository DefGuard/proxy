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

## Verifiability of releases

We provide following ways to verify the authenticity and integrity of official releases:

### Docker Image Verification with Cosign

All official Docker images are signed using [Cosign](https://docs.sigstore.dev/cosign/overview/). To verify a Docker image:

1. [Install](https://github.com/sigstore/cosign?tab=readme-ov-file#installation) cosign CLI

2. Verify the image signature (replace <IMAGE_TAG> with the tag you want to verify):
   ```bash
   cosign verify --certificate-identity-regexp="https://github.com/DefGuard/proxy" \
     --certificate-oidc-issuer="https://token.actions.githubusercontent.com" \
     ghcr.io/defguard/defguard:<IMAGE_TAG>
   ```

### Release Asset Verification

All release assets (binaries, packages, etc.) include SHA256 checksums that are automatically generated and published with each GitHub release:

1. Download the release asset and copy its corresponding checksum from the [releases page](https://github.com/DefGuard/proxy/releases)

2. Verify the checksum:
   ```bash
   # Linux/macOS
   echo known_sha256_checksum_of_the_file path/to/file | sha256sum --check
   ```
