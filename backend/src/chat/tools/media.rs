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
                "generating_file": {
                    "type": "string",
                    "description": "Filename to assign to the generated image"
                },
                "reference_files": {
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

pub(crate) fn get_generate_video_tool_def() -> crate::openrouter::Tool {
    crate::openrouter::Tool {
        name: "generate_video".to_string(),
        description: "Generate a video with optional references and controls.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Video generation prompt"
                },
                "generating_file": {
                    "type": "string",
                    "description": "Filename to assign to the generated video"
                },
                "duration": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional video duration in seconds"
                },
                "resolution": {
                    "type": "string",
                    "description": "Optional output resolution"
                },
                "aspect_ratio": {
                    "type": "string",
                    "description": "Optional output aspect ratio"
                },
                "size": {
                    "type": "string",
                    "description": "Optional provider-defined output size"
                },
                "generate_audio": {
                    "type": "boolean",
                    "description": "Whether the model should generate audio when supported"
                },
                "reference_files": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Optional uploaded file names used as image/video references"
                }
            },
            "required": ["prompt"]
        }),
    }
}
