FROM rust:1.80-alpine AS builder

WORKDIR /usr/src/app

RUN apk add --no-cache musl-dev

COPY Cargo.toml ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release || true

# Copy real source code and static assets
COPY src ./src
COPY app ./app
COPY examples ./examples
COPY runtime ./runtime

RUN touch src/main.rs src/lib.rs && cargo build --release

FROM alpine:3.20

WORKDIR /app

RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /usr/src/app/target/release/abapc-server /app/abapc-server
COPY app /app/app
COPY examples /app/examples
COPY runtime /app/runtime

EXPOSE 7345

CMD ["/app/abapc-server"]
