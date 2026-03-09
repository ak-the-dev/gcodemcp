use crate::data::{materials, printers, reference};
use crate::mcp::server::{McpServer, ResourceDef};
use serde_json::json;

pub fn register_all(server: &mut McpServer) {
    // Printer list
    server.add_resource(
        ResourceDef {
            uri: "gcode://printers".into(),
            name: "Printer Profiles".into(),
            description: "List of all built-in 3D printer profiles with specifications".into(),
            mime_type: "application/json".into(),
        },
        Box::new(|| {
            let printers: Vec<_> = printers::all_printers()
                .iter()
                .map(|p| {
                    json!({
                        "id": p.id, "name": p.name, "manufacturer": p.manufacturer,
                        "bed_size": format!("{}x{}x{}", p.bed_x, p.bed_y, p.max_z),
                        "extruder": p.extruder_type, "max_speed": p.max_print_speed,
                        "description": p.description
                    })
                })
                .collect();
            serde_json::to_string_pretty(&printers).map_err(|e| e.to_string())
        }),
    );

    // Material list
    server.add_resource(
        ResourceDef {
            uri: "gcode://materials".into(),
            name: "Material Profiles".into(),
            description: "List of all built-in filament material profiles with settings".into(),
            mime_type: "application/json".into(),
        },
        Box::new(|| {
            let mats: Vec<_> = materials::all_materials().iter().map(|m| {
                json!({
                    "id": m.id, "name": m.name,
                    "nozzle_temp": format!("{}-{}°C (default {})", m.nozzle_temp_min, m.nozzle_temp_max, m.nozzle_temp_default),
                    "bed_temp": format!("{}-{}°C (default {})", m.bed_temp_min, m.bed_temp_max, m.bed_temp_default),
                    "speed": format!("{}-{} mm/s", m.print_speed_min, m.print_speed_max),
                    "description": m.description
                })
            }).collect();
            serde_json::to_string_pretty(&mats).map_err(|e| e.to_string())
        }),
    );

    // Command reference
    server.add_resource(
        ResourceDef {
            uri: "gcode://reference/commands".into(),
            name: "G-code Command Reference".into(),
            description: "Comprehensive G-code command reference for FDM 3D printing".into(),
            mime_type: "text/markdown".into(),
        },
        Box::new(|| Ok(reference::command_reference())),
    );

    // Troubleshooting guide
    server.add_resource(
        ResourceDef {
            uri: "gcode://reference/troubleshooting".into(),
            name: "Print Troubleshooting Guide".into(),
            description: "Common 3D print quality issues and their solutions".into(),
            mime_type: "text/markdown".into(),
        },
        Box::new(|| Ok(reference::troubleshooting_guide())),
    );
}
