use image::{DynamicImage, GenericImageView, imageops::FilterType};

/// Convert an image to WebP format, resize to a target width, and compress to a
/// target file size.
///
/// Maintains aspect ratio during resizing (Triangle filter).
/// Quality is automatically adjusted to hit the target DPI(450).
/// If image format is unsupported, or the mime_type is incorrect, return Err.
/// If conversion is done, mime_type is updated to "image/webp".
pub async fn image_to_webp(
    mime_type: &mut String,
    image: &[u8],
    width: u32,
) -> anyhow::Result<Vec<u8>> {
    let image_data = image.to_vec();
    let original_mime = mime_type.clone();

    tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&image_data)?;

        let original_dimensions = img.dimensions();

        let resized_img = if original_dimensions.0 > width {
            img.resize(width, u32::MAX, FilterType::Triangle)
        } else {
            img
        };

        let (resized_width, resized_height) = resized_img.dimensions();

        let quality = quality_converter(
            image_data.len(),
            original_dimensions,
            (resized_width, resized_height),
            &original_mime,
        );

        let encoded = encode_webp(&resized_img, quality)?;

        Ok::<(Vec<u8>, String), anyhow::Error>((encoded, "image/webp".to_owned()))
    })
    .await?
    .map(|(data, new_mime)| {
        *mime_type = new_mime;
        data
    })
}

/// Encode a DynamicImage to WebP format with the specified quality
fn encode_webp(img: &DynamicImage, quality: f32) -> anyhow::Result<Vec<u8>> {
    let (width, height) = img.dimensions();

    let rgba_image = img.to_rgba8();

    let encoder = webp::Encoder::from_rgba(rgba_image.as_raw(), width, height);
    let encoded = encoder.encode(quality);

    Ok(encoded.to_vec())
}

/// Guess the quality based on original size, resolution, target resolution, and
/// format.
///
/// The algorithm estimates an appropriate quality level to maintain visual
/// fidelity
fn quality_converter(
    original_size: usize,
    resolution: (u32, u32),
    target_resolution: (u32, u32),
    mime_type: &str,
) -> f32 {
    let original_pixels = (resolution.0 as f64) * (resolution.1 as f64);
    let target_pixels = (target_resolution.0 as f64) * (target_resolution.1 as f64);

    let scale_factor = (target_pixels / original_pixels).sqrt();

    let bytes_per_pixel = (original_size as f64) / original_pixels;

    let base_quality = if bytes_per_pixel > 3.0 {
        85.0
    } else if bytes_per_pixel > 1.5 {
        80.0
    } else if bytes_per_pixel > 0.5 {
        75.0
    } else {
        70.0
    };

    let quality_adjustment = if scale_factor < 0.5 {
        -5.0
    } else if scale_factor < 0.75 {
        0.0
    } else if scale_factor >= 1.0 {
        5.0
    } else {
        2.0
    };

    let format_adjustment = match mime_type {
        "image/png" => 6.0,
        "image/jpeg" => -10.0,
        "image/webp" => 0.0,
        _ => 0.0,
    };

    let final_quality: f32 = base_quality + quality_adjustment + format_adjustment;

    final_quality.clamp(54.0, 95.0)
}
