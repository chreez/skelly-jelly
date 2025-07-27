//! Advanced privacy masking utilities

use image::{ImageBuffer, Rgb, Rgba};
use imageproc::filter::gaussian_blur_f32;
use imageproc::drawing::{draw_filled_rect_mut, draw_filled_circle_mut};
use imageproc::rect::Rect;
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::{config::{PrivacyZone, PrivacyMode}, error::{Result, DataCaptureError}};

/// Advanced masking operations for screenshots
pub struct AdvancedMasker {
    blur_cache: HashMap<u32, Vec<f32>>, // Cache blur kernels
}

impl AdvancedMasker {
    pub fn new() -> Self {
        Self {
            blur_cache: HashMap::new(),
        }
    }
    
    /// Apply gaussian blur to a region
    pub fn apply_blur(&mut self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                     zone: &PrivacyZone) -> Result<()> {
        let sigma = zone.blur_radius as f32 / 3.0; // Convert radius to sigma
        
        // Extract the region
        let mut region = self.extract_region(image, zone)?;
        
        // Apply gaussian blur
        region = gaussian_blur_f32(&region, sigma);
        
        // Put the blurred region back
        self.replace_region(image, &region, zone)?;
        
        Ok(())
    }
    
    /// Apply pixelation effect
    pub fn apply_pixelation(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                           zone: &PrivacyZone, pixel_size: u32) -> Result<()> {
        let start_x = zone.x as usize;
        let start_y = zone.y as usize;
        let end_x = (zone.x + zone.width).min(image.width()) as usize;
        let end_y = (zone.y + zone.height).min(image.height()) as usize;
        
        // Pixelate by taking average color of each pixel_size x pixel_size block
        for y in (start_y..end_y).step_by(pixel_size as usize) {
            for x in (start_x..end_x).step_by(pixel_size as usize) {
                let avg_color = self.calculate_average_color(
                    image, x, y, 
                    (x + pixel_size as usize).min(end_x),
                    (y + pixel_size as usize).min(end_y)
                );
                
                // Fill the block with average color
                for py in y..(y + pixel_size as usize).min(end_y) {
                    for px in x..(x + pixel_size as usize).min(end_x) {
                        if px < image.width() as usize && py < image.height() as usize {
                            image.put_pixel(px as u32, py as u32, avg_color);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply black box masking
    pub fn apply_blackout(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                         zone: &PrivacyZone) -> Result<()> {
        let rect = Rect::at(zone.x as i32, zone.y as i32)
            .of_size(zone.width, zone.height);
        let black = Rgb([0u8, 0u8, 0u8]);
        draw_filled_rect_mut(image, rect, black);
        Ok(())
    }
    
    /// Apply noise masking
    pub fn apply_noise(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                      zone: &PrivacyZone, intensity: u8) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let start_x = zone.x;
        let start_y = zone.y;
        let end_x = (zone.x + zone.width).min(image.width());
        let end_y = (zone.y + zone.height).min(image.height());
        
        for y in start_y..end_y {
            for x in start_x..end_x {
                let pixel = image.get_pixel_mut(x, y);
                
                // Add random noise to each channel
                pixel.0[0] = pixel.0[0].saturating_add(rng.gen_range(0..intensity));
                pixel.0[1] = pixel.0[1].saturating_add(rng.gen_range(0..intensity));
                pixel.0[2] = pixel.0[2].saturating_add(rng.gen_range(0..intensity));
            }
        }
        
        Ok(())
    }
    
    /// Detect and mask password fields using simple heuristics
    pub fn mask_password_fields(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<()> {
        // Simple heuristic: look for dark rectangular regions that might be password fields
        // In a real implementation, this would use more sophisticated computer vision
        
        let (width, height) = image.dimensions();
        let dark_threshold = 50u8; // Threshold for "dark" pixels
        
        // Scan for rectangular regions with consistent dark color
        for y in 0..height.saturating_sub(20) {
            for x in 0..width.saturating_sub(100) {
                if self.is_potential_password_field(image, x, y, 100, 20, dark_threshold) {
                    debug!("Found potential password field at ({}, {})", x, y);
                    
                    // Mask this region
                    let zone = PrivacyZone {
                        x,
                        y,
                        width: 100,
                        height: 20,
                        blur_radius: 10,
                    };
                    self.apply_blackout(image, &zone)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a region looks like a password field
    fn is_potential_password_field(&self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>, 
                                  x: u32, y: u32, width: u32, height: u32, 
                                  dark_threshold: u8) -> bool {
        let mut dark_pixels = 0;
        let total_pixels = width * height;
        
        for py in y..(y + height).min(image.height()) {
            for px in x..(x + width).min(image.width()) {
                let pixel = image.get_pixel(px, py);
                let brightness = (pixel.0[0] as u32 + pixel.0[1] as u32 + pixel.0[2] as u32) / 3;
                
                if brightness < dark_threshold as u32 {
                    dark_pixels += 1;
                }
            }
        }
        
        // Consider it a password field if >70% of pixels are dark
        dark_pixels > total_pixels * 7 / 10
    }
    
    /// Extract a region from the image
    fn extract_region(&self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>, 
                     zone: &PrivacyZone) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        let mut region = ImageBuffer::new(zone.width, zone.height);
        
        for y in 0..zone.height {
            for x in 0..zone.width {
                let img_x = zone.x + x;
                let img_y = zone.y + y;
                
                if img_x < image.width() && img_y < image.height() {
                    let pixel = image.get_pixel(img_x, img_y);
                    region.put_pixel(x, y, *pixel);
                }
            }
        }
        
        Ok(region)
    }
    
    /// Replace a region in the image
    fn replace_region(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                     region: &ImageBuffer<Rgb<u8>, Vec<u8>>, 
                     zone: &PrivacyZone) -> Result<()> {
        for y in 0..zone.height.min(region.height()) {
            for x in 0..zone.width.min(region.width()) {
                let img_x = zone.x + x;
                let img_y = zone.y + y;
                
                if img_x < image.width() && img_y < image.height() {
                    let pixel = region.get_pixel(x, y);
                    image.put_pixel(img_x, img_y, *pixel);
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculate average color of a region
    fn calculate_average_color(&self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>, 
                              start_x: usize, start_y: usize, 
                              end_x: usize, end_y: usize) -> Rgb<u8> {
        let mut r_sum = 0u32;
        let mut g_sum = 0u32;
        let mut b_sum = 0u32;
        let mut count = 0u32;
        
        for y in start_y..end_y {
            for x in start_x..end_x {
                if x < image.width() as usize && y < image.height() as usize {
                    let pixel = image.get_pixel(x as u32, y as u32);
                    r_sum += pixel.0[0] as u32;
                    g_sum += pixel.0[1] as u32;
                    b_sum += pixel.0[2] as u32;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Rgb([
                (r_sum / count) as u8,
                (g_sum / count) as u8,
                (b_sum / count) as u8,
            ])
        } else {
            Rgb([0, 0, 0])
        }
    }
}

impl Default for AdvancedMasker {
    fn default() -> Self {
        Self::new()
    }
}

/// Masking strategy based on privacy mode
pub fn apply_privacy_mode_masking(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                                 mode: PrivacyMode, 
                                 zones: &[PrivacyZone]) -> Result<()> {
    let mut masker = AdvancedMasker::new();
    
    match mode {
        PrivacyMode::Minimal => {
            // Light masking - just blur sensitive zones
            for zone in zones {
                masker.apply_blur(image, zone)?;
            }
        },
        PrivacyMode::Balanced => {
            // Moderate masking - pixelate zones
            for zone in zones {
                masker.apply_pixelation(image, zone, 8)?;
            }
            
            // Also try to detect and mask password fields
            masker.mask_password_fields(image)?;
        },
        PrivacyMode::Strict => {
            // Heavy masking - black out zones and add noise
            for zone in zones {
                masker.apply_blackout(image, zone)?;
            }
            
            // Add noise to the entire image
            let full_image_zone = PrivacyZone {
                x: 0,
                y: 0,
                width: image.width(),
                height: image.height(),
                blur_radius: 0,
            };
            masker.apply_noise(image, &full_image_zone, 30)?;
        },
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_masker_creation() {
        let masker = AdvancedMasker::new();
        assert!(masker.blur_cache.is_empty());
    }
    
    #[test]
    fn test_blackout_masking() {
        let mut image = ImageBuffer::from_pixel(100, 100, Rgb([255u8, 255u8, 255u8]));
        let masker = AdvancedMasker::new();
        
        let zone = PrivacyZone {
            x: 10,
            y: 10,
            width: 20,
            height: 20,
            blur_radius: 0,
        };
        
        masker.apply_blackout(&mut image, &zone).unwrap();
        
        // Check that the zone is now black
        let pixel = image.get_pixel(15, 15);
        assert_eq!(pixel.0, [0, 0, 0]);
        
        // Check that outside the zone is still white
        let pixel = image.get_pixel(5, 5);
        assert_eq!(pixel.0, [255, 255, 255]);
    }
}