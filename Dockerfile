FROM rust:1.89-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
RUN cargo build --release --locked
RUN cargo install isflac --version 0.1.4 --locked --root /isflac

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates ffmpeg flac \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --create-home integrity \
    && install -d -o integrity -g integrity /data
COPY --from=builder /build/target/release/audio-integrity /usr/local/bin/audio-integrity
COPY --from=builder /isflac/bin/isflac /usr/local/bin/isflac
USER integrity
ENV DATA_DIR=/data LIBRARY_ROOT=/music IMPORT_ROOTS=/downloads,/manual-import LISTEN_ADDR=0.0.0.0:8080 RUST_LOG=audio_integrity=info,tower_http=info
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/audio-integrity"]

