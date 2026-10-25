# syntax=docker/dockerfile:1
FROM rust:1.89-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
RUN cargo build --release --locked
ADD --checksum=sha256:7111af755c74be04400386047afb9e5e7b386d87f6de3f9c906992e34d96eba7 https://github.com/maxdjohnson/lossprint/releases/download/v1.1/lossprint-v1.1-x86_64-unknown-linux-gnu.tar.gz /tmp/lossprint.tar.gz
RUN mkdir /lossprint \
    && tar -xzf /tmp/lossprint.tar.gz -C /lossprint --strip-components=1 \
    && /lossprint/lossprint --help >/dev/null

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates ffmpeg flac \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --create-home integrity \
    && install -d -o integrity -g integrity /data
COPY --from=builder /build/target/release/audio-integrity /usr/local/bin/audio-integrity
COPY --from=builder /lossprint/lossprint /usr/local/bin/lossprint
COPY --from=builder /lossprint/LICENSE /lossprint/THIRD_PARTY.md /usr/share/doc/lossprint/
USER integrity
ENV DATA_DIR=/data LIBRARY_ROOT=/music IMPORT_ROOTS=/downloads,/manual-import LISTEN_ADDR=0.0.0.0:8080 RUST_LOG=audio_integrity=info,tower_http=info
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/audio-integrity"]

