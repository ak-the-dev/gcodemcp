use super::machine::{
    apply_command, classify_motion, is_motion_command, target_position, MachineState, MotionKind,
    EPSILON,
};
use super::parser;
use super::types::*;

/// Optimize G-code with provided options
pub fn optimize(input: &str, opts: &OptimizationOptions) -> String {
    let mut file = parser::parse(input);

    if opts.optimize_travel {
        remove_noop_travel_moves(&mut file);
    }
    if opts.optimize_retraction {
        optimize_retractions(&mut file, opts.min_travel_for_retract);
    }
    if opts.add_z_hop {
        add_z_hop(&mut file, opts.z_hop_height);
    }
    if opts.add_coasting {
        add_coasting(&mut file, opts.coasting_distance);
    }
    if opts.optimize_speed {
        optimize_speeds(&mut file);
    }

    parser::format_file(&file)
}

fn remove_noop_travel_moves(file: &mut GCodeFile) {
    let mut state = MachineState::default();

    for line in &mut file.lines {
        let Some(cmd) = line.command.clone() else {
            continue;
        };

        if state.absolute_positioning
            && is_motion_command(&cmd)
            && !cmd.params.contains_key(&'E')
            && !cmd.params.contains_key(&'F')
            && (cmd.params.contains_key(&'X')
                || cmd.params.contains_key(&'Y')
                || cmd.params.contains_key(&'Z'))
            && matches!(classify_motion(&cmd, &state), MotionKind::Stationary)
        {
            line.command = None;
            line.comment = Some("removed no-op travel move".into());
            continue;
        }

        apply_command(&cmd, &mut state);
    }
}

/// Remove unnecessary retractions when the following travel distance is very short.
fn optimize_retractions(file: &mut GCodeFile, min_travel: f64) {
    let mut state = MachineState::default();

    for index in 0..file.lines.len() {
        let Some(cmd) = file.lines[index].command.clone() else {
            continue;
        };

        if matches!(classify_motion(&cmd, &state), MotionKind::Retract) {
            let mut after_retract = state;
            apply_command(&cmd, &mut after_retract);

            if let Some(unretract_index) =
                find_matching_unretract(file, index + 1, after_retract, min_travel)
            {
                file.lines[index].command = None;
                file.lines[index].comment = Some("removed short retraction".into());
                file.lines[unretract_index].command = None;
                file.lines[unretract_index].comment = Some("removed unretract".into());
            }
        }

        apply_command(&cmd, &mut state);
    }
}

fn find_matching_unretract(
    file: &GCodeFile,
    start: usize,
    mut state: MachineState,
    min_travel: f64,
) -> Option<usize> {
    let mut travel_distance = 0.0_f64;

    for index in start..file.lines.len() {
        let Some(cmd) = file.lines[index].command.as_ref() else {
            continue;
        };

        match classify_motion(cmd, &state) {
            MotionKind::Travel { distance } => {
                travel_distance += distance;
                if travel_distance >= min_travel {
                    return None;
                }
            }
            MotionKind::ExtrudeOnly { amount } if amount > EPSILON => return Some(index),
            MotionKind::Print { .. } | MotionKind::Retract => return None,
            MotionKind::ExtrudeOnly { .. } | MotionKind::Stationary => {}
        }

        apply_command(cmd, &mut state);
    }

    None
}

/// Add an absolute Z-hop before and after travel moves.
fn add_z_hop(file: &mut GCodeFile, hop_height: f64) {
    if hop_height <= 0.0 {
        return;
    }

    let mut state = MachineState::default();
    let mut new_lines = Vec::with_capacity(file.lines.len());

    for line in &file.lines {
        let base_z = state.z;
        let should_hop = line.command.as_ref().is_some_and(|cmd| {
            state.absolute_positioning
                && base_z > EPSILON
                && !cmd.params.contains_key(&'Z')
                && !cmd.params.contains_key(&'E')
                && (cmd.params.contains_key(&'X') || cmd.params.contains_key(&'Y'))
                && matches!(classify_motion(cmd, &state), MotionKind::Travel { distance } if distance > EPSILON)
        });

        if should_hop {
            new_lines.push(make_absolute_z_move(base_z + hop_height, "z-hop up"));
        }

        new_lines.push(line.clone());

        if let Some(cmd) = line.command.as_ref() {
            apply_command(cmd, &mut state);
        }

        if should_hop {
            new_lines.push(make_absolute_z_move(base_z, "z-hop down"));
        }
    }

    file.total_lines = new_lines.len();
    file.lines = new_lines;
}

/// Reduce extrusion at the end of a perimeter when a travel/retract follows.
fn add_coasting(file: &mut GCodeFile, coast_dist: f64) {
    if coast_dist <= 0.0 {
        return;
    }

    let mut state = MachineState::default();

    for index in 0..file.lines.len() {
        let Some(original_cmd) = file.lines[index].command.clone() else {
            continue;
        };

        let mut updated_cmd = original_cmd.clone();

        if let MotionKind::Print {
            distance,
            extrusion,
        } = classify_motion(&original_cmd, &state)
        {
            let mut after_move = state;
            apply_command(&original_cmd, &mut after_move);

            if distance > EPSILON
                && next_transition_is_travel_or_retract(file, index + 1, after_move)
            {
                let reduction = (extrusion * (coast_dist / distance)).min(extrusion * 0.8);
                if reduction > EPSILON {
                    if let Some(e) = updated_cmd.params.get_mut(&'E') {
                        if state.absolute_extrusion {
                            *e = (*e - reduction).max(state.e + EPSILON);
                        } else {
                            *e = (*e - reduction).max(EPSILON);
                        }
                    }
                    file.lines[index].command = Some(updated_cmd.clone());
                }
            }
        }

        let state_cmd = file.lines[index]
            .command
            .as_ref()
            .unwrap_or(&updated_cmd)
            .clone();
        apply_command(&state_cmd, &mut state);
    }
}

fn next_transition_is_travel_or_retract(
    file: &GCodeFile,
    start: usize,
    mut state: MachineState,
) -> bool {
    for index in start..file.lines.len() {
        let Some(cmd) = file.lines[index].command.as_ref() else {
            continue;
        };

        match classify_motion(cmd, &state) {
            MotionKind::Travel { distance } if distance > EPSILON => return true,
            MotionKind::Retract => return true,
            MotionKind::Print { .. } | MotionKind::ExtrudeOnly { .. } => return false,
            MotionKind::Travel { .. } | MotionKind::Stationary => {}
        }

        apply_command(cmd, &mut state);
    }

    false
}

/// Cap first-layer print moves to a conservative feedrate.
fn optimize_speeds(file: &mut GCodeFile) {
    let mut state = MachineState::default();

    for line in &mut file.lines {
        let Some(cmd) = line.command.clone() else {
            continue;
        };

        if matches!(classify_motion(&cmd, &state), MotionKind::Print { .. }) {
            let print_z = target_position(&cmd, &state).2;
            if print_z <= 0.3 {
                if let Some(updated_cmd) = line.command.as_mut() {
                    if let Some(feedrate) = updated_cmd.params.get_mut(&'F') {
                        if *feedrate > 1200.0 {
                            *feedrate = 1200.0;
                        }
                    }
                }
            }
        }

        let state_cmd = line.command.as_ref().unwrap_or(&cmd).clone();
        apply_command(&state_cmd, &mut state);
    }
}

fn make_absolute_z_move(z: f64, comment: &str) -> GCodeLine {
    let mut params = std::collections::HashMap::new();
    params.insert('Z', z);
    params.insert('F', 3000.0);
    GCodeLine {
        line_number: 0,
        raw: String::new(),
        command: Some(GCodeCommand {
            letter: 'G',
            number: 1,
            params,
        }),
        comment: Some(comment.into()),
    }
}

/// Suggest optimal speed profile for printer+material
pub fn suggest_speed_profile(
    printer: &PrinterProfile,
    material: &MaterialProfile,
) -> serde_json::Value {
    let max_speed = printer.max_print_speed.min(material.print_speed_max);
    serde_json::json!({
        "first_layer_speed_mm_s": (max_speed * 0.4).round().max(15.0),
        "perimeter_speed_mm_s": (max_speed * 0.7).round(),
        "infill_speed_mm_s": max_speed.round(),
        "top_bottom_speed_mm_s": (max_speed * 0.6).round(),
        "travel_speed_mm_s": printer.max_travel_speed.min(200.0).round(),
        "retraction_speed_mm_s": material.retraction_speed,
        "retraction_distance_mm": material.retraction_distance,
        "outer_wall_speed_mm_s": (max_speed * 0.5).round(),
        "inner_wall_speed_mm_s": (max_speed * 0.7).round(),
        "bridge_speed_mm_s": (max_speed * 0.35).round().max(10.0),
        "support_speed_mm_s": max_speed.round(),
        "first_layer_travel_mm_s": (printer.max_travel_speed * 0.5).min(100.0).round(),
        "acceleration_mm_s2": printer.max_acceleration.min(3000.0).round(),
        "notes": format!("Optimized for {} on {}. Adjust based on print results.", material.name, printer.name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_short_relative_retraction_pairs() {
        let gcode = "\
M83
G1 X0 Y0 Z0.2 F600
G1 X10 E1.0 F1200
G1 E-0.8 F1800
G1 X10.4 Y0 F3000
G1 E0.8 F1800
G1 X20 E1.0 F1200
";
        let optimized = optimize(
            gcode,
            &OptimizationOptions {
                optimize_travel: false,
                optimize_retraction: true,
                optimize_speed: false,
                add_z_hop: false,
                z_hop_height: 0.4,
                add_coasting: false,
                coasting_distance: 0.3,
                min_travel_for_retract: 2.0,
            },
        );

        assert!(optimized.contains(";removed short retraction"));
        assert!(optimized.contains(";removed unretract"));
    }

    #[test]
    fn adds_balanced_z_hop_moves() {
        let gcode = "\
G90
G1 Z0.2 F600
G1 X10 Y10 F3000
G1 X20 Y20 E1.0 F1200
";
        let optimized = optimize(
            gcode,
            &OptimizationOptions {
                optimize_travel: false,
                optimize_retraction: false,
                optimize_speed: false,
                add_z_hop: true,
                z_hop_height: 0.4,
                add_coasting: false,
                coasting_distance: 0.3,
                min_travel_for_retract: 1.5,
            },
        );

        assert!(optimized.contains("G1 Z0.6000 F3000 ;z-hop up"));
        assert!(optimized.contains("G1 Z0.2000 F3000 ;z-hop down"));
    }
}
