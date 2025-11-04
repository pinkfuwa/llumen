use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<Model>,
}

#[derive(Debug, Deserialize)]
struct Model {
    id: String,
    architecture: Architecture,
    supported_parameters: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Architecture {
    output_modalities: Vec<String>,
    input_modalities: Option<Vec<String>>,
}

fn write_support_function(file_name: &str, fn_name: &str, description: &str) {
    let mut src = String::new();
    src.push_str("/// Auto‑generated – do NOT edit manually.\n\n");
    src.push_str(&format!("/// {}\n", description));
    src.push_str(&format!("pub fn {}(_id: &str) -> bool {{\n", fn_name));
    src.push_str("    true // Default implementation\n");
    src.push_str("}\n");

    let out_dir = PathBuf::from("./src/models");
    let dest_path = out_dir.join(file_name);
    let file = File::create(&dest_path).expect("Could not create generated file");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(src.as_bytes())
        .expect("Failed to write generated Rust code");
}

fn generate_default_support_functions() {
    write_support_function(
        "support_tool.rs",
        "support_tool",
        "Returns `true` if the model **does** support tools.",
    );
    write_support_function(
        "support_image.rs",
        "support_image",
        "Returns `true` if the model **does** support images.",
    );
    write_support_function(
        "support_audio.rs",
        "support_audio",
        "Returns `true` if the model **does** support audio.",
    );
    println!("cargo:rerun-if-changed=build.rs");
}

fn main() {
    let url = "https://openrouter.ai/api/v1/models";
    let client = Client::new();
    let resp = client
        .get(url)
        .send();
    
    // If we can't fetch the list (e.g., no network), use default implementations
    let resp = match resp {
        Ok(r) => match r.error_for_status() {
            Ok(r) => match r.json::<ApiResponse>() {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Warning: Failed to parse OpenRouter model list: {}", e);
                    eprintln!("Using default implementations");
                    generate_default_support_functions();
                    return;
                }
            },
            Err(e) => {
                eprintln!("Warning: Non-200 response from OpenRouter: {}", e);
                eprintln!("Using default implementations");
                generate_default_support_functions();
                return;
            }
        },
        Err(e) => {
            eprintln!("Warning: Failed to request OpenRouter model list: {}", e);
            eprintln!("Using default implementations");
            generate_default_support_functions();
            return;
        }
    };

    let non_tool_ids: Vec<String> = resp
        .data
        .iter()
        .filter(|m| {
            m.architecture
                .output_modalities
                .iter()
                .any(|mod_| mod_ == "text")
        })
        .filter(|m| {
            !m.supported_parameters
                .iter()
                .any(|p| p.eq_ignore_ascii_case("tool") || p.eq_ignore_ascii_case("tools"))
        })
        .map(|m| m.id.clone())
        .collect();

    let mut src = String::new();
    src.push_str("/// Auto‑generated – do NOT edit manually.\n\n");
    src.push_str("/// Returns `true` if the model **does** support tools.\n");
    src.push_str("pub fn support_tool(id: &str) -> bool {\n");
    src.push_str("    !matches!(id,\n");

    for (i, id) in non_tool_ids.iter().enumerate() {
        let sep = if i == non_tool_ids.len() - 1 {
            ""
        } else {
            " |"
        };
        src.push_str(&format!("        \"{}\"{}\n", id, sep));
    }

    src.push_str("    )\n");
    src.push_str("}\n");

    let out_dir = PathBuf::from("./src/models");
    let dest_path = out_dir.join("support_tool.rs");
    let file = File::create(&dest_path).expect("Could not create generated file");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(src.as_bytes())
        .expect("Failed to write generated Rust code");

    let non_image_ids: Vec<String> = resp
        .data
        .iter()
        .filter(|m| {
            m.architecture
                .output_modalities
                .iter()
                .any(|mod_| mod_ == "text")
        })
        .filter(|m| {
            !m.architecture
                .input_modalities
                .as_ref()
                .map_or(false, |modalities| {
                    modalities.iter().any(|m| m.eq_ignore_ascii_case("image"))
                })
        })
        .map(|m| m.id.clone())
        .collect();

    let mut src = String::new();
    src.push_str("/// Auto‑generated – do NOT edit manually.\n\n");
    src.push_str("/// Returns `true` if the model **does** support images.\n");
    src.push_str("pub fn support_image(id: &str) -> bool {\n");
    src.push_str("    !matches!(id,\n");

    for (i, id) in non_image_ids.iter().enumerate() {
        let sep = if i == non_image_ids.len() - 1 {
            ""
        } else {
            " |"
        };
        src.push_str(&format!("        \"{}\"{}\n", id, sep));
    }

    src.push_str("    )\n");
    src.push_str("}\n");

    let out_dir = PathBuf::from("./src/models");
    let dest_path = out_dir.join("support_image.rs");
    let file = File::create(&dest_path).expect("Could not create generated file");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(src.as_bytes())
        .expect("Failed to write generated Rust code");

    let non_audio_ids: Vec<String> = resp
        .data
        .into_iter()
        .filter(|m| {
            m.architecture
                .output_modalities
                .iter()
                .any(|mod_| mod_ == "text")
        })
        .filter(|m| {
            !m.architecture
                .input_modalities
                .as_ref()
                .map_or(false, |modalities| {
                    modalities.iter().any(|m| m.eq_ignore_ascii_case("audio"))
                })
        })
        .map(|m| m.id)
        .collect();

    let mut src = String::new();
    src.push_str("/// Auto‑generated – do NOT edit manually.\n\n");
    src.push_str("/// Returns `true` if the model **does** support audio.\n");
    src.push_str("pub fn support_audio(id: &str) -> bool {\n");
    src.push_str("    !matches!(id,\n");

    for (i, id) in non_audio_ids.iter().enumerate() {
        let sep = if i == non_audio_ids.len() - 1 {
            ""
        } else {
            " |"
        };
        src.push_str(&format!("        \"{}\"{}\n", id, sep));
    }

    src.push_str("    )\n");
    src.push_str("}\n");

    let out_dir = PathBuf::from("./src/models");
    let dest_path = out_dir.join("support_audio.rs");
    let file = File::create(&dest_path).expect("Could not create generated file");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(src.as_bytes())
        .expect("Failed to write generated Rust code");

    println!("cargo:rerun-if-changed=build.rs");
}
