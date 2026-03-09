use crate::data::{materials, printers, reference};
use crate::gcode::{analyzer, generator, modifier, optimizer, parser, types::*};
use crate::mcp::server::{McpServer, ToolDef};
use serde_json::{json, Value};
use std::f64::consts::PI;

pub fn register_all(server: &mut McpServer) {
    // === Analysis Tools ===
    register_parse_gcode(server);
    register_analyze_gcode(server);
    register_validate_gcode(server);
    register_estimate_print_time(server);
    register_calculate_filament(server);
    register_get_layer_info(server);
    register_compare_gcode(server);

    // === Generation Tools ===
    register_generate_start_gcode(server);
    register_generate_end_gcode(server);
    register_generate_primitive(server);
    register_generate_test_print(server);
    register_generate_infill(server);

    // === Optimization Tools ===
    register_optimize_gcode(server);
    register_suggest_speed_profile(server);

    // === Modification Tools ===
    register_modify_gcode(server);
    register_insert_pause(server);
    register_change_layer_settings(server);
    register_add_progress(server);
    register_convert_extrusion_mode(server);
    register_strip_comments(server);

    // === Utility Tools ===
    register_lookup_printer(server);
    register_lookup_material(server);
    register_explain_command(server);
    register_calculate_extrusion(server);
    register_convert_units(server);
}

fn s(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
fn f(v: &Value, key: &str, def: f64) -> f64 {
    v.get(key).and_then(|v| v.as_f64()).unwrap_or(def)
}
fn u(v: &Value, key: &str, def: u32) -> u32 {
    v.get(key).and_then(|v| v.as_u64()).unwrap_or(def as u64) as u32
}
fn b(v: &Value, key: &str, def: bool) -> bool {
    v.get(key).and_then(|v| v.as_bool()).unwrap_or(def)
}

fn register_parse_gcode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "parse_gcode".into(),
        description: "Parse raw G-code text into a structured representation with commands, parameters, and comments".into(),
        input_schema: json!({"type":"object","properties":{"gcode":{"type":"string","description":"Raw G-code text to parse"}},"required":["gcode"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let file = parser::parse(&gcode);
        Ok(serde_json::to_value(&file).unwrap_or(json!({})))
    }));
}

fn register_analyze_gcode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "analyze_gcode".into(),
        description: "Perform full statistical analysis of G-code: print time, filament usage, layers, speeds, temperatures, bounding box, and detected issues".into(),
        input_schema: json!({"type":"object","properties":{"gcode":{"type":"string","description":"G-code to analyze"}},"required":["gcode"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let result = analyzer::analyze(&gcode);
        Ok(serde_json::to_value(&result).unwrap_or(json!({})))
    }));
}

fn register_validate_gcode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "validate_gcode".into(),
        description: "Validate G-code and return errors, warnings, and suggestions for improvement".into(),
        input_schema: json!({"type":"object","properties":{"gcode":{"type":"string","description":"G-code to validate"}},"required":["gcode"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let issues = analyzer::validate(&gcode);
        Ok(serde_json::to_value(&issues).unwrap_or(json!([])))
    }));
}

fn register_estimate_print_time(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "estimate_print_time".into(),
        description: "Estimate total print time in seconds, accounting for move distances and feed rates".into(),
        input_schema: json!({"type":"object","properties":{"gcode":{"type":"string","description":"G-code to estimate"}},"required":["gcode"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let secs = analyzer::estimate_print_time(&gcode);
        let hrs = (secs / 3600.0).floor();
        let mins = ((secs % 3600.0) / 60.0).floor();
        Ok(json!({"seconds": secs, "formatted": format!("{}h {}m {:.0}s", hrs, mins, secs % 60.0)}))
    }));
}

fn register_calculate_filament(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "calculate_filament_usage".into(),
        description: "Calculate filament length, weight, and cost from G-code".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode":{"type":"string","description":"G-code to analyze"},
            "material_id":{"type":"string","description":"Material ID (e.g. 'pla', 'petg'). Uses density from material db. Optional."},
            "density":{"type":"number","description":"Material density in g/cm³ (overrides material_id). Default 1.24 (PLA)"},
            "filament_diameter":{"type":"number","description":"Filament diameter in mm. Default 1.75"},
            "cost_per_kg":{"type":"number","description":"Cost per kg in your currency. Default 25"}
        },"required":["gcode"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let material_id = s(&args, "material_id");
        let material = if !material_id.is_empty() { materials::get_material(&material_id) } else { None };
        let density = f(&args, "density", material.as_ref().map(|m| m.density).unwrap_or(1.24));
        let diameter = f(&args, "filament_diameter", 1.75);
        let cost = f(&args, "cost_per_kg", 25.0);
        Ok(analyzer::calculate_filament(&gcode, density, diameter, cost))
    }));
}

fn register_get_layer_info(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "get_layer_info".into(),
        description: "Get detailed information about a specific layer number (z-height, extrusion, travel, retractions)".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode":{"type":"string","description":"G-code to analyze"},
            "layer":{"type":"integer","description":"Layer number (0-indexed)"}
        },"required":["gcode","layer"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let layer = u(&args, "layer", 0) as usize;
        match analyzer::get_layer_info(&gcode, layer) {
            Some(info) => Ok(serde_json::to_value(&info).unwrap_or(json!({}))),
            None => Err(format!("Layer {} not found", layer)),
        }
    }));
}

fn register_compare_gcode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "compare_gcode".into(),
        description: "Compare two G-code files and show statistical differences (time, filament, layers, speeds)".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode_a":{"type":"string","description":"First G-code"},
            "gcode_b":{"type":"string","description":"Second G-code"}
        },"required":["gcode_a","gcode_b"]}),
    }, Box::new(|args: Value| {
        let a = analyzer::analyze(&s(&args, "gcode_a"));
        let b = analyzer::analyze(&s(&args, "gcode_b"));
        Ok(json!({
            "time_diff_seconds": b.estimated_time_seconds - a.estimated_time_seconds,
            "filament_diff_mm": b.filament_used_mm - a.filament_used_mm,
            "layer_diff": b.total_layers as i64 - a.total_layers as i64,
            "retraction_diff": b.total_retractions as i64 - a.total_retractions as i64,
            "file_a": {"time_s": a.estimated_time_seconds, "filament_mm": a.filament_used_mm, "layers": a.total_layers, "retractions": a.total_retractions, "issues": a.issues.len()},
            "file_b": {"time_s": b.estimated_time_seconds, "filament_mm": b.filament_used_mm, "layers": b.total_layers, "retractions": b.total_retractions, "issues": b.issues.len()},
        }))
    }));
}

fn register_generate_start_gcode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "generate_start_gcode".into(),
        description: "Generate printer and material-aware start G-code sequence (homing, heating, priming)".into(),
        input_schema: json!({"type":"object","properties":{
            "printer_id":{"type":"string","description":"Printer ID (e.g. 'ender3', 'prusa_mk3s', 'bambu_x1c')"},
            "material_id":{"type":"string","description":"Material ID (e.g. 'pla', 'petg', 'abs')"}
        },"required":["printer_id","material_id"]}),
    }, Box::new(|args: Value| {
        let printer = printers::get_printer(&s(&args, "printer_id")).ok_or("Unknown printer ID")?;
        let material = materials::get_material(&s(&args, "material_id")).ok_or("Unknown material ID")?;
        Ok(Value::String(generator::start_gcode(&printer, &material)))
    }));
}

fn register_generate_end_gcode(server: &mut McpServer) {
    server.add_tool(
        ToolDef {
            name: "generate_end_gcode".into(),
            description:
                "Generate safe end G-code sequence (retract, raise, cool down, disable motors)"
                    .into(),
            input_schema: json!({"type":"object","properties":{
            "printer_id":{"type":"string","description":"Printer ID"}
        },"required":["printer_id"]}),
        },
        Box::new(|args: Value| {
            let printer =
                printers::get_printer(&s(&args, "printer_id")).ok_or("Unknown printer ID")?;
            Ok(Value::String(generator::end_gcode(&printer)))
        }),
    );
}

fn register_generate_primitive(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "generate_primitive".into(),
        description: "Generate G-code for geometric primitives: line, rectangle, circle, cylinder, cube, spiral_vase".into(),
        input_schema: json!({"type":"object","properties":{
            "type":{"type":"string","enum":["line","rectangle","circle","cylinder","cube","spiral_vase"],"description":"Primitive type"},
            "x":{"type":"number","description":"X position"},
            "y":{"type":"number","description":"Y position"},
            "x2":{"type":"number","description":"End X (for line)"},
            "y2":{"type":"number","description":"End Y (for line)"},
            "width":{"type":"number","description":"Width (rectangle)"},
            "height":{"type":"number","description":"Height (3D objects or rectangle)"},
            "radius":{"type":"number","description":"Radius (circle/cylinder/spiral)"},
            "size":{"type":"number","description":"Side length (cube)"},
            "segments":{"type":"integer","description":"Number of segments for curves. Default 32"},
            "layer_height":{"type":"number","description":"Layer height in mm. Default 0.2"},
            "nozzle_diameter":{"type":"number","description":"Nozzle diameter in mm. Default 0.4"},
            "speed":{"type":"number","description":"Print speed in mm/s. Default 50"}
        },"required":["type"]}),
    }, Box::new(|args: Value| {
        let ptype = s(&args, "type");
        let lh = f(&args, "layer_height", 0.2);
        let nz = f(&args, "nozzle_diameter", 0.4);
        let sp = f(&args, "speed", 50.0);
        let segs = u(&args, "segments", 32);
        let prim = match ptype.as_str() {
            "line" => Primitive::Line { x1: f(&args,"x",0.0), y1: f(&args,"y",0.0), x2: f(&args,"x2",50.0), y2: f(&args,"y2",50.0) },
            "rectangle" => Primitive::Rectangle { x: f(&args,"x",50.0), y: f(&args,"y",50.0), width: f(&args,"width",40.0), height: f(&args,"height",30.0) },
            "circle" => Primitive::Circle { cx: f(&args,"x",100.0), cy: f(&args,"y",100.0), radius: f(&args,"radius",20.0), segments: segs },
            "cylinder" => Primitive::Cylinder { cx: f(&args,"x",100.0), cy: f(&args,"y",100.0), radius: f(&args,"radius",20.0), height: f(&args,"height",10.0), segments: segs },
            "cube" => Primitive::Cube { x: f(&args,"x",80.0), y: f(&args,"y",80.0), size: f(&args,"size",20.0), height: f(&args,"height",20.0) },
            "spiral_vase" => Primitive::SpiralVase { cx: f(&args,"x",100.0), cy: f(&args,"y",100.0), radius: f(&args,"radius",30.0), height: f(&args,"height",20.0), segments_per_layer: segs },
            _ => return Err(format!("Unknown primitive type: {}", ptype)),
        };
        Ok(Value::String(generator::primitive(&prim, lh, nz, sp)))
    }));
}

fn register_generate_test_print(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "generate_test_print".into(),
        description: "Generate calibration test print G-code: temp_tower, retraction_test, bed_level, flow_test, bridging_test, first_layer_calibration".into(),
        input_schema: json!({"type":"object","properties":{
            "test_type":{"type":"string","enum":["temp_tower","retraction_test","bed_level","flow_test","bridging_test","first_layer_calibration"]},
            "printer_id":{"type":"string","description":"Printer ID"},
            "material_id":{"type":"string","description":"Material ID"},
            "start_temp":{"type":"number","description":"Start temp for temp tower"},
            "end_temp":{"type":"number","description":"End temp for temp tower"},
            "temp_step":{"type":"number","description":"Temp step between blocks. Default 5"},
            "start_distance":{"type":"number","description":"Start retraction distance"},
            "end_distance":{"type":"number","description":"End retraction distance"},
            "distance_step":{"type":"number","description":"Retraction distance step. Default 0.5"}
        },"required":["test_type","printer_id","material_id"]}),
    }, Box::new(|args: Value| {
        let printer = printers::get_printer(&s(&args, "printer_id")).ok_or("Unknown printer ID")?;
        let material = materials::get_material(&s(&args, "material_id")).ok_or("Unknown material ID")?;
        let test = match s(&args, "test_type").as_str() {
            "temp_tower" => TestPrintType::TempTower { start_temp: f(&args,"start_temp",material.nozzle_temp_min), end_temp: f(&args,"end_temp",material.nozzle_temp_max), step: f(&args,"temp_step",5.0) },
            "retraction_test" => TestPrintType::RetractionTest { start_distance: f(&args,"start_distance",0.5), end_distance: f(&args,"end_distance",6.0), step: f(&args,"distance_step",0.5) },
            "bed_level" => TestPrintType::BedLevel,
            "flow_test" => TestPrintType::FlowTest,
            "bridging_test" => TestPrintType::BridgingTest,
            "first_layer_calibration" => TestPrintType::FirstLayerCalibration,
            t => return Err(format!("Unknown test type: {}", t)),
        };
        Ok(Value::String(generator::test_print(&test, &printer, &material)))
    }));
}

fn register_generate_infill(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "generate_infill".into(),
        description: "Generate infill pattern G-code for a rectangular region: lines, grid, triangles, honeycomb, concentric".into(),
        input_schema: json!({"type":"object","properties":{
            "pattern":{"type":"string","enum":["lines","grid","triangles","honeycomb","concentric"]},
            "x":{"type":"number"}, "y":{"type":"number"}, "width":{"type":"number"}, "height":{"type":"number"},
            "spacing":{"type":"number","description":"Line spacing mm. Default 2.0"},
            "layer_height":{"type":"number"}, "nozzle_diameter":{"type":"number"}, "speed":{"type":"number"}
        },"required":["pattern","x","y","width","height"]}),
    }, Box::new(|args: Value| {
        let pattern = match s(&args, "pattern").as_str() {
            "lines" => InfillPattern::Lines, "grid" => InfillPattern::Grid, "triangles" => InfillPattern::Triangles,
            "honeycomb" => InfillPattern::Honeycomb, "concentric" => InfillPattern::Concentric,
            p => return Err(format!("Unknown pattern: {}", p)),
        };
        let options = InfillOptions {
            x: f(&args, "x", 0.0),
            y: f(&args, "y", 0.0),
            width: f(&args, "width", 50.0),
            height: f(&args, "height", 50.0),
            spacing: f(&args, "spacing", 2.0),
            layer_height: f(&args, "layer_height", 0.2),
            nozzle_diameter: f(&args, "nozzle_diameter", 0.4),
            speed: f(&args, "speed", 50.0),
        };
        Ok(Value::String(generator::infill(&pattern, &options)))
    }));
}

fn register_optimize_gcode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "optimize_gcode".into(),
        description: "Optimize G-code with configurable options: retraction cleanup, z-hop, coasting, speed optimization".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode":{"type":"string","description":"G-code to optimize"},
            "optimize_travel":{"type":"boolean","description":"Optimize travel paths. Default true"},
            "optimize_retraction":{"type":"boolean","description":"Remove unnecessary retractions. Default true"},
            "optimize_speed":{"type":"boolean","description":"Cap first layer speed. Default true"},
            "add_z_hop":{"type":"boolean","description":"Add z-hop to travel moves. Default false"},
            "z_hop_height":{"type":"number","description":"Z-hop height mm. Default 0.4"},
            "add_coasting":{"type":"boolean","description":"Add coasting before travel. Default false"},
            "coasting_distance":{"type":"number","description":"Coasting distance mm. Default 0.3"},
            "min_travel_for_retract":{"type":"number","description":"Min travel distance to retract mm. Default 1.5"}
        },"required":["gcode"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let opts = OptimizationOptions {
            optimize_travel: b(&args, "optimize_travel", true),
            optimize_retraction: b(&args, "optimize_retraction", true),
            optimize_speed: b(&args, "optimize_speed", true),
            add_z_hop: b(&args, "add_z_hop", false),
            z_hop_height: f(&args, "z_hop_height", 0.4),
            add_coasting: b(&args, "add_coasting", false),
            coasting_distance: f(&args, "coasting_distance", 0.3),
            min_travel_for_retract: f(&args, "min_travel_for_retract", 1.5),
        };
        Ok(Value::String(optimizer::optimize(&gcode, &opts)))
    }));
}

fn register_suggest_speed_profile(server: &mut McpServer) {
    server.add_tool(
        ToolDef {
            name: "suggest_speed_profile".into(),
            description: "Suggest optimal speed settings for a printer and material combination"
                .into(),
            input_schema: json!({"type":"object","properties":{
            "printer_id":{"type":"string"}, "material_id":{"type":"string"}
        },"required":["printer_id","material_id"]}),
        },
        Box::new(|args: Value| {
            let printer =
                printers::get_printer(&s(&args, "printer_id")).ok_or("Unknown printer")?;
            let material =
                materials::get_material(&s(&args, "material_id")).ok_or("Unknown material")?;
            Ok(optimizer::suggest_speed_profile(&printer, &material))
        }),
    );
}

fn register_modify_gcode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "modify_gcode".into(),
        description: "Apply modifications: search_replace, insert_at_layer, scale, translate, mirror. Specify operation type and parameters.".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode":{"type":"string","description":"G-code to modify"},
            "operation":{"type":"string","enum":["search_replace","insert_at_layer","scale","translate","mirror"]},
            "search":{"type":"string"}, "replace":{"type":"string"},
            "layer":{"type":"integer"}, "insert_gcode":{"type":"string"},
            "scale_x":{"type":"number"}, "scale_y":{"type":"number"}, "scale_z":{"type":"number"},
            "offset_x":{"type":"number"}, "offset_y":{"type":"number"}, "offset_z":{"type":"number"},
            "axis":{"type":"string","description":"Mirror axis: X, Y, or Z"}
        },"required":["gcode","operation"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let op = match s(&args, "operation").as_str() {
            "search_replace" => ModificationOp::SearchReplace { search: s(&args, "search"), replace: s(&args, "replace") },
            "insert_at_layer" => ModificationOp::InsertAtLayer { layer: u(&args, "layer", 0) as usize, gcode: s(&args, "insert_gcode") },
            "scale" => ModificationOp::ScaleCoordinates { scale_x: f(&args,"scale_x",1.0), scale_y: f(&args,"scale_y",1.0), scale_z: f(&args,"scale_z",1.0) },
            "translate" => ModificationOp::TranslateCoordinates { offset_x: f(&args,"offset_x",0.0), offset_y: f(&args,"offset_y",0.0), offset_z: f(&args,"offset_z",0.0) },
            "mirror" => ModificationOp::MirrorAxis { axis: s(&args, "axis").chars().next().unwrap_or('X') },
            o => return Err(format!("Unknown operation: {}", o)),
        };
        Ok(Value::String(modifier::modify(&gcode, &op)))
    }));
}

fn register_insert_pause(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "insert_pause".into(),
        description: "Insert pause command (M0 or M600 filament change) at a specific layer".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode":{"type":"string"}, "layer":{"type":"integer"},
            "use_m600":{"type":"boolean","description":"Use M600 filament change instead of M0 pause. Default false"}
        },"required":["gcode","layer"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        Ok(Value::String(modifier::insert_pause(&gcode, u(&args, "layer", 0) as usize, b(&args, "use_m600", false))))
    }));
}

fn register_change_layer_settings(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "change_layer_settings".into(),
        description: "Change speed, temperature, or fan at a specific layer".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode":{"type":"string"}, "layer":{"type":"integer"},
            "setting":{"type":"string","enum":["speed","temperature","fan"]},
            "speed_multiplier":{"type":"number","description":"Speed multiplier (e.g. 0.5 for half speed)"},
            "temperature":{"type":"number","description":"New temperature °C"},
            "fan_percent":{"type":"integer","description":"Fan speed 0-100%"}
        },"required":["gcode","layer","setting"]}),
    }, Box::new(|args: Value| {
        let gcode = s(&args, "gcode");
        let layer = u(&args, "layer", 0) as usize;
        let op = match s(&args, "setting").as_str() {
            "speed" => ModificationOp::ChangeSpeedAtLayer { layer, speed_multiplier: f(&args, "speed_multiplier", 1.0) },
            "temperature" => ModificationOp::ChangeTempAtLayer { layer, temp: f(&args, "temperature", 200.0) },
            "fan" => ModificationOp::ChangeFanAtLayer { layer, fan_speed: u(&args, "fan_percent", 100) as u8 },
            s => return Err(format!("Unknown setting: {}", s)),
        };
        Ok(Value::String(modifier::modify(&gcode, &op)))
    }));
}

fn register_add_progress(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "add_progress_reporting".into(),
        description: "Add M73 progress percentage commands throughout the G-code for display progress".into(),
        input_schema: json!({"type":"object","properties":{"gcode":{"type":"string"}},"required":["gcode"]}),
    }, Box::new(|args: Value| {
        Ok(Value::String(modifier::add_progress(&s(&args, "gcode"))))
    }));
}

fn register_convert_extrusion_mode(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "convert_extrusion_mode".into(),
        description: "Convert G-code between absolute (M82) and relative (M83) extrusion modes".into(),
        input_schema: json!({"type":"object","properties":{
            "gcode":{"type":"string"},
            "to_relative":{"type":"boolean","description":"true = convert to relative (M83), false = convert to absolute (M82)"}
        },"required":["gcode","to_relative"]}),
    }, Box::new(|args: Value| {
        Ok(Value::String(modifier::convert_extrusion_mode(&s(&args, "gcode"), b(&args, "to_relative", true))))
    }));
}

fn register_strip_comments(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "strip_comments".into(),
        description: "Remove all comments from G-code, reducing file size".into(),
        input_schema: json!({"type":"object","properties":{"gcode":{"type":"string"}},"required":["gcode"]}),
    }, Box::new(|args: Value| {
        Ok(Value::String(modifier::strip_comments(&s(&args, "gcode"))))
    }));
}

fn register_lookup_printer(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "lookup_printer".into(),
        description: "Look up a printer profile by ID. Returns bed size, speeds, extruder type, etc. Use 'list' as ID to see all available printers.".into(),
        input_schema: json!({"type":"object","properties":{
            "printer_id":{"type":"string","description":"Printer ID or 'list' for all printers"}
        },"required":["printer_id"]}),
    }, Box::new(|args: Value| {
        let id = s(&args, "printer_id");
        if id == "list" {
            let list: Vec<_> = printers::all_printers().iter().map(|p| json!({"id": p.id, "name": p.name, "manufacturer": p.manufacturer})).collect();
            Ok(json!(list))
        } else {
            printers::get_printer(&id).map(|p| serde_json::to_value(&p).unwrap_or(json!({}))).ok_or_else(|| format!("Unknown printer: {}. Use 'list' to see available printers.", id))
        }
    }));
}

fn register_lookup_material(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "lookup_material".into(),
        description: "Look up a material profile by ID. Returns temperatures, speeds, retraction settings. Use 'list' as ID to see all materials.".into(),
        input_schema: json!({"type":"object","properties":{
            "material_id":{"type":"string","description":"Material ID or 'list' for all materials"}
        },"required":["material_id"]}),
    }, Box::new(|args: Value| {
        let id = s(&args, "material_id");
        if id == "list" {
            let list: Vec<_> = materials::all_materials().iter().map(|m| json!({"id": m.id, "name": m.name})).collect();
            Ok(json!(list))
        } else {
            materials::get_material(&id).map(|m| serde_json::to_value(&m).unwrap_or(json!({}))).ok_or_else(|| format!("Unknown material: {}. Use 'list' to see available materials.", id))
        }
    }));
}

fn register_explain_command(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "explain_gcode_command".into(),
        description: "Explain what a specific G-code command does, including its parameters".into(),
        input_schema: json!({"type":"object","properties":{
            "command":{"type":"string","description":"G-code command to explain, e.g. 'G1 X10 Y20 E0.5 F1500' or just 'M109'"}
        },"required":["command"]}),
    }, Box::new(|args: Value| {
        Ok(Value::String(reference::explain_command(&s(&args, "command"))))
    }));
}

fn register_calculate_extrusion(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "calculate_extrusion".into(),
        description: "Calculate the E (extrusion) value for a given move distance, nozzle size, and layer height".into(),
        input_schema: json!({"type":"object","properties":{
            "distance_mm":{"type":"number","description":"Move distance in mm"},
            "nozzle_diameter":{"type":"number","description":"Nozzle diameter mm. Default 0.4"},
            "layer_height":{"type":"number","description":"Layer height mm. Default 0.2"},
            "filament_diameter":{"type":"number","description":"Filament diameter mm. Default 1.75"},
            "flow_multiplier":{"type":"number","description":"Flow rate multiplier. Default 1.0"}
        },"required":["distance_mm"]}),
    }, Box::new(|args: Value| {
        let dist = f(&args, "distance_mm", 10.0);
        let nozzle = f(&args, "nozzle_diameter", 0.4);
        let lh = f(&args, "layer_height", 0.2);
        let fd = f(&args, "filament_diameter", 1.75);
        let flow = f(&args, "flow_multiplier", 1.0);
        let extrusion_width = nozzle * 1.2;
        let cross_section = lh * extrusion_width;
        let filament_area = PI * (fd / 2.0).powi(2);
        let e_value = (dist * cross_section / filament_area) * flow;
        Ok(json!({"e_value": (e_value * 10000.0).round() / 10000.0, "distance_mm": dist, "extrusion_width_mm": extrusion_width, "cross_section_mm2": cross_section, "filament_area_mm2": filament_area}))
    }));
}

fn register_convert_units(server: &mut McpServer) {
    server.add_tool(ToolDef {
        name: "convert_units".into(),
        description: "Convert between units: mm_min_to_mm_s, mm_s_to_mm_min, celsius_to_fahrenheit, fahrenheit_to_celsius, inches_to_mm, mm_to_inches".into(),
        input_schema: json!({"type":"object","properties":{
            "value":{"type":"number","description":"Value to convert"},
            "conversion":{"type":"string","enum":["mm_min_to_mm_s","mm_s_to_mm_min","celsius_to_fahrenheit","fahrenheit_to_celsius","inches_to_mm","mm_to_inches"]}
        },"required":["value","conversion"]}),
    }, Box::new(|args: Value| {
        let val = f(&args, "value", 0.0);
        let (result, from_unit, to_unit) = match s(&args, "conversion").as_str() {
            "mm_min_to_mm_s" => (val / 60.0, "mm/min", "mm/s"),
            "mm_s_to_mm_min" => (val * 60.0, "mm/s", "mm/min"),
            "celsius_to_fahrenheit" => (val * 9.0/5.0 + 32.0, "°C", "°F"),
            "fahrenheit_to_celsius" => ((val - 32.0) * 5.0/9.0, "°F", "°C"),
            "inches_to_mm" => (val * 25.4, "in", "mm"),
            "mm_to_inches" => (val / 25.4, "mm", "in"),
            c => return Err(format!("Unknown conversion: {}", c)),
        };
        Ok(json!({"input": val, "input_unit": from_unit, "result": (result * 10000.0).round() / 10000.0, "result_unit": to_unit}))
    }));
}
