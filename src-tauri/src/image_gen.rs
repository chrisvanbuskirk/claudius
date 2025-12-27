//! Image generation module using OpenAI DALL-E API.
//!
//! This module handles generating header images for briefing cards using
//! the DALL-E 3 API with landscape format (1792x1024) for optimal header display.
//! Works on all platforms (macOS, Windows, Linux).
#![allow(dead_code)]

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

/// Result of an image generation attempt
#[derive(Debug)]
pub enum ImageGenResult {
    /// Image generated successfully, path to the PNG file
    Success(PathBuf),
    /// Image generation is disabled in settings
    Disabled,
    /// No OpenAI API key configured
    NoApiKey,
    /// Generation failed with an error
    Failed(String),
}

/// DALL-E API request
#[derive(Serialize)]
struct DalleRequest {
    model: String,
    prompt: String,
    n: u32,
    size: String,
    response_format: String,
}

/// DALL-E API response
#[derive(Deserialize)]
struct DalleResponse {
    data: Vec<DalleImage>,
}

/// Individual image in DALL-E response
#[derive(Deserialize)]
struct DalleImage {
    b64_json: String,
}

/// Get the images directory path (~/.claudius/images/)
pub fn get_images_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())?;
    Ok(home.join(".claudius").join("images"))
}

/// Ensure the images directory exists
fn ensure_images_dir() -> Result<PathBuf, String> {
    let images_dir = get_images_dir()?;
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;
    Ok(images_dir)
}

/// Generate image path for a card
pub fn get_image_path(briefing_id: i64, card_index: usize) -> Result<PathBuf, String> {
    Ok(get_images_dir()?.join(format!("{}_{}.png", briefing_id, card_index)))
}

/// Check if an image exists for a card
pub fn image_exists(briefing_id: i64, card_index: usize) -> bool {
    get_image_path(briefing_id, card_index)
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Delete image for a card (used during cleanup)
pub fn delete_image(briefing_id: i64, card_index: usize) -> Result<(), String> {
    let path = get_image_path(briefing_id, card_index)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete image: {}", e))?;
        debug!("Deleted image: {:?}", path);
    }
    Ok(())
}

/// Delete all images for a briefing
pub fn delete_briefing_images(briefing_id: i64) -> Result<usize, String> {
    let images_dir = get_images_dir()?;
    if !images_dir.exists() {
        return Ok(0);
    }

    let prefix = format!("{}_", briefing_id);
    let mut deleted = 0;

    let entries = std::fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read images directory: {}", e))?;

    for entry in entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.starts_with(&prefix) && filename.ends_with(".png") {
                if let Err(e) = std::fs::remove_file(entry.path()) {
                    warn!("Failed to delete image {}: {}", filename, e);
                } else {
                    deleted += 1;
                    debug!("Deleted image: {}", filename);
                }
            }
        }
    }

    if deleted > 0 {
        info!("Deleted {} images for briefing {}", deleted, briefing_id);
    }
    Ok(deleted)
}

/// Save a base64-encoded image to disk
fn save_base64_image(b64: &str, briefing_id: i64, card_index: usize) -> Result<PathBuf, String> {
    let bytes = STANDARD
        .decode(b64)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let path = get_image_path(briefing_id, card_index)?;
    ensure_images_dir()?;

    std::fs::write(&path, bytes).map_err(|e| format!("Failed to write image: {}", e))?;

    Ok(path)
}

/// Generate an image using OpenAI DALL-E API.
///
/// # Arguments
/// * `prompt` - Text description for image generation
/// * `briefing_id` - ID of the briefing (for file naming)
/// * `card_index` - Index of the card within the briefing
/// * `api_key` - OpenAI API key
///
/// # Returns
/// `ImageGenResult` indicating success, failure, or configuration issues.
pub async fn generate_image(
    prompt: &str,
    briefing_id: i64,
    card_index: usize,
    api_key: &str,
) -> ImageGenResult {
    // Ensure images directory exists
    if let Err(e) = ensure_images_dir() {
        return ImageGenResult::Failed(e);
    }

    debug!("Generating image with DALL-E");
    debug!("  Prompt: {}", prompt);
    debug!("  Briefing: {}, Card: {}", briefing_id, card_index);

    let client = reqwest::Client::new();

    let request = DalleRequest {
        model: "dall-e-3".to_string(),
        prompt: prompt.to_string(),
        n: 1,
        size: "1792x1024".to_string(), // Landscape format, ideal for header images
        response_format: "b64_json".to_string(),
    };

    let response = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                error!("DALL-E API error {}: {}", status, text);
                return ImageGenResult::Failed(format!("API error {}: {}", status, text));
            }

            match resp.json::<DalleResponse>().await {
                Ok(dalle_resp) => {
                    if let Some(image) = dalle_resp.data.first() {
                        match save_base64_image(&image.b64_json, briefing_id, card_index) {
                            Ok(path) => {
                                info!("Image generated: {:?}", path);
                                ImageGenResult::Success(path)
                            }
                            Err(e) => {
                                error!("Failed to save image: {}", e);
                                ImageGenResult::Failed(e)
                            }
                        }
                    } else {
                        error!("No image in DALL-E response");
                        ImageGenResult::Failed("No image in response".to_string())
                    }
                }
                Err(e) => {
                    error!("Failed to parse DALL-E response: {}", e);
                    ImageGenResult::Failed(format!("Parse error: {}", e))
                }
            }
        }
        Err(e) => {
            error!("DALL-E API request failed: {}", e);
            ImageGenResult::Failed(format!("Request failed: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_image_path() {
        let path = get_image_path(123, 0).expect("Should get image path");
        assert!(path.to_string_lossy().contains("123_0.png"));
    }

    #[test]
    fn test_get_image_path_multiple() {
        let path0 = get_image_path(456, 0).expect("Should get image path");
        let path1 = get_image_path(456, 1).expect("Should get image path");
        assert_ne!(path0, path1);
        assert!(path0.to_string_lossy().contains("456_0.png"));
        assert!(path1.to_string_lossy().contains("456_1.png"));
    }

    #[test]
    fn test_get_images_dir() {
        let dir = get_images_dir().expect("Should get images dir");
        assert!(dir.to_string_lossy().contains(".claudius"));
        assert!(dir.to_string_lossy().contains("images"));
    }
}
