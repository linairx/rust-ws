# Build stage
FROM rust:1.93-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/rust-ws /app/rust-ws

# Copy static files
COPY static ./static

# Create non-root user
RUN addgroup -g 1000 app && \
    adduser -u 1000 -G app -s /bin/sh -D app && \
    chown -R app:app /app

USER app

# Environment variables with defaults
ENV PORT=3000
ENV UUID=7bd180e8-1142-4387-93f5-03e8d750a896
ENV WS_PATH=7bd180e8
ENV SUB_PATH=sub
ENV DOMAIN=""
ENV NAME=""
ENV AUTO_ACCESS=false
ENV DEBUG=false

EXPOSE ${PORT}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT}/health || exit 1

CMD ["./rust-ws"]
