use super::machine::{apply_command, is_motion_command, MachineState};
use super::parser;
use super::types::*;

/// Apply a modification operation to G-code
pub fn modify(input: &str, op: &ModificationOp) -> String {
    match op {
        ModificationOp::SearchReplace { search, replace } => {
            input.replace(search.as_str(), replace.as_str())
        }
        ModificationOp::InsertAtLayer { layer, gcode } => insert_at_layer(input, *layer, gcode),
        ModificationOp::ScaleCoordinates {
            scale_x,
            scale_y,
            scale_z,
        } => scale_coords(input, *scale_x, *scale_y, *scale_z),
        ModificationOp::TranslateCoordinates {
            offset_x,
            offset_y,
            offset_z,
        } => translate_coords(input, *offset_x, *offset_y, *offset_z),
        ModificationOp::MirrorAxis { axis } => mirror(input, *axis),
        ModificationOp::ChangeSpeedAtLayer {
            layer,
            speed_multiplier,
        } => change_speed_at_layer(input, *layer, *speed_multiplier),
        ModificationOp::ChangeTempAtLayer { layer, temp } => {
            change_temp_at_layer(input, *layer, *temp)
        }
        ModificationOp::ChangeFanAtLayer { layer, fan_speed } => {
            change_fan_at_layer(input, *layer, *fan_speed)
        }
    }
}

/// Insert pause at layer
pub fn insert_pause(input: &str, layer: usize, use_m600: bool) -> String {
    let pause_cmd = if use_m600 {
        "M600 ;filament change\n"
    } else {
        "M0 ;pause print\n"
    };
    insert_at_layer(input, layer, pause_cmd)
}

/// Add M73 progress reporting
pub fn add_progress(input: &str) -> String {
    let file = parser::parse(input);
    let total = file.total_lines;
    let mut output = String::new();
    let mut last_pct = 0;

    for (i, line) in file.lines.iter().enumerate() {
        let pct = ((i as f64 / total as f64) * 100.0) as u32;
        if pct >= last_pct + 5 {
            output.push_str(&format!("M73 P{}\n", pct));
            last_pct = pct;
        }
        output.push_str(&line.raw);
        output.push('\n');
    }
    output.push_str("M73 P100\n");
    output
}

/// Convert between absolute and relative extrusion
pub fn convert_extrusion_mode(input: &str, to_relative: bool) -> String {
    let file = parser::parse(input);
    let mut output = String::new();
    let mut current_e = 0.0_f64;
    let mut is_relative = false;

    for line in &file.lines {
        if let Some(ref cmd) = line.command {
            match (cmd.letter, cmd.number) {
                ('M', 82) => {
                    is_relative = false;
                    if to_relative {
                        output.push_str("M83 ;relative extrusion\n");
                    } else {
                        output.push_str(&line.raw);
                        output.push('\n');
                    }
                    continue;
                }
                ('M', 83) => {
                    is_relative = true;
                    if !to_relative {
                        output.push_str("M82 ;absolute extrusion\n");
                    } else {
                        output.push_str(&line.raw);
                        output.push('\n');
                    }
                    continue;
                }
                ('G', 1) | ('G', 0) => {
                    if let Some(&e) = cmd.params.get(&'E') {
                        let mut new_cmd = cmd.clone();
                        if to_relative && !is_relative {
                            // Convert absolute to relative
                            let delta = e - current_e;
                            current_e = e;
                            new_cmd.params.insert('E', delta);
                        } else if !to_relative && is_relative {
                            // Convert relative to absolute
                            current_e += e;
                            new_cmd.params.insert('E', current_e);
                        }
                        output.push_str(&parser::format_command(&new_cmd));
                        if let Some(ref c) = line.comment {
                            output.push_str(&format!(" ;{}", c));
                        }
                        output.push('\n');
                        continue;
                    }
                }
                ('G', 92) => {
                    if let Some(&e) = cmd.params.get(&'E') {
                        current_e = e;
                    }
                }
                _ => {}
            }
        }
        output.push_str(&line.raw);
        output.push('\n');
    }
    output
}

/// Strip all comments
pub fn strip_comments(input: &str) -> String {
    let file = parser::parse(input);
    let mut output = String::new();
    for line in &file.lines {
        if let Some(ref cmd) = line.command {
            output.push_str(&parser::format_command(cmd));
            output.push('\n');
        } else if line.command.is_none() && line.comment.is_some() {
            // Skip comment-only lines
        } else {
            output.push('\n');
        }
    }
    output
}

fn insert_at_layer(input: &str, target_layer: usize, gcode: &str) -> String {
    let analysis = super::analyzer::analyze(input);
    let layer_info = analysis
        .layers
        .iter()
        .find(|l| l.layer_number == target_layer);

    let insert_line = match layer_info {
        Some(l) => l.line_start,
        None => return format!("; ERROR: Layer {} not found\n{}", target_layer, input),
    };

    let mut output = String::new();
    for (i, line) in input.lines().enumerate() {
        if i + 1 == insert_line {
            output.push_str(&format!("; --- Inserted at layer {} ---\n", target_layer));
            output.push_str(gcode);
            if !gcode.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("; --- End insert ---\n");
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}

fn scale_coords(input: &str, sx: f64, sy: f64, sz: f64) -> String {
    let mut file = parser::parse(input);
    let mut state = MachineState::default();
    for line in &mut file.lines {
        if let Some(cmd) = line.command.as_mut() {
            if state.absolute_positioning && is_motion_command(cmd) {
                if let Some(x) = cmd.params.get_mut(&'X') {
                    *x *= sx;
                }
                if let Some(y) = cmd.params.get_mut(&'Y') {
                    *y *= sy;
                }
                if let Some(z) = cmd.params.get_mut(&'Z') {
                    *z *= sz;
                }
            }
            let updated_cmd = cmd.clone();
            apply_command(&updated_cmd, &mut state);
        }
    }
    parser::format_file(&file)
}

fn translate_coords(input: &str, ox: f64, oy: f64, oz: f64) -> String {
    let mut file = parser::parse(input);
    let mut state = MachineState::default();
    for line in &mut file.lines {
        if let Some(cmd) = line.command.as_mut() {
            if state.absolute_positioning && is_motion_command(cmd) {
                if let Some(x) = cmd.params.get_mut(&'X') {
                    *x += ox;
                }
                if let Some(y) = cmd.params.get_mut(&'Y') {
                    *y += oy;
                }
                if let Some(z) = cmd.params.get_mut(&'Z') {
                    *z += oz;
                }
            }
            let updated_cmd = cmd.clone();
            apply_command(&updated_cmd, &mut state);
        }
    }
    parser::format_file(&file)
}

fn mirror(input: &str, axis: char) -> String {
    let analysis = super::analyzer::analyze(input);
    let bb = &analysis.bounding_box;
    let mut file = parser::parse(input);
    let mut state = MachineState::default();
    for line in &mut file.lines {
        if let Some(cmd) = line.command.as_mut() {
            if state.absolute_positioning && is_motion_command(cmd) {
                match axis {
                    'X' | 'x' => {
                        if let Some(x) = cmd.params.get_mut(&'X') {
                            *x = bb.max_x - (*x - bb.min_x);
                        }
                    }
                    'Y' | 'y' => {
                        if let Some(y) = cmd.params.get_mut(&'Y') {
                            *y = bb.max_y - (*y - bb.min_y);
                        }
                    }
                    'Z' | 'z' => {
                        if let Some(z) = cmd.params.get_mut(&'Z') {
                            *z = bb.max_z - (*z - bb.min_z);
                        }
                    }
                    _ => {}
                }
            }
            let updated_cmd = cmd.clone();
            apply_command(&updated_cmd, &mut state);
        }
    }
    parser::format_file(&file)
}

fn change_speed_at_layer(input: &str, target_layer: usize, multiplier: f64) -> String {
    let analysis = super::analyzer::analyze(input);
    let layer = match analysis
        .layers
        .iter()
        .find(|l| l.layer_number == target_layer)
    {
        Some(l) => l.clone(),
        None => return input.to_string(),
    };

    let mut file = parser::parse(input);
    for line in &mut file.lines {
        if line.line_number >= layer.line_start && line.line_number <= layer.line_end {
            if let Some(ref mut cmd) = line.command {
                if cmd.letter == 'G' && (cmd.number == 0 || cmd.number == 1) {
                    if let Some(f) = cmd.params.get_mut(&'F') {
                        *f *= multiplier;
                    }
                }
            }
        }
    }
    parser::format_file(&file)
}

fn change_temp_at_layer(input: &str, target_layer: usize, temp: f64) -> String {
    let analysis = super::analyzer::analyze(input);
    let layer = match analysis
        .layers
        .iter()
        .find(|l| l.layer_number == target_layer)
    {
        Some(l) => l.clone(),
        None => return input.to_string(),
    };

    let mut output = String::new();
    for (i, line) in input.lines().enumerate() {
        if i + 1 == layer.line_start {
            output.push_str(&format!(
                "M104 S{} ;temp change at layer {}\n",
                temp, target_layer
            ));
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}

fn change_fan_at_layer(input: &str, target_layer: usize, fan_speed: u8) -> String {
    let analysis = super::analyzer::analyze(input);
    let layer = match analysis
        .layers
        .iter()
        .find(|l| l.layer_number == target_layer)
    {
        Some(l) => l.clone(),
        None => return input.to_string(),
    };

    let mut output = String::new();
    for (i, line) in input.lines().enumerate() {
        if i + 1 == layer.line_start {
            let s = (fan_speed as f64 / 100.0 * 255.0).round() as u32;
            output.push_str(&format!(
                "M106 S{} ;fan {}% at layer {}\n",
                s, fan_speed, target_layer
            ));
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}
