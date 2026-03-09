use super::types::{GCodeCommand, GCodeFile, GCodeLine};
use std::collections::HashMap;

/// Parse a raw G-code string into structured GCodeFile
pub fn parse(input: &str) -> GCodeFile {
    let lines: Vec<GCodeLine> = input
        .lines()
        .enumerate()
        .map(|(i, raw)| parse_line(i + 1, raw))
        .collect();
    let total_lines = lines.len();
    GCodeFile { lines, total_lines }
}

/// Parse a single G-code line
fn parse_line(line_number: usize, raw: &str) -> GCodeLine {
    let raw = raw.to_string();
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return GCodeLine {
            line_number,
            raw,
            command: None,
            comment: None,
        };
    }

    // Extract comment (everything after ';')
    let (code_part, comment) = if let Some(idx) = trimmed.find(';') {
        let comment = trimmed[idx + 1..].trim().to_string();
        let code = trimmed[..idx].trim();
        (code, Some(comment))
    } else {
        (trimmed, None)
    };

    // Parse command
    let command = if code_part.is_empty() {
        None
    } else {
        parse_command(code_part)
    };

    GCodeLine {
        line_number,
        raw,
        command,
        comment,
    }
}

/// Parse a G-code command string (e.g., "G1 X10.5 Y20.3 E0.5 F1500")
fn parse_command(input: &str) -> Option<GCodeCommand> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Skip line numbers (N1234 prefix)
    let input = if input.starts_with('N') || input.starts_with('n') {
        let rest = input[1..].trim_start();
        // Skip digits
        let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        rest.trim()
    } else {
        input
    };

    if input.is_empty() {
        return None;
    }

    let first_char = input.chars().next()?;
    let letter = first_char.to_ascii_uppercase();

    if !letter.is_ascii_alphabetic() {
        return None;
    }

    // Find the number after the letter
    let rest = &input[1..];
    let num_end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    let num_str = &rest[..num_end];
    let number: u32 = num_str.parse().unwrap_or(0);

    // Parse parameters
    let param_str = &rest[num_end..].trim();
    let params = parse_params(param_str);

    Some(GCodeCommand {
        letter,
        number,
        params,
    })
}

/// Parse parameters like "X10.5 Y20.3 E0.5 F1500"
fn parse_params(input: &str) -> HashMap<char, f64> {
    let mut params = HashMap::new();
    let input = input.trim();
    if input.is_empty() {
        return params;
    }

    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            let key = c.to_ascii_uppercase();
            chars.next(); // consume letter

            // Collect the number
            let mut num_str = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' || nc == '-' || nc == '+' {
                    num_str.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }

            if let Ok(val) = num_str.parse::<f64>() {
                params.insert(key, val);
            }
        } else {
            chars.next(); // skip whitespace/other
        }
    }

    params
}

/// Format a GCodeCommand back into a string
pub fn format_command(cmd: &GCodeCommand) -> String {
    let mut result = format!("{}{}", cmd.letter, cmd.number);
    // Standard parameter order
    let order = ['X', 'Y', 'Z', 'E', 'F', 'S', 'P', 'R', 'I', 'J', 'T'];
    for key in &order {
        if let Some(val) = cmd.params.get(key) {
            if val.fract() == 0.0 && *val < 100000.0 {
                result.push_str(&format!(" {}{}", key, *val as i64));
            } else {
                result.push_str(&format!(" {}{:.4}", key, val));
            }
        }
    }
    // Any remaining params not in the standard order
    for (key, val) in &cmd.params {
        if !order.contains(key) {
            if val.fract() == 0.0 && *val < 100000.0 {
                result.push_str(&format!(" {}{}", key, *val as i64));
            } else {
                result.push_str(&format!(" {}{:.4}", key, val));
            }
        }
    }
    result
}

/// Format an entire GCodeFile back into a string
pub fn format_file(file: &GCodeFile) -> String {
    let mut output = String::new();
    for line in &file.lines {
        if let Some(ref cmd) = line.command {
            output.push_str(&format_command(cmd));
            if let Some(ref comment) = line.comment {
                output.push_str(&format!(" ;{}", comment));
            }
        } else if let Some(ref comment) = line.comment {
            output.push_str(&format!(";{}", comment));
        }
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let gcode = "G28\nG1 X10 Y20 F3000\n;comment\nM104 S200";
        let file = parse(gcode);
        assert_eq!(file.total_lines, 4);
        assert!(file.lines[0].command.is_some());
        let cmd = file.lines[0].command.as_ref().unwrap();
        assert_eq!(cmd.letter, 'G');
        assert_eq!(cmd.number, 28);
    }

    #[test]
    fn test_parse_params() {
        let gcode = "G1 X10.5 Y20.3 E0.05 F1500";
        let file = parse(gcode);
        let cmd = file.lines[0].command.as_ref().unwrap();
        assert_eq!(cmd.letter, 'G');
        assert_eq!(cmd.number, 1);
        assert!((cmd.params[&'X'] - 10.5).abs() < 0.001);
        assert!((cmd.params[&'Y'] - 20.3).abs() < 0.001);
    }

    #[test]
    fn test_parse_comment() {
        let gcode = "G1 X10 ;move to position";
        let file = parse(gcode);
        assert_eq!(file.lines[0].comment, Some("move to position".to_string()));
    }
}
