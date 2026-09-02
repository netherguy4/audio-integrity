# Audio Integrity Lidarr plugin

This version-pinned plugin adds an `IImportDecisionEngineSpecification<LocalTrack>` to Lidarr nightly. It calls Audio Integrity before `ImportApprovedTracks` performs any library mutation.

Configuration can be supplied with environment variables:

- `AUDIO_INTEGRITY_URL=http://audio-integrity:8080`
- `AUDIO_INTEGRITY_TOKEN=<same value as LIDARR_TOKEN>`

For an existing compose stack, `/config/audio-integrity.json` is also supported:

```json
{"url":"http://audio-integrity:8080","token":"shared secret"}
```

The plugin fails closed: an unavailable service, missing configuration, invalid response, timeout, or corrupt file rejects the import. A `likely_lossy` authenticity result is logged as a warning but does not reject the structurally healthy file because spectral detection is heuristic.
