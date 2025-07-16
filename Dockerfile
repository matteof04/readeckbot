FROM rust:alpine AS build
COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "readeckbot"
RUN apk add musl musl-dev
RUN cargo build --release
FROM alpine:latest
COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
USER readeckbot:readeckbot
COPY --from=build --chown=readeckbot:readeckbot ./target/release/readeckbot /app/readeckbot
ENTRYPOINT ["./app/readeckbot"]
