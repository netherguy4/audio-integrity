# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Rust/Axum backend with a server-delivered HTML, CSS, and JavaScript interface; SQLite persistence; containerized deployment in Dokploy.

## Users

The primary user is the homeserver owner reviewing and protecting a large personal music library from a desktop or phone browser.

## Product Purpose

Audio Integrity performs strict, read-only validation of audio files, remembers completed checks, and prevents corrupt downloads from being imported by Lidarr. Success means a manual incremental scan finishes without rereading unchanged files and every actionable defect is visible with its exact path and validator output.

## Positioning

Unlike general media probes, FLAC files are fully decoded with the reference Xiph `flac --test` path; a small Lidarr pre-import adapter asks the same Rust service for a verdict before Lidarr mutates the library.

## Operating Context

The app runs in the homeserver's Dokploy Media stack. It reads `/srv/media/music`, Lidarr download staging, and manual-import staging through read-only mounts. The existing library is approximately 29,000 tracks and 786 GiB on an HDD.

## Capabilities and Constraints

- Manual incremental and forced full-library scans; no scheduled library scans.
- Persistent per-file results keyed by path, size, high-resolution modification time, and validator version.
- Strict FLAC validation plus decoder-based validation for supported non-FLAC audio.
- Three bounded library workers by default, configurable from one to four. Integrity reads are serialized to keep a single HDD sequential while spectral analysis of cached files overlaps on the other workers.
- Pause/cancel is safe and retains completed results.
- Lidarr import validation is automatic and fail-closed.
- No feature may modify, move, rename, repair, quarantine, or delete music.
- Responsive web UI with Russian and English localization.

## Brand Commitments

The interface uses AudioMuse-AI as its explicit interaction and visual reference: familiar sidebar navigation, restrained blue accent, light/dark themes, task-status cards, live progress, and operational tables. It must remain clearly named Audio Integrity and must not impersonate AudioMuse.

## Evidence on Hand

- Known corrupt fixture in the library: `Cjbeards/Killed The Cat (2022)/01 - Killed The Cat.flac`.
- FFmpeg reports `invalid residual` for the fixture but exits successfully, proving that FFmpeg exit status alone is insufficient.
- Current Lidarr is LinuxServer nightly 3.1.5.5041 with plugin support.

## Product Principles

- Read-only by construction.
- Remember trustworthy work; make deliberate full audits explicit.
- Put the exact failing file and error ahead of aggregate decoration.
- Block uncertain imports rather than admitting unverified media.
- Keep long-running work observable and safely cancellable.

## Accessibility & Inclusion

Keyboard-operable controls, visible focus, semantic status text in addition to color, reduced-motion support, responsive layouts, and complete RU/EN interface strings are required.
