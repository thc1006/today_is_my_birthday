# Stage 1: Build Rust binary
FROM docker.io/library/rust:1.85-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create non-root user for build
WORKDIR /build

# Copy Cargo files first for dependency caching (including Cargo.lock for reproducible builds)
COPY Cargo.toml Cargo.lock ./

# Create dummy src for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (locked versions)
RUN cargo build --release --locked && rm -rf src target/release/deps/thc1006*

# Copy the source, templates and page data (templates and data are compiled into the binary)
COPY src ./src
COPY templates ./templates
COPY content ./content

# Build the actual binary (with --locked for reproducible builds)
RUN cargo build --release --locked

# Stage 2: Runtime image (minimal Alpine)
FROM docker.io/library/alpine:3.21

# Install ca-certificates for HTTPS if needed
RUN apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

# Create non-root user
RUN addgroup -g 1000 app && \
    adduser -u 1000 -G app -s /bin/sh -D app

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/thc1006-web /app/thc1006-web

# Copy static files
COPY static /app/static

# Change ownership to non-root user
RUN chown -R app:app /app

# Switch to non-root user
USER app

# Expose port
EXPOSE 8080

# Health check (using BusyBox-compatible wget flags)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -q -T 3 -O /dev/null http://localhost:8080/ || exit 1

# Record the commit this image was built from (scripts/deploy.sh passes it in).
# A build argument changes the cache key, so the label is written even when every
# earlier step is cached; podman 3.4 skips `--label` in that case.
ARG REVISION=unknown
LABEL org.opencontainers.image.revision=$REVISION

# The deployed commit's date, shown as 最後更新 in the homepage footer.
ARG SITE_UPDATED=
ENV SITE_UPDATED=$SITE_UPDATED

# Run the server
CMD ["/app/thc1006-web"]
