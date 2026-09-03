# Audio Integrity

Read-only, self-hosted verification for large audio libraries.

- FLAC integrity: full decode and STREAMINFO MD5 verification through the Xiph reference `flac --test` command.
- Lossless authenticity: separate spectral heuristic powered by the Rust `isflac` analyzer. Its `likely lossy` result is evidence, not proof, and does not override a healthy integrity verdict.
- Other formats: full FFmpeg audio decode; stderr at error level is treated as a failure even when FFmpeg exits zero.
- Persistent SQLite evidence keyed by path, size, nanosecond mtime, and validator version.
- Manual incremental or forced full scans. Three workers overlap validation, while first-pass disk reads stay sequential for HDD-friendly throughput.
- Authenticated WebSocket updates keep every open console session on the same live scan state.
- A fail-closed Lidarr import plugin asks the same service for an integrity verdict before any library mutation.

The music, download, and manual-import mounts are read-only. Audio Integrity has no file mutation endpoints.

## Configuration

Required environment variables:

- `ADMIN_PASSWORD`: web login password.
- `SESSION_TOKEN`: random secret used for the secure session cookie.
- `LIDARR_TOKEN`: shared secret for the internal import-verification endpoint.

Optional variables include `ADMIN_USER`, `DATA_DIR`, `LIBRARY_ROOT`, `IMPORT_ROOTS`, `LISTEN_ADDR`, and `SCAN_WORKERS` (1–4, default 3).

## Development

```sh
mise install
mise run check
```

Run the container with a small read-only fixture directory mounted at `/music`; never point development scans at the production library.

## Production deployment

Production images are built by GitHub Actions and published to `ghcr.io/netherguy4/audio-integrity`. Dokploy only pulls the finished `main` image from `compose.yaml`; it does not compile the application on the production server.

Create a protected GitHub environment named `production` and add these secrets:

- `DOKPLOY_URL`: the base URL of the Dokploy instance, for example `https://dokploy.example.com`.
- `DOKPLOY_API_TOKEN`: an API token created in the Dokploy profile settings.
- `DOKPLOY_COMPOSE_ID`: the ID of the Audio Integrity Compose service.

In Dokploy, keep this repository and `compose.yaml` configured as the Compose source, disable its push-triggered Auto Deploy, and configure the GHCR registry if the package is private. A push to `main` now deploys only after the `Check` workflow succeeds: the deploy workflow builds and publishes `main` plus an immutable `sha-…` tag, then calls Dokploy's `compose.deploy` API.
