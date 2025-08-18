# Multi-stage Docker build for Kagikanri
FROM node:20-alpine AS frontend-builder

# Set working directory for frontend
WORKDIR /app/frontend

# Copy frontend package files
COPY frontend/package.json frontend/pnpm-lock.yaml ./

# Install pnpm and dependencies
RUN npm install -g pnpm
RUN pnpm install --frozen-lockfile

# Copy frontend source
COPY frontend/ ./

# Build frontend
RUN pnpm build

# Rust builder stage
FROM rust:1.75-alpine AS backend-builder

# Install dependencies for Alpine Linux
RUN apk add --no-cache musl-dev openssl-dev

# Set working directory
WORKDIR /app

# Copy backend Cargo files
COPY backend/Cargo.toml backend/Cargo.lock* backend/build.rs ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src target/release/deps/kagikanri*

# Copy backend source
COPY backend/src ./src

# Copy built frontend
COPY --from=frontend-builder /app/frontend/build ./frontend/build

# Build the backend
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    gnupg \
    git \
    ca-certificates \
    # For pass password manager
    && apk add --no-cache pass \
    # Clean up
    && rm -rf /var/cache/apk/*

# Create app user
RUN addgroup -g 1000 app && \
    adduser -D -s /bin/sh -u 1000 -G app app

# Create necessary directories
RUN mkdir -p /data/password-store /data/passkeys && \
    chown -R app:app /data

# Copy the built binary
COPY --from=backend-builder /app/target/release/kagikanri /usr/local/bin/
COPY --from=backend-builder /app/frontend/build /app/frontend/build

# Switch to app user
USER app

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/api/health || exit 1

# Run the application
CMD ["kagikanri"]