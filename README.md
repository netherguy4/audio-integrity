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

The following is a generic Dokploy deployment example, not a description of a
specific installation. Keep actual domains, hostnames, volume names, host paths
and credentials in your deployment settings or a private operations runbook.

Create a single **Application** in Dokploy, connect your repository and deployment
branch, select the **Dockerfile** build type, set `Dockerfile` and context `.`,
and use container port `8080`. Enable **Auto Deploy** when pushes to that branch
should trigger a deployment. GitHub Actions runs checks only; no Dokploy API token
or deployment secrets are needed in GitHub for this setup. Native push deployment
runs independently of the `Check` workflow.

Keep one replica on the host containing the media and database volumes. Set both
update and rollback order to `stop-first`, because replicas must not concurrently
manage scans against the same SQLite database. Attach the shared Docker network
used by your deployment, with a stable service alias such as `audio-integrity`
for Lidarr and other internal clients. Configure your HTTPS domain in Dokploy
Domains with port `8080`; `https://integrity.example.com` is an example only.

Set the environment variables above, plus an optional, dedicated `API_TOKEN` for
maintenance access. Mount your persistent data volume at `/data`; it contains
the scan history and cached evidence. **For an existing installation, reuse its
original data volume. Do not replace, rename or delete it to match an example.**

Media, downloads and manual-import directories must remain read-only. If your
Dokploy Application mount UI does not expose a read-only option, use local-driver
volumes whose underlying bind mounts are read-only, then add them as volume mounts
in the Application. Set `MUSIC_HOST_PATH`, `DOWNLOADS_HOST_PATH` and
`IMPORT_HOST_PATH` to existing absolute directories on the selected media host
before running the example below. The `example-integrity-*` volume names are
illustrative; choose unused names for a new installation and retain the configured
names for an existing one.

```sh
: "${MUSIC_HOST_PATH:?Set the absolute path to your existing music directory}"
: "${DOWNLOADS_HOST_PATH:?Set the absolute path to your existing downloads directory}"
: "${IMPORT_HOST_PATH:?Set the absolute path to your existing manual-import directory}"

for source in "$MUSIC_HOST_PATH" "$DOWNLOADS_HOST_PATH" "$IMPORT_HOST_PATH"; do
  case "$source" in
    /*) ;;
    *) printf '%s\n' 'Each source must be an absolute path.' >&2; exit 1 ;;
  esac
  [ -d "$source" ] || { printf '%s\n' 'Each source directory must already exist.' >&2; exit 1; }
done

docker volume create --driver local --opt type=none --opt o=bind,ro \
  --opt device="$MUSIC_HOST_PATH" example-integrity-music-ro
docker volume create --driver local --opt type=none --opt o=bind,ro \
  --opt device="$DOWNLOADS_HOST_PATH" example-integrity-downloads-ro
docker volume create --driver local --opt type=none --opt o=bind,ro \
  --opt device="$IMPORT_HOST_PATH" example-integrity-manual-import-ro
```

Mount these at `/music`, `/downloads`, and `/manual-import`, respectively. Pin
placement to the host holding the source directories, using a constraint such as
`node.hostname==<media-hostname>` with your actual hostname substituted. These
volumes reference local paths and must not be scheduled on an unrelated node.
Verify read-only enforcement before scanning and after changing mount settings.
This example does not require moving any existing data or changing application
paths inside the container.

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
