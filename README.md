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
