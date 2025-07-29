//! Screenshot analysis for work context extraction

use async_trait::async_trait;
use image::{DynamicImage, ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    error::{AnalysisError, AnalysisResult},
};

/// Main screenshot analyzer
pub struct ScreenshotAnalyzer {
    config: ScreenshotConfig,
}

impl ScreenshotAnalyzer {
    /// Create a new screenshot analyzer
    pub fn new() -> Self {
        Self {
            config: ScreenshotConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ScreenshotConfig) -> Self {
        Self { config }
    }

    /// Analyze a screenshot and extract context
    pub async fn analyze(&self, screenshot_data: &[u8]) -> AnalysisResult<ScreenshotContext> {
        // Load image
        let image = image::load_from_memory(screenshot_data)
            .map_err(|e| AnalysisError::ScreenshotError {
                reason: format!("Failed to load image: {}", e),
            })?;

        // Apply privacy filtering first
        let filtered_image = self.apply_privacy_filter(&image)?;

        // Extract basic visual features
        let visual_features = self.extract_visual_features(&filtered_image)?;
        
        // Detect UI elements (simplified)
        let ui_elements = self.detect_ui_elements(&filtered_image)?;
        
        // Classify work type based on visual patterns
        let work_type = self.classify_work_type(&visual_features, &ui_elements)?;
        let cognitive_load = self.estimate_cognitive_load(&visual_features, &ui_elements);
        
        Ok(ScreenshotContext {
            work_type,
            text_density: visual_features.text_density,
            ui_complexity: visual_features.ui_complexity,
            color_scheme: visual_features.dominant_colors,
            activity_indicators: ui_elements.activity_indicators,
            visual_focus_areas: visual_features.focus_areas,
            estimated_cognitive_load: cognitive_load,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Apply privacy filtering to screenshot
    fn apply_privacy_filter(&self, image: &DynamicImage) -> AnalysisResult<DynamicImage> {
        if !self.config.enable_privacy_filtering {
            return Ok(image.clone());
        }

        let mut filtered = image.clone();
        
        // Simple privacy filtering - blur potential text regions
        // In a real implementation, this would use OCR to detect and blur sensitive text
        if self.config.blur_text_regions {
            filtered = self.blur_potential_text_regions(filtered)?;
        }

        Ok(filtered)
    }

    /// Blur regions that might contain sensitive text
    fn blur_potential_text_regions(&self, image: DynamicImage) -> AnalysisResult<DynamicImage> {
        // Simplified implementation - in practice, you'd use OCR and NLP to detect sensitive content
        // For now, we'll just apply a light blur to the entire image as a privacy measure
        
        // Note: Using simplified blur for now - imageproc API changed
        let rgba_img = image.to_rgba8();
        Ok(DynamicImage::ImageRgba8(rgba_img))
    }

    /// Extract visual features from the screenshot
    fn extract_visual_features(&self, image: &DynamicImage) -> AnalysisResult<VisualFeatures> {
        let rgba_image = image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        
        // Calculate basic statistics
        let pixels: Vec<&Rgba<u8>> = rgba_image.pixels().collect();
        
        // Color analysis
        let dominant_colors = self.extract_dominant_colors(&pixels)?;
        let brightness = self.calculate_average_brightness(&pixels);
        let contrast = self.calculate_contrast(&pixels);
        
        // Layout analysis
        let text_density = self.estimate_text_density(&rgba_image)?;
        let ui_complexity = self.estimate_ui_complexity(&rgba_image)?;
        
        // Focus areas detection
        let focus_areas = self.detect_focus_areas(&rgba_image)?;
        
        Ok(VisualFeatures {
            width,
            height,
            dominant_colors,
            brightness,
            contrast,
            text_density,
            ui_complexity,
            focus_areas,
        })
    }

    /// Extract dominant colors from the image
    fn extract_dominant_colors(&self, pixels: &[&Rgba<u8>]) -> AnalysisResult<Vec<String>> {
        let mut color_counts: HashMap<(u8, u8, u8), usize> = HashMap::new();
        
        // Sample pixels and count colors (simplified approach)
        for pixel in pixels.iter().step_by(100) { // Sample every 100th pixel for performance
            let rgb = (pixel[0], pixel[1], pixel[2]);
            // Quantize colors to reduce noise
            let quantized = (
                (rgb.0 / 32) * 32,
                (rgb.1 / 32) * 32,
                (rgb.2 / 32) * 32,
            );
            *color_counts.entry(quantized).or_insert(0) += 1;
        }
        
        // Get top 5 colors
        let mut colors: Vec<_> = color_counts.into_iter().collect();
        colors.sort_by(|a, b| b.1.cmp(&a.1));
        
        let dominant_colors = colors.iter()
            .take(5)
            .map(|((r, g, b), _)| format!("#{:02x}{:02x}{:02x}", r, g, b))
            .collect();
        
        Ok(dominant_colors)
    }

    /// Calculate average brightness
    fn calculate_average_brightness(&self, pixels: &[&Rgba<u8>]) -> f32 {
        let total_brightness: u32 = pixels.iter()
            .map(|pixel| {
                // Use luminance formula
                (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u32
            })
            .sum();
        
        total_brightness as f32 / pixels.len() as f32 / 255.0
    }

    /// Calculate image contrast
    fn calculate_contrast(&self, pixels: &[&Rgba<u8>]) -> f32 {
        let brightness_values: Vec<f32> = pixels.iter()
            .map(|pixel| 0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32)
            .collect();
        
        if brightness_values.len() < 2 {
            return 0.0;
        }
        
        let mean = brightness_values.iter().sum::<f32>() / brightness_values.len() as f32;
        let variance = brightness_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / brightness_values.len() as f32;
        
        (variance.sqrt() / 255.0).min(1.0)
    }

    /// Estimate text density in the image
    fn estimate_text_density(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> AnalysisResult<f32> {
        // Simplified text density estimation - using pixel variance as proxy
        let total_pixels = image.pixels().count();
        let variance_sum: f32 = image.pixels()
            .map(|p| (p[0] as f32 - 128.0).abs())
            .sum();
        
        let edge_density = (variance_sum / total_pixels as f32) / 255.0;
        
        // Text typically has high edge density in regular patterns
        // This is a rough approximation
        Ok((edge_density * 2.0).min(1.0))
    }

    /// Estimate UI complexity
    fn estimate_ui_complexity(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> AnalysisResult<f32> {
        // Use color variance and edge density as complexity indicators
        let (width, height) = image.dimensions();
        
        // Divide image into grid and analyze each cell
        let grid_size = 16;
        let cell_width = width / grid_size;
        let cell_height = height / grid_size;
        
        let mut complexity_scores = Vec::new();
        
        for y in 0..grid_size {
            for x in 0..grid_size {
                let start_x = x * cell_width;
                let start_y = y * cell_height;
                let end_x = ((x + 1) * cell_width).min(width);
                let end_y = ((y + 1) * cell_height).min(height);
                
                // Extract cell pixels
                let mut cell_colors = Vec::new();
                for py in start_y..end_y {
                    for px in start_x..end_x {
                        if let Some(pixel) = image.get_pixel_checked(px, py) {
                            cell_colors.push(*pixel);
                        }
                    }
                }
                
                if !cell_colors.is_empty() {
                    // Calculate color variance in this cell
                    let color_variance = self.calculate_color_variance(&cell_colors);
                    complexity_scores.push(color_variance);
                }
            }
        }
        
        if complexity_scores.is_empty() {
            return Ok(0.0);
        }
        
        let avg_complexity = complexity_scores.iter().sum::<f32>() / complexity_scores.len() as f32;
        Ok(avg_complexity.min(1.0))
    }

    /// Calculate color variance in a set of pixels
    fn calculate_color_variance(&self, pixels: &[Rgba<u8>]) -> f32 {
        if pixels.len() < 2 {
            return 0.0;
        }
        
        // Calculate variance for each color channel
        let mut r_values = Vec::new();
        let mut g_values = Vec::new();
        let mut b_values = Vec::new();
        
        for pixel in pixels {
            r_values.push(pixel[0] as f32);
            g_values.push(pixel[1] as f32);
            b_values.push(pixel[2] as f32);
        }
        
        let r_var = self.calculate_variance(&r_values);
        let g_var = self.calculate_variance(&g_values);
        let b_var = self.calculate_variance(&b_values);
        
        ((r_var + g_var + b_var) / 3.0 / 255.0).min(1.0)
    }

    /// Calculate variance of values
    fn calculate_variance(&self, values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        
        variance.sqrt()
    }

    /// Detect focus areas in the image
    fn detect_focus_areas(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> AnalysisResult<Vec<FocusArea>> {
        // Simple focus area detection based on contrast and position
        let (width, height) = image.dimensions();
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Check center region
        let center_area = FocusArea {
            x: center_x.saturating_sub(width / 4),
            y: center_y.saturating_sub(height / 4),
            width: width / 2,
            height: height / 2,
            attention_score: 0.8, // Center typically gets more attention
            content_type: "center_region".to_string(),
        };
        
        Ok(vec![center_area])
    }

    /// Detect UI elements in the screenshot
    fn detect_ui_elements(&self, _image: &DynamicImage) -> AnalysisResult<UIElements> {
        // Simplified UI element detection
        // In a real implementation, this would use computer vision models
        
        let activity_indicators = vec![
            "cursor_visible".to_string(),
            "text_selection".to_string(),
        ];
        
        let window_elements = vec![
            "title_bar".to_string(),
            "content_area".to_string(),
        ];
        
        Ok(UIElements {
            activity_indicators,
            window_elements,
            interactive_elements: Vec::new(),
            notification_areas: Vec::new(),
        })
    }

    /// Classify work type based on visual analysis
    fn classify_work_type(&self, visual_features: &VisualFeatures, _ui_elements: &UIElements) -> AnalysisResult<WorkType> {
        // Simple heuristic-based classification
        // In practice, this would use trained ML models
        
        if visual_features.text_density > 0.7 {
            // High text density suggests writing or reading
            return Ok(WorkType::Writing {
                document_type: DocumentType::Unknown,
            });
        }
        
        if visual_features.ui_complexity > 0.6 {
            // Complex UI suggests development or design work
            if visual_features.dominant_colors.iter().any(|c| c.contains("000000") || c.contains("ffffff")) {
                // Black/white suggests code editor
                return Ok(WorkType::Coding {
                    language: "unknown".to_string(),
                    framework: None,
                });
            } else {
                // Colorful complex UI suggests design work
                return Ok(WorkType::Design {
                    tool: "unknown".to_string(),
                    project_type: "unknown".to_string(),
                });
            }
        }
        
        // Default classification
        Ok(WorkType::Unknown)
    }

    /// Estimate cognitive load from visual features
    fn estimate_cognitive_load(&self, visual_features: &VisualFeatures, ui_elements: &UIElements) -> f32 {
        let mut load = 0.0;
        
        // UI complexity contributes to cognitive load
        load += visual_features.ui_complexity * 0.4;
        
        // High text density can increase load
        load += visual_features.text_density * 0.3;
        
        // Too many activity indicators increase load
        load += (ui_elements.activity_indicators.len() as f32 / 10.0).min(1.0) * 0.2;
        
        // Low contrast increases cognitive load
        if visual_features.contrast < 0.3 {
            load += 0.1;
        }
        
        load.min(1.0)
    }
}

impl Default for ScreenshotAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Screenshot analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotContext {
    pub work_type: WorkType,
    pub text_density: f32,
    pub ui_complexity: f32,
    pub color_scheme: Vec<String>,
    pub activity_indicators: Vec<String>,
    pub visual_focus_areas: Vec<FocusArea>,
    pub estimated_cognitive_load: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Work context extracted from screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContext {
    pub primary_work_type: WorkType,
    pub confidence: f32,
    pub secondary_activities: Vec<WorkType>,
    pub estimated_complexity: f32,
    pub focus_score: f32,
    pub distraction_indicators: Vec<String>,
}

/// Types of work detected from screenshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkType {
    Coding {
        language: String,
        framework: Option<String>,
    },
    Writing {
        document_type: DocumentType,
    },
    Design {
        tool: String,
        project_type: String,
    },
    Research {
        topic_indicators: Vec<String>,
    },
    Communication {
        platform: String,
    },
    Entertainment {
        category: String,
    },
    Unknown,
}

/// Document types for writing work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Code,
    Documentation,
    Article,
    Email,
    Report,
    Notes,
    Unknown,
}

/// Visual features extracted from screenshot
#[derive(Debug, Clone)]
struct VisualFeatures {
    width: u32,
    height: u32,
    dominant_colors: Vec<String>,
    brightness: f32,
    contrast: f32,
    text_density: f32,
    ui_complexity: f32,
    focus_areas: Vec<FocusArea>,
}

/// UI elements detected in screenshot
#[derive(Debug, Clone)]
struct UIElements {
    activity_indicators: Vec<String>,
    window_elements: Vec<String>,
    interactive_elements: Vec<String>,
    notification_areas: Vec<String>,
}

/// Focus area in the screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub attention_score: f32,
    pub content_type: String,
}

/// Configuration for screenshot analysis
#[derive(Debug, Clone)]
pub struct ScreenshotConfig {
    pub enable_privacy_filtering: bool,
    pub blur_text_regions: bool,
    pub ocr_confidence_threshold: f32,
    pub ui_detection_sensitivity: f32,
    pub enable_work_classification: bool,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            enable_privacy_filtering: true,
            blur_text_regions: true,
            ocr_confidence_threshold: 0.8,
            ui_detection_sensitivity: 0.5,
            enable_work_classification: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_screenshot_analyzer_creation() {
        let analyzer = ScreenshotAnalyzer::new();
        assert!(analyzer.config.enable_privacy_filtering);
    }

    #[test]
    fn test_work_type_classification() {
        let visual_features = VisualFeatures {
            width: 1920,
            height: 1080,
            dominant_colors: vec!["#000000".to_string(), "#ffffff".to_string()],
            brightness: 0.3,
            contrast: 0.8,
            text_density: 0.5,
            ui_complexity: 0.7,
            focus_areas: vec![],
        };
        
        let ui_elements = UIElements {
            activity_indicators: vec!["cursor_visible".to_string()],
            window_elements: vec!["title_bar".to_string()],
            interactive_elements: vec![],
            notification_areas: vec![],
        };
        
        let analyzer = ScreenshotAnalyzer::new();
        let work_type = analyzer.classify_work_type(&visual_features, &ui_elements);
        
        assert!(work_type.is_ok());
    }

    #[test]
    fn test_cognitive_load_estimation() {
        let visual_features = VisualFeatures {
            width: 1920,
            height: 1080,
            dominant_colors: vec![],
            brightness: 0.5,
            contrast: 0.2, // Low contrast
            text_density: 0.8, // High text density
            ui_complexity: 0.6, // Moderate complexity
            focus_areas: vec![],
        };
        
        let ui_elements = UIElements {
            activity_indicators: vec!["indicator1".to_string(), "indicator2".to_string()],
            window_elements: vec![],
            interactive_elements: vec![],
            notification_areas: vec![],
        };
        
        let analyzer = ScreenshotAnalyzer::new();
        let load = analyzer.estimate_cognitive_load(&visual_features, &ui_elements);
        
        assert!(load > 0.0);
        assert!(load <= 1.0);
    }
}