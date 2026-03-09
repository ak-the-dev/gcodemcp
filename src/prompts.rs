use crate::mcp::server::{McpServer, PromptArgument, PromptContent, PromptDef, PromptMessage};
use std::collections::HashMap;

pub fn register_all(server: &mut McpServer) {
    // Create G-code prompt
    server.add_prompt(
        PromptDef {
            name: "create_gcode".into(),
            description: "Guided prompt for creating G-code. Provides context about printer capabilities and material requirements.".into(),
            arguments: Some(vec![
                PromptArgument { name: "printer_id".into(), description: "Printer ID (e.g. ender3, prusa_mk3s)".into(), required: false },
                PromptArgument { name: "material_id".into(), description: "Material ID (e.g. pla, petg)".into(), required: false },
                PromptArgument { name: "description".into(), description: "What you want to print/generate".into(), required: true },
            ]),
        },
        Box::new(|args: HashMap<String, String>| {
            let printer = args.get("printer_id").map(|s| s.as_str()).unwrap_or("unknown");
            let material = args.get("material_id").map(|s| s.as_str()).unwrap_or("unknown");
            let desc = args.get("description").map(|s| s.as_str()).unwrap_or("a 3D printed part");

            Ok(vec![PromptMessage {
                role: "user".into(),
                content: PromptContent {
                    content_type: "text".into(),
                    text: format!(
r#"I need to create G-code for: {}

Printer: {} (use lookup_printer tool to get specifications)
Material: {} (use lookup_material tool to get temperature/speed settings)

Please:
1. First look up the printer and material profiles using the lookup tools
2. Generate appropriate start G-code using generate_start_gcode
3. Create the main G-code for the described geometry using available generation tools
4. Generate appropriate end G-code using generate_end_gcode
5. Analyze the result with analyze_gcode to verify it looks correct
6. Report the estimated print time and filament usage

Use the suggest_speed_profile tool to determine optimal speeds for this printer/material combination."#,
                        desc, printer, material
                    ),
                },
            }])
        }),
    );

    // Optimize print prompt
    server.add_prompt(
        PromptDef {
            name: "optimize_print".into(),
            description: "Analyze provided G-code and suggest optimizations for quality and speed.".into(),
            arguments: Some(vec![
                PromptArgument { name: "gcode".into(), description: "G-code to optimize".into(), required: true },
                PromptArgument { name: "priority".into(), description: "Optimization priority: 'quality', 'speed', or 'balanced'".into(), required: false },
            ]),
        },
        Box::new(|args: HashMap<String, String>| {
            let priority = args.get("priority").map(|s| s.as_str()).unwrap_or("balanced");
            Ok(vec![PromptMessage {
                role: "user".into(),
                content: PromptContent {
                    content_type: "text".into(),
                    text: format!(
r#"Please analyze and optimize the provided G-code with priority: {}

Steps:
1. Use analyze_gcode to get full statistics
2. Use validate_gcode to check for issues
3. Based on the analysis:
   - If priority is 'quality': focus on retraction optimization, add z-hop, reduce speed on overhangs
   - If priority is 'speed': optimize travel paths, increase speeds where safe, reduce unnecessary retractions
   - If priority is 'balanced': apply all safe optimizations
4. Use optimize_gcode with appropriate settings
5. Compare the original and optimized versions using compare_gcode
6. Report the improvements (time saved, quality improvements, issues fixed)

The G-code to optimize is provided in the conversation."#,
                        priority
                    ),
                },
            }])
        }),
    );

    // Troubleshoot prompt
    server.add_prompt(
        PromptDef {
            name: "troubleshoot_print".into(),
            description: "Diagnose and fix common 3D print quality issues.".into(),
            arguments: Some(vec![
                PromptArgument { name: "issue".into(), description: "Describe the print issue (e.g. 'stringing', 'layer adhesion', 'warping')".into(), required: true },
                PromptArgument { name: "printer_id".into(), description: "Printer being used".into(), required: false },
                PromptArgument { name: "material_id".into(), description: "Material being used".into(), required: false },
            ]),
        },
        Box::new(|args: HashMap<String, String>| {
            let issue = args.get("issue").map(|s| s.as_str()).unwrap_or("unknown issue");
            let printer = args.get("printer_id").map(|s| s.as_str()).unwrap_or("unknown");
            let material = args.get("material_id").map(|s| s.as_str()).unwrap_or("unknown");

            Ok(vec![PromptMessage {
                role: "user".into(),
                content: PromptContent {
                    content_type: "text".into(),
                    text: format!(
r#"I'm experiencing this 3D printing issue: {}
Printer: {}
Material: {}

Please:
1. Read the troubleshooting guide resource (gcode://reference/troubleshooting)
2. Look up the printer and material profiles
3. Provide specific diagnosis and fixes for this issue
4. If G-code is provided, analyze it for potential causes
5. Suggest appropriate calibration test prints (e.g. retraction test for stringing, temp tower for temperature issues)
6. Generate the recommended test print G-code if applicable"#,
                        issue, printer, material
                    ),
                },
            }])
        }),
    );

    // Calibrate prompt
    server.add_prompt(
        PromptDef {
            name: "calibrate_printer".into(),
            description: "Step-by-step calibration guidance for a 3D printer.".into(),
            arguments: Some(vec![
                PromptArgument {
                    name: "printer_id".into(),
                    description: "Printer to calibrate".into(),
                    required: true,
                },
                PromptArgument {
                    name: "material_id".into(),
                    description: "Material to calibrate for".into(),
                    required: true,
                },
            ]),
        },
        Box::new(|args: HashMap<String, String>| {
            let printer = args
                .get("printer_id")
                .map(|s| s.as_str())
                .unwrap_or("generic_cartesian");
            let material = args.get("material_id").map(|s| s.as_str()).unwrap_or("pla");

            Ok(vec![PromptMessage {
                role: "user".into(),
                content: PromptContent {
                    content_type: "text".into(),
                    text: format!(
                        r#"Please guide me through calibrating my printer for optimal results.
Printer: {}
Material: {}

Calibration sequence:
1. First layer calibration - generate first_layer_calibration test
2. Temperature calibration - generate temp_tower test
3. Retraction calibration - generate retraction_test
4. Flow calibration - generate flow_test
5. Speed optimization - use suggest_speed_profile
6. Bridging test - generate bridging_test

For each step:
- Generate the test print G-code
- Explain what to look for
- Explain how to adjust settings based on results
- Provide the optimal values to use going forward"#,
                        printer, material
                    ),
                },
            }])
        }),
    );

    // Explain G-code prompt
    server.add_prompt(
        PromptDef {
            name: "explain_gcode".into(),
            description:
                "Explain a G-code file section by section, breaking down what each part does."
                    .into(),
            arguments: Some(vec![PromptArgument {
                name: "gcode".into(),
                description: "G-code to explain".into(),
                required: true,
            }]),
        },
        Box::new(|_args: HashMap<String, String>| {
            Ok(vec![PromptMessage {
                role: "user".into(),
                content: PromptContent {
                    content_type: "text".into(),
                    text: r#"Please explain the provided G-code in detail:

1. Use analyze_gcode to get an overview
2. Break down the G-code into logical sections (start sequence, printing, end sequence)
3. For each major command, use explain_gcode_command to provide detailed explanations
4. Highlight any potential issues found via validate_gcode
5. Summarize the overall print: what it does, estimated time, filament usage, and any concerns

Present the explanation in a clear, educational format suitable for someone learning G-code."#
                        .into(),
                },
            }])
        }),
    );
}
