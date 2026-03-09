use super::types::*;
use std::f64::consts::PI;

/// Generate start G-code for a given printer and material
pub fn start_gcode(printer: &PrinterProfile, material: &MaterialProfile) -> String {
    let setup_sequence = if printer.default_start_gcode.trim().is_empty() {
        if printer.auto_bed_leveling {
            "G28\nG29".to_string()
        } else {
            "G28".to_string()
        }
    } else {
        printer.default_start_gcode.trim().to_string()
    };
    let max_x = (printer.bed_x - 1.0).max(1.0);
    let max_y = (printer.bed_y - 5.0).max(10.0);
    let prime_x1 = (printer.nozzle_diameter * 0.5).clamp(0.1, max_x - 0.6);
    let prime_x2 = (prime_x1 + printer.nozzle_diameter * 0.75).min(max_x);
    let prime_y_start = (printer.bed_y * 0.08).clamp(5.0, max_y - 10.0);
    let prime_y_end = (printer.bed_y - 5.0).max(prime_y_start + 10.0);

    let mut g = String::new();
    g.push_str("; === Start G-code ===\n");
    g.push_str(&format!("; Printer: {}\n", printer.name));
    g.push_str(&format!("; Material: {}\n", material.name));
    g.push_str(&format!(
        "; Nozzle: {}°C, Bed: {}°C\n\n",
        material.nozzle_temp_default, material.bed_temp_default
    ));
    g.push_str(&format!(
        "M140 S{} ;set bed temp\n",
        material.bed_temp_default
    ));
    g.push_str(&format!(
        "M104 S{} ;set hotend temp\n",
        material.nozzle_temp_default
    ));
    g.push_str(&format!(
        "M190 S{} ;wait for bed\n",
        material.bed_temp_default
    ));
    g.push_str(&format!(
        "M109 S{} ;wait for hotend\n",
        material.nozzle_temp_default
    ));
    g.push_str(&setup_sequence);
    if !setup_sequence.ends_with('\n') {
        g.push('\n');
    }
    g.push_str("G90 ;absolute positioning\n");
    g.push_str("M82 ;absolute extrusion\n");
    g.push_str("G92 E0 ;reset extruder\n");
    // Prime line
    g.push_str("; --- Prime line ---\n");
    g.push_str("G1 Z2.0 F3000\n");
    g.push_str(&format!(
        "G1 X{prime_x1:.3} Y{prime_y_start:.3} Z0.3 F5000\n"
    ));
    g.push_str(&format!(
        "G1 X{prime_x1:.3} Y{prime_y_end:.3} Z0.3 F1500 E15\n"
    ));
    g.push_str(&format!("G1 X{prime_x2:.3} Y{prime_y_end:.3} Z0.3 F5000\n"));
    g.push_str(&format!(
        "G1 X{prime_x2:.3} Y{prime_y_start:.3} Z0.3 F1500 E30\n"
    ));
    g.push_str("G92 E0 ;reset extruder\nG1 Z2.0 F3000\n");
    g.push_str("; === End start G-code ===\n");
    g
}

/// Generate end G-code
pub fn end_gcode(printer: &PrinterProfile) -> String {
    let end_sequence = if printer.default_end_gcode.trim().is_empty() {
        "G91\nG1 E-2 F2700\nG1 E-2 Z0.2 F2400\nG1 X5 Y5 F3000\nG1 Z10\nG90".to_string()
    } else {
        printer.default_end_gcode.trim().to_string()
    };

    let mut g = String::new();
    g.push_str("; === End G-code ===\n");
    g.push_str(&end_sequence);
    if !end_sequence.ends_with('\n') {
        g.push('\n');
    }
    g.push_str("M104 S0 ;hotend off\nM140 S0 ;bed off\n");
    g.push_str("M106 S0 ;fan off\nM84 ;steppers off\n");
    g.push_str("; === End of print ===\n");
    g
}

/// Generate a geometric primitive
pub fn primitive(prim: &Primitive, layer_height: f64, nozzle: f64, speed: f64) -> String {
    let ew = nozzle * 1.2; // extrusion width
    let e_per_mm = (layer_height * ew) / (PI * (1.75_f64 / 2.0).powi(2));
    let f = speed * 60.0;
    let mut g = String::new();
    g.push_str("; Generated primitive\nG90\nM82\nG92 E0\n");

    match prim {
        Primitive::Line { x1, y1, x2, y2 } => {
            let d = ((*x2 - *x1).powi(2) + (*y2 - *y1).powi(2)).sqrt();
            g.push_str(&format!("G1 Z{:.3} F3000\n", layer_height));
            g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", x1, y1));
            g.push_str(&format!(
                "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                x2,
                y2,
                d * e_per_mm,
                f
            ));
        }
        Primitive::Rectangle {
            x,
            y,
            width,
            height,
        } => {
            let layers = 1;
            for layer in 0..layers {
                let z = layer_height * (layer + 1) as f64;
                g.push_str(&format!("\n; Layer {}\nG1 Z{:.3} F3000\n", layer, z));
                g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", x, y));
                let mut e = 0.0_f64;
                let pts = [
                    (x + width, *y),
                    (x + width, y + height),
                    (*x, y + height),
                    (*x, *y),
                ];
                let starts = [
                    (*x, *y),
                    (x + width, *y),
                    (x + width, y + height),
                    (*x, y + height),
                ];
                for i in 0..4 {
                    let d = ((pts[i].0 - starts[i].0).powi(2) + (pts[i].1 - starts[i].1).powi(2))
                        .sqrt();
                    e += d * e_per_mm;
                    g.push_str(&format!(
                        "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                        pts[i].0, pts[i].1, e, f
                    ));
                }
            }
        }
        Primitive::Circle {
            cx,
            cy,
            radius,
            segments,
        } => {
            let segs = *segments.max(&16);
            g.push_str(&format!("G1 Z{:.3} F3000\n", layer_height));
            g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", cx + radius, cy));
            let mut e = 0.0_f64;
            let mut lx = cx + radius;
            let mut ly = *cy;
            for i in 1..=segs {
                let angle = 2.0 * PI * i as f64 / segs as f64;
                let nx = cx + radius * angle.cos();
                let ny = cy + radius * angle.sin();
                let d = ((nx - lx).powi(2) + (ny - ly).powi(2)).sqrt();
                e += d * e_per_mm;
                g.push_str(&format!("G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n", nx, ny, e, f));
                lx = nx;
                ly = ny;
            }
        }
        Primitive::Cylinder {
            cx,
            cy,
            radius,
            height,
            segments,
        } => {
            let segs = *segments.max(&16);
            let num_layers = (height / layer_height).ceil() as usize;
            let mut total_e = 0.0_f64;
            for layer in 0..num_layers {
                let z = layer_height * (layer + 1) as f64;
                g.push_str(&format!("\n; Layer {}\nG1 Z{:.3} F3000\n", layer, z));
                g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", cx + radius, cy));
                let mut lx = cx + radius;
                let mut ly = *cy;
                for i in 1..=segs {
                    let angle = 2.0 * PI * i as f64 / segs as f64;
                    let nx = cx + radius * angle.cos();
                    let ny = cy + radius * angle.sin();
                    let d = ((nx - lx).powi(2) + (ny - ly).powi(2)).sqrt();
                    total_e += d * e_per_mm;
                    g.push_str(&format!(
                        "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                        nx, ny, total_e, f
                    ));
                    lx = nx;
                    ly = ny;
                }
            }
        }
        Primitive::Cube { x, y, size, height } => {
            let num_layers = (height / layer_height).ceil() as usize;
            let mut total_e = 0.0_f64;
            for layer in 0..num_layers {
                let z = layer_height * (layer + 1) as f64;
                g.push_str(&format!("\n; Layer {}\nG1 Z{:.3} F3000\n", layer, z));
                g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", x, y));
                let pts = [
                    (x + size, *y),
                    (x + size, y + size),
                    (*x, y + size),
                    (*x, *y),
                ];
                let sts = [
                    (*x, *y),
                    (x + size, *y),
                    (x + size, y + size),
                    (*x, y + size),
                ];
                for i in 0..4 {
                    let d = ((pts[i].0 - sts[i].0).powi(2) + (pts[i].1 - sts[i].1).powi(2)).sqrt();
                    total_e += d * e_per_mm;
                    g.push_str(&format!(
                        "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                        pts[i].0, pts[i].1, total_e, f
                    ));
                }
            }
        }
        Primitive::SpiralVase {
            cx,
            cy,
            radius,
            height,
            segments_per_layer,
        } => {
            let segs = *segments_per_layer.max(&32);
            let total_segs = ((height / layer_height) * segs as f64).ceil() as usize;
            g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", cx + radius, cy));
            let mut total_e = 0.0_f64;
            let mut lx = cx + radius;
            let mut ly = *cy;
            for i in 1..=total_segs {
                let angle = 2.0 * PI * i as f64 / segs as f64;
                let z = height * i as f64 / total_segs as f64;
                let nx = cx + radius * angle.cos();
                let ny = cy + radius * angle.sin();
                let d = ((nx - lx).powi(2) + (ny - ly).powi(2)).sqrt();
                total_e += d * e_per_mm;
                g.push_str(&format!(
                    "G1 X{:.3} Y{:.3} Z{:.3} E{:.4} F{:.0}\n",
                    nx, ny, z, total_e, f
                ));
                lx = nx;
                ly = ny;
            }
        }
    }
    g
}

/// Generate test print G-code
pub fn test_print(
    tp: &TestPrintType,
    printer: &PrinterProfile,
    material: &MaterialProfile,
) -> String {
    let mut g = String::new();
    g.push_str(&start_gcode(printer, material));

    match tp {
        TestPrintType::TempTower {
            start_temp,
            end_temp,
            step,
        } => {
            g.push_str("\n; === Temperature Tower ===\n");
            let layer_h = 0.2;
            let layers_per_block = 25;
            let block_height = layer_h * layers_per_block as f64;
            let ew = printer.nozzle_diameter * 1.2;
            let e_per_mm = (layer_h * ew) / (PI * 0.875_f64.powi(2));
            let mut temp = *start_temp;
            let dir = if end_temp > start_temp { *step } else { -step };
            let mut total_e = 0.0_f64;
            let mut block = 0;
            while (dir > 0.0 && temp <= *end_temp)
                || (dir < 0.0 && temp >= *end_temp)
                || (dir == 0.0 && block == 0)
            {
                g.push_str(&format!(
                    "\n; Block {} - {}°C\nM104 S{}\n",
                    block, temp, temp
                ));
                for l in 0..layers_per_block {
                    let z = block as f64 * block_height + (l + 1) as f64 * layer_h;
                    g.push_str(&format!("G1 Z{:.3} F3000\n", z));
                    append_rectangle_path(
                        &mut g,
                        (80.0, 80.0, 20.0, 20.0),
                        &mut total_e,
                        e_per_mm,
                        material.print_speed_default * 60.0,
                    );
                }
                temp += dir;
                block += 1;
                if dir == 0.0 {
                    break;
                }
            }
        }
        TestPrintType::RetractionTest {
            start_distance,
            end_distance,
            step,
        } => {
            g.push_str("\n; === Retraction Test ===\n");
            let mut dist = *start_distance;
            let mut col = 0;
            while dist <= *end_distance {
                g.push_str(&format!("\n; Column {} - Retraction: {}mm\n", col, dist));
                let x = 50.0 + col as f64 * 15.0;
                for l in 0..30 {
                    let z = 0.2 * (l + 1) as f64;
                    g.push_str(&format!("G1 Z{:.3} F3000\n", z));
                    g.push_str(&format!("G1 X{:.3} Y80 F5000\n", x));
                    g.push_str(&format!("G1 X{:.3} Y85 E0.3 F1200\n", x));
                    g.push_str(&format!(
                        "G1 E-{:.1} F{:.0} ;retract\n",
                        dist,
                        material.retraction_speed * 60.0
                    ));
                    g.push_str(&format!("G1 X{:.3} Y120 F5000 ;travel\n", x));
                    g.push_str(&format!(
                        "G1 E{:.1} F{:.0} ;unretract\n",
                        dist,
                        material.retraction_speed * 60.0
                    ));
                    g.push_str(&format!("G1 X{:.3} Y125 E0.3 F1200\n", x));
                }
                dist += step;
                col += 1;
            }
        }
        TestPrintType::BedLevel => {
            g.push_str("\n; === Bed Level Test ===\n");
            let margin = 30.0;
            let bx = printer.bed_x - margin * 2.0;
            let by = printer.bed_y - margin * 2.0;
            let sq = 30.0;
            let positions = [
                (margin, margin),
                (margin + bx - sq, margin),
                (margin + bx - sq, margin + by - sq),
                (margin, margin + by - sq),
                (margin + (bx - sq) / 2.0, margin + (by - sq) / 2.0),
            ];
            g.push_str("G1 Z0.2 F3000\n");
            let ew = printer.nozzle_diameter * 1.2;
            let e_per_mm = (0.2 * ew) / (PI * 0.875_f64.powi(2));
            let mut total_e = 0.0_f64;
            for (i, (px, py)) in positions.iter().enumerate() {
                g.push_str(&format!("\n; Square {}\n", i + 1));
                append_rectangle_path(&mut g, (*px, *py, sq, sq), &mut total_e, e_per_mm, 1000.0);
            }
            g.push_str("G92 E0\n");
        }
        TestPrintType::FlowTest => {
            g.push_str("\n; === Flow Rate Test ===\n");
            g.push_str("; Single-wall cubes at different flow rates\n");
            let ew = printer.nozzle_diameter * 1.2;
            let e_per_mm = (0.2 * ew) / (PI * 0.875_f64.powi(2));
            for (i, flow) in [80, 90, 95, 100, 105, 110, 120].iter().enumerate() {
                let x = 30.0 + i as f64 * 25.0;
                let mut total_e = 0.0_f64;
                g.push_str(&format!(
                    "\n; Cube {} - Flow {}%\nM221 S{}\n",
                    i + 1,
                    flow,
                    flow
                ));
                for l in 0..20 {
                    let z = 0.2 * (l + 1) as f64;
                    g.push_str(&format!("G1 Z{:.3} F3000\n", z));
                    append_rectangle_path(
                        &mut g,
                        (x, 80.0, 20.0, 20.0),
                        &mut total_e,
                        e_per_mm,
                        1000.0,
                    );
                }
                g.push_str("G92 E0\n");
            }
            g.push_str("M221 S100 ;reset flow\n");
        }
        TestPrintType::BridgingTest => {
            g.push_str("\n; === Bridging Test ===\n");
            g.push_str("; Pillars with increasing bridge gaps\n");
            let base_y = 80.0;
            for (i, gap) in [5.0, 10.0, 15.0, 20.0, 25.0].iter().enumerate() {
                let x = 40.0 + i as f64 * 30.0;
                g.push_str(&format!("\n; Bridge gap: {}mm\n", gap));
                // Two pillars
                for l in 0..25 {
                    let z = 0.2 * (l + 1) as f64;
                    g.push_str(&format!("G1 Z{:.3} F3000\n", z));
                    for px in &[x, x + gap] {
                        g.push_str(&format!("G1 X{:.1} Y{:.1} F5000\n", px, base_y));
                        g.push_str(&format!("G1 X{:.1} Y{:.1} E0.5 F1000\n", px + 5.0, base_y));
                        g.push_str(&format!(
                            "G1 X{:.1} Y{:.1} E1.0 F1000\n",
                            px + 5.0,
                            base_y + 5.0
                        ));
                        g.push_str(&format!("G1 X{:.1} Y{:.1} E1.5 F1000\n", px, base_y + 5.0));
                        g.push_str(&format!(
                            "G1 X{:.1} Y{:.1} E2.0 F1000\nG92 E0\n",
                            px, base_y
                        ));
                    }
                }
                // Bridge layer
                let bz = 0.2 * 26.0;
                g.push_str(&format!("G1 Z{:.3} F3000\n", bz));
                g.push_str(&format!("G1 X{:.1} Y{:.1} F5000\n", x, base_y));
                g.push_str(&format!(
                    "G1 X{:.1} Y{:.1} E2 F600 ;bridge\nG92 E0\n",
                    x + gap + 5.0,
                    base_y
                ));
            }
        }
        TestPrintType::FirstLayerCalibration => {
            g.push_str("\n; === First Layer Calibration ===\n");
            let cx = printer.bed_x / 2.0;
            let cy = printer.bed_y / 2.0;
            g.push_str("G1 Z0.2 F3000\n");
            let ew = printer.nozzle_diameter * 1.2;
            let e_per_mm = (0.2 * ew) / (PI * 0.875_f64.powi(2));
            let mut total_e = 0.0_f64;
            // Concentric squares
            for i in 0..5 {
                let sz = 20.0 + i as f64 * 15.0;
                let x0 = cx - sz / 2.0;
                let y0 = cy - sz / 2.0;
                append_rectangle_path(&mut g, (x0, y0, sz, sz), &mut total_e, e_per_mm, 600.0);
            }
            g.push_str("G92 E0\n");
        }
    }
    g.push_str(&end_gcode(printer));
    g
}

/// Generate infill pattern
pub fn infill(pattern: &InfillPattern, options: &InfillOptions) -> String {
    let ew = options.nozzle_diameter * 1.2;
    let e_per_mm = (options.layer_height * ew) / (PI * 0.875_f64.powi(2));
    let f = options.speed * 60.0;
    let mut g = String::new();
    let mut total_e = 0.0_f64;
    let x = options.x;
    let y = options.y;
    let w = options.width;
    let h = options.height;
    let spacing = options.spacing;

    match pattern {
        InfillPattern::Lines => {
            let mut cx = x;
            let mut flip = false;
            while cx <= x + w {
                let (sy, ey) = if flip { (y + h, y) } else { (y, y + h) };
                g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", cx, sy));
                let d = h;
                total_e += d * e_per_mm;
                g.push_str(&format!(
                    "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                    cx, ey, total_e, f
                ));
                cx += spacing;
                flip = !flip;
            }
        }
        InfillPattern::Grid => {
            // Horizontal lines
            let mut cy = y;
            let mut flip = false;
            while cy <= y + h {
                let (sx, ex) = if flip { (x + w, x) } else { (x, x + w) };
                g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", sx, cy));
                total_e += w * e_per_mm;
                g.push_str(&format!(
                    "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                    ex, cy, total_e, f
                ));
                cy += spacing;
                flip = !flip;
            }
            // Vertical lines
            let mut cx = x;
            flip = false;
            while cx <= x + w {
                let (sy, ey) = if flip { (y + h, y) } else { (y, y + h) };
                g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", cx, sy));
                total_e += h * e_per_mm;
                g.push_str(&format!(
                    "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                    cx, ey, total_e, f
                ));
                cx += spacing;
                flip = !flip;
            }
        }
        InfillPattern::Triangles => {
            let mut cy = y;
            while cy <= y + h {
                g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", x, cy));
                total_e += w * e_per_mm;
                g.push_str(&format!(
                    "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                    x + w,
                    cy,
                    total_e,
                    f
                ));
                if cy + spacing <= y + h {
                    let d = (spacing * spacing + w * w).sqrt();
                    total_e += d * e_per_mm;
                    g.push_str(&format!(
                        "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                        x,
                        cy + spacing,
                        total_e,
                        f
                    ));
                }
                cy += spacing;
            }
        }
        InfillPattern::Honeycomb => {
            let s = spacing;
            let mut cy = y;
            let mut row = 0;
            while cy <= y + h {
                let offset = if row % 2 == 0 { 0.0 } else { s * 0.75 };
                let mut cx = x + offset;
                while cx <= x + w {
                    // Hexagon
                    let r = s / 2.0;
                    for i in 0..6 {
                        let a1 = PI / 3.0 * i as f64;
                        let a2 = PI / 3.0 * (i + 1) as f64;
                        let x1 = cx + r * a1.cos();
                        let y1 = cy + r * a1.sin();
                        let x2 = cx + r * a2.cos();
                        let y2 = cy + r * a2.sin();
                        if i == 0 {
                            g.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", x1, y1));
                        }
                        let d = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                        total_e += d * e_per_mm;
                        g.push_str(&format!(
                            "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
                            x2, y2, total_e, f
                        ));
                    }
                    cx += s * 1.5;
                }
                cy += s * (3.0_f64).sqrt() / 2.0;
                row += 1;
            }
        }
        InfillPattern::Concentric => {
            let max_offset = w.min(h) / 2.0;
            let mut offset = 0.0;
            while offset < max_offset {
                let rx = x + offset;
                let ry = y + offset;
                let rw = w - offset * 2.0;
                let rh = h - offset * 2.0;
                if rw <= 0.0 || rh <= 0.0 {
                    break;
                }
                append_rectangle_path(&mut g, (rx, ry, rw, rh), &mut total_e, e_per_mm, f);
                offset += spacing;
            }
        }
    }
    g
}

fn append_rectangle_path(
    gcode: &mut String,
    rect: (f64, f64, f64, f64),
    total_e: &mut f64,
    e_per_mm: f64,
    feed_rate: f64,
) {
    let (x, y, width, height) = rect;
    gcode.push_str(&format!("G1 X{:.3} Y{:.3} F5000\n", x, y));
    let mut current_x = x;
    let mut current_y = y;
    for (next_x, next_y) in [
        (x + width, y),
        (x + width, y + height),
        (x, y + height),
        (x, y),
    ] {
        let distance = ((next_x - current_x).powi(2) + (next_y - current_y).powi(2)).sqrt();
        *total_e += distance * e_per_mm;
        gcode.push_str(&format!(
            "G1 X{:.3} Y{:.3} E{:.4} F{:.0}\n",
            next_x, next_y, *total_e, feed_rate
        ));
        current_x = next_x;
        current_y = next_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_gcode_keeps_prime_line_within_bed() {
        let printer = PrinterProfile {
            id: "test".into(),
            name: "Tiny".into(),
            manufacturer: "Test".into(),
            bed_x: 120.0,
            bed_y: 120.0,
            max_z: 120.0,
            nozzle_diameter: 0.4,
            max_print_speed: 60.0,
            max_travel_speed: 120.0,
            max_acceleration: 500.0,
            extruder_type: "direct".into(),
            heated_bed: true,
            auto_bed_leveling: false,
            default_start_gcode: "G28".into(),
            default_end_gcode: "M84".into(),
            description: "test".into(),
        };
        let material = MaterialProfile {
            id: "pla".into(),
            name: "PLA".into(),
            nozzle_temp_min: 190.0,
            nozzle_temp_max: 220.0,
            nozzle_temp_default: 200.0,
            bed_temp_min: 50.0,
            bed_temp_max: 60.0,
            bed_temp_default: 60.0,
            print_speed_min: 20.0,
            print_speed_max: 80.0,
            print_speed_default: 50.0,
            retraction_distance: 0.8,
            retraction_speed: 35.0,
            fan_speed_min: 100,
            fan_speed_max: 100,
            density: 1.24,
            description: "test".into(),
            notes: "test".into(),
        };

        let gcode = start_gcode(&printer, &material);

        assert!(gcode.contains("Y115.000"));
        assert!(!gcode.contains("Y200"));
    }
}
