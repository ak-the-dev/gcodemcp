# gcode-mcp

A high-performance MCP (Model Context Protocol) server for 3D printer G-code creation, analysis, and optimization. Built in Rust for speed and reliability.

## Status

This repository is now set up like a publishable Rust binary project:

- CI runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`
- build artifacts are ignored instead of committed
- the crate metadata points at [ak-the-dev/gcodemcp](https://github.com/ak-the-dev/gcodemcp)
- the integration plan lives in [ROADMAP.md](./ROADMAP.md)

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

## Installation

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.70+)

### Build

```bash
git clone https://github.com/ak-the-dev/gcodemcp.git
cd gcodemcp
cargo build --release
```

The binary will be at `target/release/gcode-mcp`.

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

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for:

- direct slicer integrations via post-processing and native plugins
- OctoPrint, Moonraker/Klipper, Prusa Connect/PrusaLink, and similar printer-host backends
- production-readiness criteria for each integration layer

## License

MIT
