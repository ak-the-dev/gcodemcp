FROM rust:1.78-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM debian:bookworm-slim

ARG VERSION=dev
ARG VCS_REF=unknown

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

LABEL org.opencontainers.image.title="gcode-mcp" \
      org.opencontainers.image.description="Analyze, validate, generate, and post-process 3D printer G-code over MCP" \
      org.opencontainers.image.source="https://github.com/ak-the-dev/gcodemcp" \
      org.opencontainers.image.url="https://github.com/ak-the-dev/gcodemcp" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      io.modelcontextprotocol.server.name="io.github.ak-the-dev/gcode-mcp"

COPY --from=builder /app/target/release/gcode-mcp /usr/local/bin/gcode-mcp

ENTRYPOINT ["/usr/local/bin/gcode-mcp"]
