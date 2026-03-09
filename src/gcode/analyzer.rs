use super::machine::{
    apply_command, classify_motion, is_motion_command, target_position, MachineState, MotionKind,
    EPSILON,
};
use super::parser;
use super::types::*;

pub fn analyze(input: &str) -> AnalysisResult {
    let file = parser::parse(input);
    let mut result = AnalysisResult {
        total_lines: file.total_lines,
        command_lines: 0,
        comment_lines: 0,
        blank_lines: 0,
        total_layers: 0,
        estimated_time_seconds: 0.0,
        filament_used_mm: 0.0,
        filament_used_grams: 0.0,
        bounding_box: BoundingBox {
            min_x: f64::MAX,
            max_x: f64::MIN,
            min_y: f64::MAX,
            max_y: f64::MIN,
            min_z: f64::MAX,
            max_z: f64::MIN,
        },
        speed_stats: SpeedStats {
            min_print_speed: f64::MAX,
            max_print_speed: 0.0,
            avg_print_speed: 0.0,
            min_travel_speed: f64::MAX,
            max_travel_speed: 0.0,
            avg_travel_speed: 0.0,
        },
        total_retractions: 0,
        total_travel_distance: 0.0,
        total_print_distance: 0.0,
        temperature_events: Vec::new(),
        layers: Vec::new(),
        issues: Vec::new(),
    };

    let mut state = MachineState::default();
    let mut current_feed_rate = 1000.0_f64;

    let mut layer_start = 0usize;
    let mut layer_extrusion = 0.0_f64;
    let mut layer_travel = 0.0_f64;
    let mut layer_print = 0.0_f64;
    let mut layer_retractions = 0usize;
    let mut current_layer_z: Option<f64> = None;
    let mut pending_layer: Option<(f64, usize)> = None;

    let mut print_speed_sum = 0.0_f64;
    let mut print_speed_count = 0u64;
    let mut travel_speed_sum = 0.0_f64;
    let mut travel_speed_count = 0u64;

    let mut has_homing = false;
    let mut has_hotend_temp = false;
    let mut has_bed_temp = false;

    for line in &file.lines {
        if line.command.is_none() && line.comment.is_none() {
            result.blank_lines += 1;
            continue;
        }
        if line.command.is_none() {
            result.comment_lines += 1;
            continue;
        }

        result.command_lines += 1;
        let cmd = match &line.command {
            Some(cmd) => cmd,
            None => continue,
        };

        if is_motion_command(cmd) && cmd.params.contains_key(&'Z') {
            let target_z = target_position(cmd, &state).2;
            match current_layer_z {
                Some(active_z) if target_z > active_z + EPSILON => {
                    pending_layer = Some((target_z, line.line_number));
                }
                Some(active_z) if target_z <= active_z + EPSILON => {
                    pending_layer = None;
                }
                None if target_z > 0.0 => {
                    pending_layer = Some((target_z, line.line_number));
                }
                _ => {}
            }
        }

        let motion = classify_motion(cmd, &state);
        let feed_rate = cmd.params.get(&'F').copied().unwrap_or(current_feed_rate);
        let speed_mm_s = if feed_rate > 0.0 {
            feed_rate / 60.0
        } else {
            0.0
        };

        match motion {
            MotionKind::Print {
                distance,
                extrusion,
            } => {
                let (_, _, target_z) = target_position(cmd, &state);

                match current_layer_z {
                    None => {
                        current_layer_z = Some(target_z);
                        layer_start = pending_layer
                            .filter(|(pending_z, _)| (*pending_z - target_z).abs() <= EPSILON)
                            .map_or(line.line_number, |(_, start)| start);
                        pending_layer = None;
                    }
                    Some(active_z) if target_z > active_z + EPSILON => {
                        result.layers.push(LayerInfo {
                            layer_number: result.layers.len(),
                            z_height: active_z,
                            line_start: layer_start,
                            line_end: line.line_number.saturating_sub(1),
                            extrusion_length: layer_extrusion,
                            travel_distance: layer_travel,
                            print_distance: layer_print,
                            num_retractions: layer_retractions,
                        });
                        current_layer_z = Some(target_z);
                        layer_start = pending_layer
                            .filter(|(pending_z, _)| (*pending_z - target_z).abs() <= EPSILON)
                            .map_or(line.line_number, |(_, start)| start);
                        layer_extrusion = 0.0;
                        layer_travel = 0.0;
                        layer_print = 0.0;
                        layer_retractions = 0;
                        pending_layer = None;
                    }
                    _ => {
                        pending_layer = None;
                    }
                }

                result.total_print_distance += distance;
                result.filament_used_mm += extrusion;
                layer_print += distance;
                layer_extrusion += extrusion;

                if speed_mm_s > 0.0 {
                    result.estimated_time_seconds += distance / speed_mm_s;
                    result.speed_stats.min_print_speed =
                        result.speed_stats.min_print_speed.min(speed_mm_s);
                    result.speed_stats.max_print_speed =
                        result.speed_stats.max_print_speed.max(speed_mm_s);
                    print_speed_sum += speed_mm_s;
                    print_speed_count += 1;
                }

                let (next_x, next_y, next_z) = target_position(cmd, &state);
                update_bounds(&mut result.bounding_box, state.x, state.y, state.z);
                update_bounds(&mut result.bounding_box, next_x, next_y, next_z);
            }
            MotionKind::Travel { distance } => {
                result.total_travel_distance += distance;
                layer_travel += distance;

                if speed_mm_s > 0.0 {
                    result.estimated_time_seconds += distance / speed_mm_s;
                    result.speed_stats.min_travel_speed =
                        result.speed_stats.min_travel_speed.min(speed_mm_s);
                    result.speed_stats.max_travel_speed =
                        result.speed_stats.max_travel_speed.max(speed_mm_s);
                    travel_speed_sum += speed_mm_s;
                    travel_speed_count += 1;
                }
            }
            MotionKind::Retract => {
                result.total_retractions += 1;
                layer_retractions += 1;
            }
            MotionKind::ExtrudeOnly { .. } | MotionKind::Stationary => {}
        }

        match (cmd.letter, cmd.number) {
            ('G', 28) => has_homing = true,
            ('M', 104) | ('M', 109) => {
                has_hotend_temp = true;
                if let Some(temp) = cmd
                    .params
                    .get(&'S')
                    .copied()
                    .or_else(|| cmd.params.get(&'R').copied())
                {
                    result.temperature_events.push(TemperatureEvent {
                        line_number: line.line_number,
                        target_type: "hotend".into(),
                        temperature: temp,
                        wait: cmd.number == 109,
                    });
                }
            }
            ('M', 140) | ('M', 190) => {
                has_bed_temp = true;
                if let Some(temp) = cmd
                    .params
                    .get(&'S')
                    .copied()
                    .or_else(|| cmd.params.get(&'R').copied())
                {
                    result.temperature_events.push(TemperatureEvent {
                        line_number: line.line_number,
                        target_type: "bed".into(),
                        temperature: temp,
                        wait: cmd.number == 190,
                    });
                }
            }
            _ => {}
        }

        if cmd.params.contains_key(&'F') {
            current_feed_rate = feed_rate;
        }
        apply_command(cmd, &mut state);
    }

    if let Some(active_z) = current_layer_z {
        result.layers.push(LayerInfo {
            layer_number: result.layers.len(),
            z_height: active_z,
            line_start: layer_start,
            line_end: file.total_lines,
            extrusion_length: layer_extrusion,
            travel_distance: layer_travel,
            print_distance: layer_print,
            num_retractions: layer_retractions,
        });
    }

    result.total_layers = result.layers.len();
    if print_speed_count > 0 {
        result.speed_stats.avg_print_speed = print_speed_sum / print_speed_count as f64;
    }
    if travel_speed_count > 0 {
        result.speed_stats.avg_travel_speed = travel_speed_sum / travel_speed_count as f64;
    }
    if result.speed_stats.min_print_speed == f64::MAX {
        result.speed_stats.min_print_speed = 0.0;
    }
    if result.speed_stats.min_travel_speed == f64::MAX {
        result.speed_stats.min_travel_speed = 0.0;
    }
    if result.bounding_box.min_x == f64::MAX {
        result.bounding_box = BoundingBox {
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
            min_z: 0.0,
            max_z: 0.0,
        };
    }

    let filament_area = std::f64::consts::PI * 0.875_f64.powi(2);
    result.filament_used_grams = (result.filament_used_mm * filament_area / 1000.0) * 1.24;

    if !has_homing {
        result.issues.push(GCodeIssue {
            severity: "warning".into(),
            line_number: None,
            message: "No homing command (G28) found".into(),
            suggestion: "Add G28 at the start".into(),
        });
    }
    if !has_hotend_temp {
        result.issues.push(GCodeIssue {
            severity: "error".into(),
            line_number: None,
            message: "No hotend temperature set".into(),
            suggestion: "Add M104 or M109".into(),
        });
    }
    if !has_bed_temp {
        result.issues.push(GCodeIssue {
            severity: "info".into(),
            line_number: None,
            message: "No bed temperature set".into(),
            suggestion: "Add M140 or M190 if the printer has a heated bed".into(),
        });
    }
    if result.total_retractions > 500 {
        result.issues.push(GCodeIssue {
            severity: "warning".into(),
            line_number: None,
            message: format!("High retraction count: {}", result.total_retractions),
            suggestion: "Increase the minimum travel distance for retraction".into(),
        });
    }

    result
}

pub fn validate(input: &str) -> Vec<GCodeIssue> {
    let analysis = analyze(input);
    let mut issues = analysis.issues;
    let file = parser::parse(input);

    for line in &file.lines {
        if let Some(cmd) = &line.command {
            if cmd.letter == 'M' && (cmd.number == 104 || cmd.number == 109) {
                if let Some(temp) = cmd
                    .params
                    .get(&'S')
                    .copied()
                    .or_else(|| cmd.params.get(&'R').copied())
                {
                    if temp > 300.0 {
                        issues.push(GCodeIssue {
                            severity: "error".into(),
                            line_number: Some(line.line_number),
                            message: format!("Hotend temp too high: {}°C", temp),
                            suggestion: "Keep hotend temperatures at or below about 300°C".into(),
                        });
                    }
                }
            }
            if cmd.letter == 'M' && (cmd.number == 140 || cmd.number == 190) {
                if let Some(temp) = cmd
                    .params
                    .get(&'S')
                    .copied()
                    .or_else(|| cmd.params.get(&'R').copied())
                {
                    if temp > 120.0 {
                        issues.push(GCodeIssue {
                            severity: "warning".into(),
                            line_number: Some(line.line_number),
                            message: format!("Bed temp very high: {}°C", temp),
                            suggestion: "Most FDM beds should stay at or below about 110°C".into(),
                        });
                    }
                }
            }
            if cmd.letter == 'G'
                && (cmd.number == 0 || cmd.number == 1)
                && cmd.params.get(&'F').copied().is_some_and(|f| f > 30000.0)
            {
                issues.push(GCodeIssue {
                    severity: "warning".into(),
                    line_number: Some(line.line_number),
                    message: format!(
                        "Feed rate very high: {} mm/min",
                        cmd.params.get(&'F').copied().unwrap_or_default()
                    ),
                    suggestion: "Most printers top out around 300 mm/s".into(),
                });
            }
        }
    }

    issues
}

pub fn get_layer_info(input: &str, layer: usize) -> Option<LayerInfo> {
    analyze(input)
        .layers
        .into_iter()
        .find(|layer_info| layer_info.layer_number == layer)
}

pub fn estimate_print_time(input: &str) -> f64 {
    analyze(input).estimated_time_seconds
}

pub fn calculate_filament(
    input: &str,
    density: f64,
    diameter: f64,
    cost_per_kg: f64,
) -> serde_json::Value {
    let analysis = analyze(input);
    let filament_area = std::f64::consts::PI * (diameter / 2.0).powi(2);
    let volume_cm3 = analysis.filament_used_mm * filament_area / 1000.0;
    let weight_g = volume_cm3 * density;

    serde_json::json!({
        "length_mm": analysis.filament_used_mm,
        "length_m": analysis.filament_used_mm / 1000.0,
        "volume_cm3": volume_cm3,
        "weight_g": weight_g,
        "cost": (weight_g / 1000.0) * cost_per_kg
    })
}

fn update_bounds(bounds: &mut BoundingBox, x: f64, y: f64, z: f64) {
    if x < bounds.min_x {
        bounds.min_x = x;
    }
    if x > bounds.max_x {
        bounds.max_x = x;
    }

    if y < bounds.min_y {
        bounds.min_y = y;
    }
    if y > bounds.max_y {
        bounds.max_y = y;
    }

    if z < bounds.min_z {
        bounds.min_z = z;
    }
    if z > bounds.max_z {
        bounds.max_z = z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_relative_positioning_and_extrusion() {
        let gcode = "\
M83
G91
G1 Z0.2 F600
G1 X10 E1.0 F1200
G1 Y10 E1.0
G1 E-0.5 F1800
G1 X5 F3000
G1 E0.5 F1800
";

        let analysis = analyze(gcode);

        assert_eq!(analysis.total_layers, 1);
        assert!((analysis.total_print_distance - 20.0).abs() < 0.01);
        assert!((analysis.filament_used_mm - 2.0).abs() < 0.01);
        assert_eq!(analysis.total_retractions, 1);
    }
}
