<!-- mcp-name: io.github.ak-the-dev/gcode-mcp -->
# gcode-mcp

[![CI](https://github.com/ak-the-dev/gcodemcp/actions/workflows/ci.yml/badge.svg)](https://github.com/ak-the-dev/gcodemcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

`gcode-mcp` is an MCP server for analyzing, validating, generating, and post-processing 3D printer G-code.

It fits between your slicer and your printer: use it to inspect sliced files, add printer-aware start/end sequences, compare profiles, insert post-processing steps, and power MCP-native print assistants. It is not a CAD tool, mesh modeler, or slicer replacement.

## Positioning

`gcode-mcp` is the G-code automation layer in a modern print workflow:

- CAD or sculpting tools create the model
- slicers generate the base G-code
- `gcode-mcp` analyzes, validates, explains, compares, and modifies that G-code before it reaches a printer or printer host

Use it for:

- preflight checks before printing
- G-code QA for printer, filament, and profile mismatches
- post-processing tasks like pauses, progress tags, speed changes, comment stripping, and extrusion-mode conversion
- calibration utilities and simple procedural print generation
- MCP-powered assistants in Claude, Cursor, Windsurf, and custom clients

Not for:

- arbitrary 3D model generation
- mesh sculpting or character modeling
- replacing a full slicer pipeline

## Distribution Ready

This repository is prepared for MCP directory distribution and broader outreach:

- `server.json` provides official MCP Registry metadata for `io.github.ak-the-dev/gcode-mcp`
- `Dockerfile` and release automation publish a GHCR-backed OCI package for registry verification
- GitHub Actions run format, lint, tests, JSON validation, and container build validation
- [DISTRIBUTION.md](./DISTRIBUTION.md) includes submission copy, release flow, and outreach checklist
- [ROADMAP.md](./ROADMAP.md) covers direct slicer integrations plus OctoPrint and related printer-host backends

## Features

### 🔧 25 Tools

| Category | Tools |
|----------|-------|
| **Analysis** | `parse_gcode`, `analyze_gcode`, `validate_gcode`, `estimate_print_time`, `calculate_filament_usage`, `get_layer_info`, `compare_gcode` |
| **Generation** | `generate_start_gcode`, `generate_end_gcode`, `generate_primitive`, `generate_test_print`, `generate_infill` |
| **Optimization** | `optimize_gcode`, `suggest_speed_profile` |
| **Modification** | `modify_gcode`, `insert_pause`, `change_layer_settings`, `add_progress_reporting`, `convert_extrusion_mode`, `strip_comments` |
| **Utilities** | `lookup_printer`, `lookup_material`, `explain_gcode_command`, `calculate_extrusion`, `convert_units` |

### 📦 4 Resources

- `gcode://printers` — 15 built-in printer profiles (Ender 3, Prusa, Bambu Lab, Voron, etc.)
- `gcode://materials` — 12 material profiles (PLA, PETG, ABS, TPU, Nylon, PC, etc.)
- `gcode://reference/commands` — G-code command reference card
- `gcode://reference/troubleshooting` — Print quality troubleshooting guide

### 💬 5 Prompt Templates

- `create_gcode` — Guided G-code generation workflow
- `optimize_print` — Analyze and optimize existing G-code
- `troubleshoot_print` — Diagnose print quality issues
- `calibrate_printer` — Step-by-step calibration workflow
- `explain_gcode` — Section-by-section G-code explanation

## Real-World Use Cases

- Audit a sliced file before sending it to a printer
- Compare slicer profiles by time, filament, and retraction behavior
- Insert filament changes, pauses, or progress reporting into existing G-code
- Generate safe start and end sequences for known printer/material combinations
- Produce simple procedural outputs like calibration shapes and test prints
- Power an LLM workflow that can answer “what will this file do?” before you hit print

## Installation

### Build from source

```bash
git clone https://github.com/ak-the-dev/gcodemcp.git
cd gcodemcp
cargo build --locked --release
```

The binary will be at `target/release/gcode-mcp`.

### Run from Docker / GHCR

Tagged releases are configured to publish `ghcr.io/ak-the-dev/gcode-mcp`.

```bash
docker run -i --rm ghcr.io/ak-the-dev/gcode-mcp:latest
```

## Usage

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "gcode": {
      "command": "/path/to/gcode_mcp/target/release/gcode-mcp"
    }
  }
}
```

### Docker-based client config

```json
{
  "mcpServers": {
    "gcode": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "ghcr.io/ak-the-dev/gcode-mcp:latest"]
    }
  }
}
```

### Cursor / Windsurf / Other MCP Clients

Add to your MCP configuration:

```json
{
  "mcpServers": {
    "gcode": {
      "command": "/path/to/gcode_mcp/target/release/gcode-mcp"
    }
  }
}
```

## Registry Metadata

- MCP server name: `io.github.ak-the-dev/gcode-mcp`
- Package channel: `ghcr.io/ak-the-dev/gcode-mcp`
- Registry manifest: [`server.json`](./server.json)
- Release and submission guide: [`DISTRIBUTION.md`](./DISTRIBUTION.md)

## Example Queries

Once connected, try asking your LLM:

- *"Generate start G-code for an Ender 3 V2 printing PETG"*
- *"Analyze this G-code and tell me print time and filament usage"*
- *"What does M109 S200 mean?"*
- *"Create a temperature tower test print for PLA on a Prusa MK3S+"*
- *"Optimize this G-code for quality — add z-hop and coasting"*
- *"Suggest speed settings for printing ABS on a Bambu Lab X1C"*
- *"Insert a filament change at layer 15"*
- *"Generate a 20mm calibration cube"*

## Built-in Printer Profiles

| Printer | Type | Build Volume |
|---------|------|--------------|
| Creality Ender 3 / V2 / S1 | Cartesian | 220×220×250 |
| Prusa i3 MK3S+ | Cartesian | 250×210×210 |
| Prusa MINI+ | Cartesian | 180×180×180 |
| Bambu Lab X1C / P1S / A1 | CoreXY/Bedslinger | 256×256×256 |
| Voron 2.4 | CoreXY | 350×350×340 |
| Voron 0.2 | CoreXY | 120×120×120 |
| Creality K1 | CoreXY | 220×220×250 |
| Artillery Sidewinder X2 | Cartesian | 300×300×400 |
| Generic Cartesian/CoreXY/Delta | Various | Various |

## Built-in Materials

PLA, PETG, ABS, ASA, TPU (95A), Nylon (PA), Polycarbonate (PC), PVA, HIPS, Carbon Fiber PLA, Carbon Fiber PETG, Wood PLA

## Architecture

```
src/
├── main.rs              # Entry point
├── mcp/server.rs        # MCP JSON-RPC protocol implementation (stdio)
├── gcode/
│   ├── types.rs         # Core data types
│   ├── parser.rs        # G-code parser & formatter
│   ├── analyzer.rs      # Statistical analysis & validation
│   ├── generator.rs     # G-code generation (primitives, tests, infill)
│   ├── optimizer.rs     # Speed, retraction, z-hop, coasting optimization
│   └── modifier.rs      # Post-processing modifications
├── data/
│   ├── printers.rs      # Printer profile database
│   ├── materials.rs     # Material properties database
│   └── reference.rs     # Command reference & troubleshooting
├── tools.rs             # MCP tool registrations
├── resources.rs         # MCP resource registrations
└── prompts.rs           # MCP prompt templates
```

## Quality Gates

Run the local production checks with:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Release Flow

Tagging `vX.Y.Z` is set up to:

- build and attach release binaries for supported platforms
- publish a multi-arch GHCR image
- publish the server metadata to the official MCP Registry

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for:

- direct slicer integrations via post-processing and native plugins
- OctoPrint, Moonraker/Klipper, Prusa Connect/PrusaLink, and similar printer-host backends
- production-readiness criteria for each integration layer

## License

MIT
