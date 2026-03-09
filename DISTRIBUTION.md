# Distribution and Outreach

This repository is prepared for the official MCP Registry plus downstream aggregators and directory listings that consume registry metadata, GitHub release assets, and container packages.

## Included Distribution Assets

- [`server.json`](./server.json): canonical MCP metadata for `io.github.ak-the-dev/gcode-mcp`
- [`Dockerfile`](./Dockerfile): GHCR-ready OCI image with the required MCP server annotation
- [`.github/workflows/release.yml`](./.github/workflows/release.yml): tagged-release pipeline for binaries, GHCR, and registry publication
- [`docs/assets/gcode-mcp-icon.svg`](./docs/assets/gcode-mcp-icon.svg): icon for registries and listings
- [`README.md`](./README.md): install snippets, positioning, and MCP identity marker

## Listing Copy

One-line pitch:

`gcode-mcp` is an MCP server for analyzing, validating, generating, and post-processing 3D printer G-code.

Short description:

Use `gcode-mcp` between your slicer and your printer to inspect G-code, compare profiles, add printer-aware start and end sequences, insert pauses or progress reporting, and power print-focused MCP assistants.

Suggested tags:

- MCP
- Model Context Protocol
- 3D printing
- G-code
- slicer post-processing
- OctoPrint
- printer automation

## Official MCP Registry Flow

1. Keep `Cargo.toml`, `server.json`, and README messaging aligned for the next release.
2. Create a version tag such as `v1.0.1`.
3. Push the tag to GitHub.
4. The release workflow will:
   - run format, lint, test, and metadata checks
   - build release binaries
   - publish `ghcr.io/ak-the-dev/gcode-mcp`
   - stamp `server.json` with the tag version for publication
   - publish the server to the official MCP Registry via `mcp-publisher`

## Downstream Aggregator Readiness

The official MCP Registry exposes a read-only REST API that downstream aggregators can ingest. This repo is prepared for that model by shipping:

- a stable reverse-DNS server name
- machine-readable registry metadata
- a public OCI package source
- a public GitHub repository with install instructions and roadmap

## Manual Outreach Checklist

- Set GitHub topics: `mcp`, `model-context-protocol`, `3d-printing`, `gcode`, `claude-desktop`, `cursor`, `octoprint`
- Keep the GitHub repo description aligned with the one-line pitch above
- Add a GitHub social preview image if you want better link unfurls on X, LinkedIn, and Discord
- Submit the project to MCP server directories and awesome lists that accept local stdio or Docker-backed servers
- Post release notes with one real printer or slicer workflow example
- Link to the integration roadmap when sharing in 3D-printing communities

## Outreach Angles

- Safer print workflows: preflight validation before a file ever hits a printer
- Slicer-adjacent automation: post-processing without writing one-off scripts per slicer
- Print assistant workflows: natural-language auditing and G-code explanation in MCP clients
- Future integrations: direct slicer plugins plus OctoPrint, Moonraker, and similar printer-host backends
