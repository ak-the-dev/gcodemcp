/// G-code command reference
pub fn command_reference() -> String {
    r#"# G-code Command Reference for FDM 3D Printing

## Movement Commands
| Command | Description | Parameters |
|---------|-------------|------------|
| G0 | Rapid move (non-printing travel) | X Y Z F |
| G1 | Linear move (print/travel) | X Y Z E F |
| G2 | Clockwise arc | X Y I J E F |
| G3 | Counter-clockwise arc | X Y I J E F |
| G28 | Home axes | X Y Z (optional, homes all if none) |
| G29 | Auto bed leveling | |
| G30 | Single Z-probe | X Y |

## Positioning
| Command | Description |
|---------|-------------|
| G90 | Absolute positioning |
| G91 | Relative positioning |
| G92 | Set position (e.g., G92 E0 resets extruder) |

## Temperature Control
| Command | Description | Parameters |
|---------|-------------|------------|
| M104 | Set hotend temp (no wait) | S(temp) T(extruder) |
| M109 | Set hotend temp (wait) | S(temp) R(temp) T(extruder) |
| M140 | Set bed temp (no wait) | S(temp) |
| M190 | Set bed temp (wait) | S(temp) R(temp) |

## Fan Control
| Command | Description | Parameters |
|---------|-------------|------------|
| M106 | Set fan speed | S(0-255) P(fan index) |
| M107 | Fan off | P(fan index) |

## Extrusion
| Command | Description |
|---------|-------------|
| M82 | Absolute extrusion mode |
| M83 | Relative extrusion mode |
| G10 | Firmware retract |
| G11 | Firmware unretract |

## Print Control
| Command | Description | Parameters |
|---------|-------------|------------|
| M0 | Pause/stop | |
| M600 | Filament change | |
| M73 | Set print progress | P(percent) R(remaining min) |
| M84 | Disable steppers | |
| M17 | Enable steppers | |
| M211 | Software endstops | S(0=off, 1=on) |

## Speed/Acceleration
| Command | Description | Parameters |
|---------|-------------|------------|
| M203 | Max feedrate | X Y Z E (mm/s) |
| M204 | Acceleration | P(print) T(travel) R(retract) |
| M205 | Jerk settings | X Y Z E |
| M220 | Speed factor override | S(percent) |
| M221 | Flow rate factor | S(percent) |
| M900 | Linear advance | K(factor) |

## EEPROM
| Command | Description |
|---------|-------------|
| M500 | Save settings |
| M501 | Load settings |
| M502 | Factory reset |
| M503 | Report settings |

## Parameters
- **X, Y, Z**: Axis coordinates (mm)
- **E**: Extruder position/delta (mm)
- **F**: Feed rate (mm/min). Divide by 60 for mm/s
- **S**: Parameter value (temp in °C, fan 0-255, etc.)
- **T**: Tool/extruder index
- **P**: Various (fan index, time in ms, etc.)
- **I, J**: Arc center offsets (relative to start)
"#
    .to_string()
}

/// Troubleshooting guide
pub fn troubleshooting_guide() -> String {
    r#"# 3D Print Troubleshooting Guide

## Stringing / Oozing
- **Cause**: Filament leaking during travel moves
- **Fixes**: Increase retraction distance (0.5-2mm direct, 4-7mm bowden), increase retraction speed (25-50mm/s), lower hotend temp by 5-10°C, enable coasting, increase travel speed

## Layer Adhesion Issues
- **Cause**: Layers not bonding properly
- **Fixes**: Increase hotend temp by 5-10°C, decrease layer height, decrease fan speed, increase flow rate, slow print speed

## Warping
- **Cause**: Part lifting from bed
- **Fixes**: Increase bed temp, use brim/raft, enclose printer, reduce fan for first layers, use adhesion aid (glue stick, hairspray)

## Elephant's Foot (First Layer Bulge)
- **Cause**: First layer squished too much
- **Fixes**: Raise Z-offset slightly, lower bed temp for first layer, lower first layer flow rate

## Under-extrusion
- **Cause**: Not enough filament being deposited
- **Fixes**: Increase flow rate, check for clogs, calibrate e-steps, increase temp, check filament diameter, check for grinding

## Over-extrusion
- **Cause**: Too much filament being deposited
- **Fixes**: Decrease flow rate, calibrate e-steps, check filament diameter

## Z-Banding / Lines on Walls
- **Cause**: Inconsistent layer heights
- **Fixes**: Check Z-axis (lead screw, V-wheels), use anti-backlash nut, check belt tension, ensure proper frame alignment

## Blobs / Zits
- **Cause**: Extra material at layer changes
- **Fixes**: Enable coasting, tune retraction, randomize seam position, tune linear advance (K factor)

## Bridging Failure
- **Cause**: Filament sagging in unsupported areas
- **Fixes**: Increase fan to 100%, decrease bridge speed (10-20mm/s), decrease temp slightly, optimize bridge direction

## Clogging
- **Cause**: Filament jammed in hotend
- **Fixes**: Cold pull, check PTFE tube (all-metal gap), dry filament, check hotend temp, replace nozzle

## Layer Shifting
- **Cause**: Print shifted mid-print
- **Fixes**: Check belt tension, reduce speed/acceleration, check stepper current, check for mechanical binding
"#.to_string()
}

/// Explain a specific G-code command
pub fn explain_command(command: &str) -> String {
    let cmd = command.trim().to_uppercase();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let base = parts.first().map(|s| s.as_ref()).unwrap_or("");

    let explanation = match base {
        "G0" => "**G0 - Rapid Move**: Moves the print head to the specified coordinates at maximum travel speed without extruding. Used for non-printing repositioning. Parameters: X(pos) Y(pos) Z(pos) F(speed mm/min)",
        "G1" => "**G1 - Linear Move**: Moves the print head linearly to the specified coordinates. Can extrude filament if E parameter is provided. This is the primary command for both printing and non-printing moves. Parameters: X(pos) Y(pos) Z(pos) E(extrusion) F(speed mm/min)",
        "G28" => "**G28 - Home**: Homes all specified axes (or all axes if none specified) by moving them to their endstops. This establishes the coordinate origin. Should always be the first movement command.",
        "G29" => "**G29 - Auto Bed Leveling**: Probes the bed at multiple points to create a mesh compensation map. Requires a bed probe (BLTouch, inductive, etc.). Run after G28.",
        "G90" => "**G90 - Absolute Positioning**: All subsequent coordinates are interpreted as absolute positions relative to the origin. This is the default and most common mode.",
        "G91" => "**G91 - Relative Positioning**: All subsequent coordinates are interpreted as offsets from the current position. Useful for Z-hops and retractions.",
        "G92" => "**G92 - Set Position**: Sets the current position to the specified values without moving. Most commonly used as 'G92 E0' to reset the extruder position.",
        "M82" => "**M82 - Absolute Extrusion**: E values represent absolute positions. Each E value is the total filament extruded since last reset.",
        "M83" => "**M83 - Relative Extrusion**: E values represent incremental amounts. Each E value is the amount to extrude for that move only.",
        "M104" => "**M104 - Set Hotend Temperature**: Sets the target hotend temperature and continues executing. Does NOT wait for temp to be reached. Parameters: S(target °C) T(extruder index)",
        "M109" => "**M109 - Set Hotend Temperature and Wait**: Sets the target hotend temperature and waits until it's reached before continuing. Parameters: S(target °C, heat only) R(target °C, heat and cool)",
        "M140" => "**M140 - Set Bed Temperature**: Sets the target bed temperature and continues executing. Does NOT wait. Parameters: S(target °C)",
        "M190" => "**M190 - Set Bed Temperature and Wait**: Sets the target bed temperature and waits until reached. Parameters: S(target °C, heat only) R(target °C, heat and cool)",
        "M106" => "**M106 - Set Fan Speed**: Sets the part cooling fan speed. Parameters: S(speed 0-255, where 255 = 100%) P(fan index, default 0)",
        "M107" => "**M107 - Fan Off**: Turns off the part cooling fan. Equivalent to M106 S0.",
        "M0" => "**M0 - Unconditional Stop**: Pauses the print and waits for user input (button press on LCD) to continue. Used for manual filament changes or inspection.",
        "M600" => "**M600 - Filament Change**: Initiates a filament change sequence. Retracts filament, moves head to a park position, waits for user to load new filament.",
        "M84" => "**M84 - Disable Steppers**: Disables all stepper motors. Print head and bed can be moved by hand. Typically used at end of print.",
        "M73" => "**M73 - Set Print Progress**: Reports print progress to the display. Parameters: P(percent 0-100) R(remaining minutes)",
        "M220" => "**M220 - Set Speed Factor**: Overrides all feed rates by a percentage. M220 S50 = 50% speed. M220 S100 = normal. Useful for live tuning.",
        "M221" => "**M221 - Set Flow Rate Factor**: Overrides extrusion rate by a percentage. M221 S110 = 110% flow. Useful for live flow calibration.",
        "M900" => "**M900 - Linear Advance**: Sets the Linear Advance K factor for pressure compensation. Higher K = more compensation. Start at 0 and tune up. Parameters: K(factor)",
        "M204" => "**M204 - Set Acceleration**: Sets default acceleration values. Parameters: P(print accel mm/s²) T(travel accel mm/s²) R(retract accel mm/s²)",
        "M205" => "**M205 - Set Jerk/Junction Deviation**: Controls instantaneous speed changes at direction changes. Parameters depend on firmware (jerk: X Y Z E in mm/s, junction deviation: J in mm)",
        "M500" => "**M500 - Save Settings**: Saves all current settings to EEPROM/flash. Persists across power cycles.",
        "M503" => "**M503 - Report Settings**: Prints all current firmware settings to serial output. Useful for checking current configuration.",
        _ => {
            return format!("Command '{}' not found in reference database. Common G-code prefixes: G = geometry/movement, M = miscellaneous/machine. Full command list available via the gcode://reference/commands resource.", command);
        }
    };

    // If there are parameters, try to explain them
    if parts.len() > 1 {
        let param_explanations: Vec<String> = parts[1..]
            .iter()
            .map(|p| {
                let ch = p.chars().next().unwrap_or(' ');
                let val = &p[1..];
                match ch {
                    'X' => format!("X{} = X position {}mm", val, val),
                    'Y' => format!("Y{} = Y position {}mm", val, val),
                    'Z' => format!("Z{} = Z height {}mm", val, val),
                    'E' => format!("E{} = Extrude {}mm of filament", val, val),
                    'F' => format!(
                        "F{} = Feed rate {}mm/min ({:.1}mm/s)",
                        val,
                        val,
                        val.parse::<f64>().unwrap_or(0.0) / 60.0
                    ),
                    'S' => format!("S{} = Value {} (temperature °C or speed 0-255)", val, val),
                    'P' => format!("P{} = Parameter {}", val, val),
                    'T' => format!("T{} = Tool/extruder index {}", val, val),
                    _ => format!("{} = parameter", p),
                }
            })
            .collect();
        format!(
            "{}\n\n**Parameters in your command:**\n{}",
            explanation,
            param_explanations.join("\n")
        )
    } else {
        explanation.to_string()
    }
}
