FROM rust AS rust-build
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
COPY src/ /app/src/
RUN cat Cargo.toml
RUN cargo b -r

FROM ghcr.io/dongrote/ffmpeg:v1.0.0
COPY --from=rust-build /app/target/release/compress-mkv /usr/bin/compress-mkv
WORKDIR /work
ENTRYPOINT ["compress-mkv"]
