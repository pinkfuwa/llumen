pub(crate) fn get_generate_image_tool_def() -> crate::openrouter::Tool {
    crate::openrouter::Tool {
        name: "generate_image".to_string(),
        description: "Generate an image with optional reference files and aspect ratio."
            .to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Image generation prompt"
                },
                "aspect_ratio": {
                    "type": "string",
                    "enum": ["1:1", "2:3", "3:2", "3:4", "4:3", "4:5", "5:4", "9:16", "16:9", "21:9"],
                    "description": "Output aspect ratio"
                },
                "reference_file_names": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Optional uploaded file names used as reference images"
                }
            },
            "required": ["prompt", "aspect_ratio"]
        }),
    }
}
