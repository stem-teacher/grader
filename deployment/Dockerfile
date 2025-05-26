# Frontend build stage
FROM node:18-alpine AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# Backend build stage
FROM rust:1.75 AS backend-builder

WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

COPY backend/src ./src
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    wkhtmltopdf \
    xvfb \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/target/release/hsc-chemistry-backend ./

# Copy frontend assets
COPY --from=frontend-builder /app/frontend/dist ./assets/

# Copy marking guidelines
COPY marking-guidelines/ ./marking-guidelines/

# Create storage directory
RUN mkdir -p ./storage

EXPOSE 8080

CMD ["./hsc-chemistry-backend"]
