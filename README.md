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

Production runs as a single **Application** in Dokploy, connected to this GitHub
repository on `main`. Enable **Auto Deploy**, select the **Dockerfile** build type,
set `Dockerfile` and context `.`, and use container port `8080`. Dokploy receives
GitHub push events and builds/deploys the application itself. GitHub Actions runs
checks only; no Dokploy API token or deployment secrets are needed in GitHub.
Native push deployment runs independently of the `Check` workflow.

Keep one replica on the host containing the media and database volumes. Set both
update and rollback order to `stop-first`, because replicas must not concurrently
manage scans against the same SQLite database. Attach `dokploy-network` with the
network alias `audio-integrity` for Lidarr and media-maintenance clients. Configure
`https://integrity.nether.pp.ua` in Dokploy Domains with port `8080`.

Set the environment variables above, plus `API_TOKEN` for maintenance access.
Mount the existing `media-audio-integrity-4iyid0_audio-integrity-data` volume at
`/data`; it contains the scan history and cached evidence.

Dokploy 0.30.6 Application mounts do not expose a read-only option. Use local-driver
volumes whose underlying bind mounts are read-only, then add them as volume mounts
in the Application. Run once on the media host:

```sh
docker volume create --driver local --opt type=none --opt o=bind,ro \
  --opt device=/srv/media/music audio-integrity-music-ro
docker volume create --driver local --opt type=none --opt o=bind,ro \
  --opt device=/srv/media/downloads audio-integrity-downloads-ro
docker volume create --driver local --opt type=none --opt o=bind,ro \
  --opt device=/srv/media/import/lidarr-manual audio-integrity-manual-import-ro
```

Mount these at `/music`, `/downloads`, and `/manual-import`, respectively. Keep
placement pinned to `node.hostname==homelab`; these volumes reference local paths.
The named volumes preserve read-only enforcement across native redeployments.

## Automation API

Set an optional, dedicated `API_TOKEN` and pass it in `x-integrity-token` to
access `/api/summary`, `/api/status`, `/api/results`, `/api/history`, and scan
start/cancel endpoints without a browser session. This token has scan-management
access; the existing `LIDARR_TOKEN` remains limited to import verification.
An empty or absent `API_TOKEN` disables token access to the management API.

`POST /api/scans` accepts `{"mode":"incremental"}` or `{"mode":"full"}`.
Poll `/api/status` until `phase` is `completed`; do not consume incomplete scans.
`GET /api/results?verdict=corrupt&limit=500&offset=0` supports pagination; use
`verdict=likely_lossy` for authenticity suspects. Each result includes `mtimeNs`
as a decimal string, `size`, `validatorVersion`, and `authenticityMessage`.
Summary and results describe the library only; Lidarr staging checks remain in
an independent cache and do not inflate library counts. After a successful scan,
results absent from its complete directory inventory are removed atomically.
An empty, readable library clears its old results. A failed directory traversal
or cancelled scan leaves the previous inventory intact. Scan history is retained.
The status response reports `removedFiles` for the completed scan.
Consumers must still match path, size, nanosecond mtime and the current summary's
validator version before acting: files can change after the directory inventory. `likely_lossy` remains a heuristic, and `error` is not a
corruption verdict. The service still mounts audio read-only and never deletes it.
