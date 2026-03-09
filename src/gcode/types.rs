use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single parsed G-code line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCodeLine {
    pub line_number: usize,
    pub raw: String,
    pub command: Option<GCodeCommand>,
    pub comment: Option<String>,
}

/// A parsed G-code command (e.g., G1, M104)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCodeCommand {
    pub letter: char,
    pub number: u32,
    pub params: HashMap<char, f64>,
}

/// Full parsed G-code file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCodeFile {
    pub lines: Vec<GCodeLine>,
    pub total_lines: usize,
}

/// Bounding box of the print
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

/// Layer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub layer_number: usize,
    pub z_height: f64,
    pub line_start: usize,
    pub line_end: usize,
    pub extrusion_length: f64,
    pub travel_distance: f64,
    pub print_distance: f64,
    pub num_retractions: usize,
}

/// Speed statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedStats {
    pub min_print_speed: f64,
    pub max_print_speed: f64,
    pub avg_print_speed: f64,
    pub min_travel_speed: f64,
    pub max_travel_speed: f64,
    pub avg_travel_speed: f64,
}

/// Temperature event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureEvent {
    pub line_number: usize,
    pub target_type: String,
    pub temperature: f64,
    pub wait: bool,
}

/// Issue/warning detected in G-code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCodeIssue {
    pub severity: String,
    pub line_number: Option<usize>,
    pub message: String,
    pub suggestion: String,
}

/// Full analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub total_lines: usize,
    pub command_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub total_layers: usize,
    pub estimated_time_seconds: f64,
    pub filament_used_mm: f64,
    pub filament_used_grams: f64,
    pub bounding_box: BoundingBox,
    pub speed_stats: SpeedStats,
    pub total_retractions: usize,
    pub total_travel_distance: f64,
    pub total_print_distance: f64,
    pub temperature_events: Vec<TemperatureEvent>,
    pub layers: Vec<LayerInfo>,
    pub issues: Vec<GCodeIssue>,
}

/// Printer profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterProfile {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub bed_x: f64,
    pub bed_y: f64,
    pub max_z: f64,
    pub nozzle_diameter: f64,
    pub max_print_speed: f64,
    pub max_travel_speed: f64,
    pub max_acceleration: f64,
    pub extruder_type: String,
    pub heated_bed: bool,
    pub auto_bed_leveling: bool,
    pub default_start_gcode: String,
    pub default_end_gcode: String,
    pub description: String,
}

/// Material profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProfile {
    pub id: String,
    pub name: String,
    pub nozzle_temp_min: f64,
    pub nozzle_temp_max: f64,
    pub nozzle_temp_default: f64,
    pub bed_temp_min: f64,
    pub bed_temp_max: f64,
    pub bed_temp_default: f64,
    pub print_speed_min: f64,
    pub print_speed_max: f64,
    pub print_speed_default: f64,
    pub retraction_distance: f64,
    pub retraction_speed: f64,
    pub fan_speed_min: u8,
    pub fan_speed_max: u8,
    pub density: f64,
    pub description: String,
    pub notes: String,
}

/// Optimization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOptions {
    pub optimize_travel: bool,
    pub optimize_retraction: bool,
    pub optimize_speed: bool,
    pub add_z_hop: bool,
    pub z_hop_height: f64,
    pub add_coasting: bool,
    pub coasting_distance: f64,
    pub min_travel_for_retract: f64,
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        Self {
            optimize_travel: true,
            optimize_retraction: true,
            optimize_speed: true,
            add_z_hop: false,
            z_hop_height: 0.4,
            add_coasting: false,
            coasting_distance: 0.3,
            min_travel_for_retract: 1.5,
        }
    }
}

/// Geometric primitive types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Primitive {
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Circle {
        cx: f64,
        cy: f64,
        radius: f64,
        segments: u32,
    },
    Cylinder {
        cx: f64,
        cy: f64,
        radius: f64,
        height: f64,
        segments: u32,
    },
    Cube {
        x: f64,
        y: f64,
        size: f64,
        height: f64,
    },
    SpiralVase {
        cx: f64,
        cy: f64,
        radius: f64,
        height: f64,
        segments_per_layer: u32,
    },
}

/// Test print types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestPrintType {
    TempTower {
        start_temp: f64,
        end_temp: f64,
        step: f64,
    },
    RetractionTest {
        start_distance: f64,
        end_distance: f64,
        step: f64,
    },
    BedLevel,
    FlowTest,
    BridgingTest,
    FirstLayerCalibration,
}

/// Infill pattern types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InfillPattern {
    Lines,
    Grid,
    Triangles,
    Honeycomb,
    Concentric,
}

/// Infill generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfillOptions {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub spacing: f64,
    pub layer_height: f64,
    pub nozzle_diameter: f64,
    pub speed: f64,
}

/// G-code modification operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModificationOp {
    SearchReplace {
        search: String,
        replace: String,
    },
    InsertAtLayer {
        layer: usize,
        gcode: String,
    },
    ScaleCoordinates {
        scale_x: f64,
        scale_y: f64,
        scale_z: f64,
    },
    TranslateCoordinates {
        offset_x: f64,
        offset_y: f64,
        offset_z: f64,
    },
    MirrorAxis {
        axis: char,
    },
    ChangeSpeedAtLayer {
        layer: usize,
        speed_multiplier: f64,
    },
    ChangeTempAtLayer {
        layer: usize,
        temp: f64,
    },
    ChangeFanAtLayer {
        layer: usize,
        fan_speed: u8,
    },
}
