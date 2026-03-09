# GcodeMCP Roadmap

## Current focus

The immediate goal is to keep `gcode-mcp` reliable as a local MCP server before expanding into slicer and printer-host integrations. That means:

- keep the stdio MCP transport compatible with common clients
- keep G-code transforms deterministic and safe for absolute/relative motion modes
- ship CI, release metadata, and docs that make the repo publishable

## Phase 1: Stable core

- Expand parser and analyzer coverage for firmware retract (`G10`/`G11`), arc moves (`G2`/`G3`), and more slicer comment formats.
- Add golden-file tests for start/end G-code generation, optimization passes, and layer-aware modifications.
- Introduce structured logging to `stderr` for production debugging without breaking MCP stdio.
- Add semantic versioning and a release workflow for GitHub builds.

Exit criteria:

- reproducible `cargo test`, `cargo clippy`, and `cargo fmt --check` in CI
- no optimizer pass may silently corrupt relative-position or relative-extrusion files
- generated start/end sequences stay inside the declared printer profile envelope

## Phase 2: Slicer-side integration

### 2.1 Post-processing entry point

Target slicers:

- PrusaSlicer
- OrcaSlicer / Bambu Studio-compatible workflows
- SuperSlicer

Plan:

- add a CLI subcommand that accepts a G-code path and a named transform profile
- support in-place post-processing so slicers can call `gcode-mcp` as an executable script hook
- ship example presets for pause insertion, progress tags, coasting, and printer/material validation

Why this goes first:

- PrusaSlicer already supports post-processing scripts that receive the generated G-code path and can modify the file in place
- that path also maps cleanly onto OrcaSlicer-class workflows, which commonly expose post-processing hooks

Exit criteria:

- one-command install/use examples for PrusaSlicer-style post-processing
- stable JSON output mode for machine-readable slicer wrappers
- dry-run mode that reports intended edits without mutating the file

### 2.2 Native Cura plugin

Plan:

- build a Cura plugin around the stable `Cura.API`
- start with an `OutputDevice` integration for export/upload actions
- add an `Extension` for analysis/repair workflows inside the Cura UI

Scope:

- inspect sliced G-code before upload
- run validation and safe post-processing profiles
- optionally forward jobs to OctoPrint/Moonraker from inside Cura

Exit criteria:

- plugin compatible with the current supported Cura SDK major/minor line
- local-only mode for users who do not want cloud dependencies

### 2.3 Deeper slicer UX integrations

Plan:

- add shared config/import profiles so printer and material mappings stay consistent between slicers and `gcode-mcp`
- expose recommended presets/templates for calibration prints and troubleshooting flows
- investigate Bambu/Orca-specific packaging only after the plugin/post-processing path is stable

## Phase 3: Printer-host integrations

Introduce a `PrintHost` abstraction so MCP tools can upload, queue, start, monitor, and cancel jobs across multiple backends.

### 3.1 OctoPrint

Plan:

- support API-key auth, file upload, job start, job status, printer state, and basic command passthrough
- add MCP tools such as `upload_gcode`, `start_print`, `pause_print`, `cancel_print`, and `get_printer_status`
- optionally expose snapshot/timelapse links as MCP resources

Why first:

- OctoPrint has a mature REST API covering file, job, and printer operations
- Cura and PrusaSlicer already have established OctoPrint upload flows, so this backend fits existing user habits

### 3.2 Moonraker / Klipper

Plan:

- prefer Moonraker's native upload and job APIs
- keep an OctoPrint-compat fallback only where it helps interoperability
- add Klipper-specific resource reads for macros, printer objects, and job metadata

Why second:

- Moonraker explicitly documents both its native external API and an OctoPrint compatibility layer
- many modern slicers already speak to Moonraker directly, so native support matters more than emulation long term

### 3.3 Prusa Connect / PrusaLink

Plan:

- add upload/status support aligned with the same network-print workflows already exposed in PrusaSlicer
- extend later with camera/snapshot support where the public API is available

Why third:

- PrusaSlicer already models these as physical-printer/network-send integrations
- this lets `gcode-mcp` fit into a familiar operator workflow instead of inventing a parallel one

### 3.4 Duet and similar hosts

Candidates:

- Duet / RepRapFirmware
- Obico where direct printer-host APIs are appropriate
- other OctoPrint-compatible hosts where the compatibility surface is strong enough

Plan:

- implement only behind backend capability flags
- require per-host integration tests before marking any backend production ready

## Phase 4: Closed-loop and operator workflows

- Add MCP prompts/resources for print farm triage: queue review, failed job diagnosis, and host health summaries.
- Add policy checks before upload: temperature limits, bed-size bounds, material/printer mismatch, and missing homing/heating.
- Add optional camera/timelapse metadata resources for supported hosts.
- Add slicer-to-host one-click flows once both plugin and host abstractions are stable.

## Recommended architecture changes

- Add `src/integrations/slicers/` and `src/integrations/hosts/`.
- Define traits for `SlicerHook` and `PrintHost`.
- Keep transport/client code separate from G-code mutation logic.
- Treat host credentials as external configuration only; never embed secrets in printer profiles.
- Add integration fixtures with representative outputs from PrusaSlicer, Cura, OrcaSlicer, and Klipper/OctoPrint hosts.

## Definition of production-ready for integrations

Do not mark an integration production-ready until it has:

- end-to-end tests against a real or emulated host
- explicit timeout/retry behavior
- redacted logging for secrets
- deterministic failure messages for the MCP client
- versioned compatibility notes in the docs
