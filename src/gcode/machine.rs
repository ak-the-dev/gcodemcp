use super::types::GCodeCommand;

pub(crate) const EPSILON: f64 = 0.001;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MachineState {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub e: f64,
    pub absolute_positioning: bool,
    pub absolute_extrusion: bool,
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            e: 0.0,
            absolute_positioning: true,
            absolute_extrusion: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MotionKind {
    Print { distance: f64, extrusion: f64 },
    Travel { distance: f64 },
    Retract,
    ExtrudeOnly { amount: f64 },
    Stationary,
}

pub(crate) fn is_motion_command(cmd: &GCodeCommand) -> bool {
    cmd.letter == 'G' && (cmd.number == 0 || cmd.number == 1)
}

pub(crate) fn target_position(cmd: &GCodeCommand, state: &MachineState) -> (f64, f64, f64) {
    let x = if state.absolute_positioning {
        cmd.params.get(&'X').copied().unwrap_or(state.x)
    } else {
        state.x + cmd.params.get(&'X').copied().unwrap_or(0.0)
    };
    let y = if state.absolute_positioning {
        cmd.params.get(&'Y').copied().unwrap_or(state.y)
    } else {
        state.y + cmd.params.get(&'Y').copied().unwrap_or(0.0)
    };
    let z = if state.absolute_positioning {
        cmd.params.get(&'Z').copied().unwrap_or(state.z)
    } else {
        state.z + cmd.params.get(&'Z').copied().unwrap_or(0.0)
    };
    (x, y, z)
}

pub(crate) fn extrusion_delta(cmd: &GCodeCommand, state: &MachineState) -> Option<(f64, f64)> {
    let e = cmd.params.get(&'E').copied()?;
    if state.absolute_extrusion {
        Some((e - state.e, e))
    } else {
        Some((e, state.e + e))
    }
}

pub(crate) fn classify_motion(cmd: &GCodeCommand, state: &MachineState) -> MotionKind {
    if !is_motion_command(cmd) {
        return MotionKind::Stationary;
    }

    let (next_x, next_y, next_z) = target_position(cmd, state);
    let distance =
        ((next_x - state.x).powi(2) + (next_y - state.y).powi(2) + (next_z - state.z).powi(2))
            .sqrt();
    let extrusion = extrusion_delta(cmd, state)
        .map(|(delta, _)| delta)
        .unwrap_or(0.0);

    if extrusion > EPSILON && distance > EPSILON {
        MotionKind::Print {
            distance,
            extrusion,
        }
    } else if extrusion < -EPSILON && distance <= EPSILON {
        MotionKind::Retract
    } else if extrusion > EPSILON && distance <= EPSILON {
        MotionKind::ExtrudeOnly { amount: extrusion }
    } else if distance > EPSILON {
        MotionKind::Travel { distance }
    } else {
        MotionKind::Stationary
    }
}

pub(crate) fn apply_command(cmd: &GCodeCommand, state: &mut MachineState) {
    match (cmd.letter, cmd.number) {
        ('G', 90) => state.absolute_positioning = true,
        ('G', 91) => state.absolute_positioning = false,
        ('M', 82) => state.absolute_extrusion = true,
        ('M', 83) => state.absolute_extrusion = false,
        ('G', 92) => {
            if let Some(x) = cmd.params.get(&'X') {
                state.x = *x;
            }
            if let Some(y) = cmd.params.get(&'Y') {
                state.y = *y;
            }
            if let Some(z) = cmd.params.get(&'Z') {
                state.z = *z;
            }
            if let Some(e) = cmd.params.get(&'E') {
                state.e = *e;
            }
        }
        _ if is_motion_command(cmd) => {
            let (next_x, next_y, next_z) = target_position(cmd, state);
            state.x = next_x;
            state.y = next_y;
            state.z = next_z;
            if let Some((_, next_e)) = extrusion_delta(cmd, state) {
                state.e = next_e;
            }
        }
        _ => {}
    }
}
