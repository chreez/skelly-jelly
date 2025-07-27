//! Privacy protection utilities for data capture

use regex::Regex;
use once_cell::sync::Lazy;
use image::{ImageBuffer, Rgb};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use tracing::{warn, debug};

use crate::{config::{PrivacyConfig, PrivacyZone, PrivacyMode}, error::{DataCaptureError, Result}};

pub mod masking;
pub mod filters;

/// PII detection patterns
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

static SSN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{3}-?\d{2}-?\d{4}\b").unwrap()
});

static CREDIT_CARD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap()
});

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b").unwrap()
});

/// Main privacy filter for processing captured data
pub struct PrivacyFilter {
    config: PrivacyConfig,
}

impl PrivacyFilter {
    pub fn new(config: PrivacyConfig) -> Self {
        Self { config }
    }
    
    /// Update the privacy configuration
    pub fn update_config(&mut self, config: PrivacyConfig) {
        self.config = config;
    }
    
    /// Check if an application should be monitored
    pub fn should_monitor_app(&self, app_name: &str) -> bool {
        // Check if app is in ignored list - these apps should NOT be monitored
        if self.config.ignored_app_list.iter().any(|ignored| app_name.contains(ignored)) {
            debug!("App {} is in ignored list", app_name);
            return false;
        }
        
        // Check if app is sensitive - these should still be monitored but with enhanced privacy
        if self.config.sensitive_app_list.iter().any(|sensitive| app_name.contains(sensitive)) {
            warn!("App {} is sensitive, monitoring with enhanced privacy", app_name);
            return true; // Still monitor but with enhanced privacy
        }
        
        true
    }
    
    /// Check if a window title contains sensitive information
    pub fn is_sensitive_window(&self, window_title: &str, app_name: &str) -> bool {
        // Always consider password-related windows sensitive
        let password_indicators = ["password", "login", "sign in", "authentication", "2fa", "mfa"];
        for indicator in &password_indicators {
            if window_title.to_lowercase().contains(indicator) {
                return true;
            }
        }
        
        // Check if app is in sensitive list
        self.config.sensitive_app_list.iter().any(|sensitive| app_name.contains(sensitive))
    }
    
    /// Filter text content for PII
    pub fn filter_text(&self, text: &str) -> String {
        if !self.config.pii_detection {
            return text.to_string();
        }
        
        let mut filtered = text.to_string();
        
        // Mask emails
        if self.config.mask_emails {
            filtered = EMAIL_REGEX.replace_all(&filtered, "[EMAIL]").to_string();
        }
        
        // Mask SSNs
        if self.config.mask_ssn {
            filtered = SSN_REGEX.replace_all(&filtered, "[SSN]").to_string();
        }
        
        // Mask credit cards
        if self.config.mask_credit_cards {
            filtered = CREDIT_CARD_REGEX.replace_all(&filtered, "[CREDIT_CARD]").to_string();
        }
        
        // Mask phone numbers
        filtered = PHONE_REGEX.replace_all(&filtered, "[PHONE]").to_string();
        
        // Additional password field detection
        if self.config.mask_passwords {
            // Look for password-like patterns (sequences of asterisks or dots)
            let password_pattern = Regex::new(r"\*{3,}|•{3,}|\.{3,}").unwrap();
            filtered = password_pattern.replace_all(&filtered, "[PASSWORD]").to_string();
        }
        
        filtered
    }
    
    /// Apply privacy masking to a screenshot
    pub fn mask_screenshot(&self, image_data: &mut Vec<u8>, width: u32, height: u32, 
                          app_name: &str, window_title: &str) -> Result<bool> {
        // Quick check if masking is needed
        if !self.should_apply_screenshot_masking(app_name, window_title) {
            return Ok(false);
        }
        
        // Load image from bytes
        let mut image = self.load_image_from_bytes(image_data, width, height)?;
        
        // Apply privacy zones
        for zone in &self.config.screenshot_privacy_zones {
            self.apply_privacy_zone(&mut image, zone);
        }
        
        // Apply app-specific masking
        if self.is_sensitive_window(window_title, app_name) {
            self.apply_sensitive_app_masking(&mut image, app_name)?;
        }
        
        // Save back to bytes
        *image_data = self.image_to_bytes(&image)?;
        
        Ok(true)
    }
    
    /// Check if screenshot masking should be applied
    fn should_apply_screenshot_masking(&self, app_name: &str, window_title: &str) -> bool {
        // Always mask sensitive apps
        if self.config.sensitive_app_list.iter().any(|sensitive| app_name.contains(sensitive)) {
            return true;
        }
        
        // Mask password-related windows
        if self.is_sensitive_window(window_title, app_name) {
            return true;
        }
        
        // Apply privacy zones if configured
        !self.config.screenshot_privacy_zones.is_empty()
    }
    
    /// Load image from raw bytes
    fn load_image_from_bytes(&self, data: &[u8], width: u32, height: u32) 
                           -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        // For simplicity, assume RGB format
        if data.len() != (width * height * 3) as usize {
            return Err(DataCaptureError::Screenshot(
                format!("Invalid image data size: expected {}, got {}", 
                        width * height * 3, data.len())
            ));
        }
        
        ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or_else(|| DataCaptureError::Screenshot("Failed to create image buffer".into()))
    }
    
    /// Convert image back to bytes
    fn image_to_bytes(&self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Vec<u8>> {
        Ok(image.as_raw().clone())
    }
    
    /// Apply a privacy zone (blur/black out region)
    fn apply_privacy_zone(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, zone: &PrivacyZone) {
        let rect = Rect::at(zone.x as i32, zone.y as i32)
            .of_size(zone.width, zone.height);
        
        // For simplicity, just black out the region
        // In a full implementation, you'd apply gaussian blur
        let black = Rgb([0u8, 0u8, 0u8]);
        draw_filled_rect_mut(image, rect, black);
    }
    
    /// Apply app-specific masking
    fn apply_sensitive_app_masking(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                                  app_name: &str) -> Result<()> {
        match PrivacyMode::Balanced { // TODO: Get from config when field is added
            PrivacyMode::Minimal => {
                // Light masking - just password fields
                self.mask_password_fields(image)?;
            },
            PrivacyMode::Balanced => {
                // Moderate masking - password fields and form inputs
                self.mask_password_fields(image)?;
                self.mask_form_inputs(image)?;
            },
            PrivacyMode::Strict => {
                // Heavy masking - black out most of the screen
                self.apply_strict_masking(image)?;
            },
        }
        Ok(())
    }
    
    /// Mask password fields (simplified implementation)
    fn mask_password_fields(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<()> {
        // In a real implementation, this would use computer vision
        // to detect password fields. For now, just a placeholder.
        debug!("Masking password fields");
        Ok(())
    }
    
    /// Mask form inputs
    fn mask_form_inputs(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<()> {
        debug!("Masking form inputs");
        Ok(())
    }
    
    /// Apply strict privacy masking
    fn apply_strict_masking(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<()> {
        // Black out large portions of the screen
        let (width, height) = image.dimensions();
        let rect = Rect::at(0, 0).of_size(width, height);
        let black = Rgb([0u8, 0u8, 0u8]);
        draw_filled_rect_mut(image, rect, black);
        Ok(())
    }
}

/// Privacy mode detection based on context
pub fn detect_privacy_mode(app_name: &str, window_title: &str) -> PrivacyMode {
    let sensitive_apps = [
        "1Password", "KeePass", "Bitwarden", "LastPass",
        "Banking", "Chase", "Wells Fargo", "Bank of America",
        "Mint", "Personal Capital", "YNAB",
        "TurboTax", "H&R Block",
    ];
    
    for sensitive in &sensitive_apps {
        if app_name.contains(sensitive) {
            return PrivacyMode::Strict;
        }
    }
    
    let password_indicators = ["password", "login", "sign in", "authentication"];
    for indicator in &password_indicators {
        if window_title.to_lowercase().contains(indicator) {
            return PrivacyMode::Balanced;
        }
    }
    
    PrivacyMode::Minimal
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_detection() {
        let mut config = PrivacyConfig::default();
        config.mask_emails = true; // Enable email masking for test
        let filter = PrivacyFilter::new(config);
        let text = "Contact me at john.doe@example.com";
        let filtered = filter.filter_text(text);
        assert!(filtered.contains("[EMAIL]"));
        assert!(!filtered.contains("john.doe@example.com"));
    }
    
    #[test]
    fn test_ssn_detection() {
        let filter = PrivacyFilter::new(PrivacyConfig::default());
        let text = "SSN: 123-45-6789";
        let filtered = filter.filter_text(text);
        assert!(filtered.contains("[SSN]"));
        assert!(!filtered.contains("123-45-6789"));
    }
    
    #[test]
    fn test_credit_card_detection() {
        let filter = PrivacyFilter::new(PrivacyConfig::default());
        let text = "Card: 4532 1234 5678 9012";
        let filtered = filter.filter_text(text);
        assert!(filtered.contains("[CREDIT_CARD]"));
        assert!(!filtered.contains("4532 1234 5678 9012"));
    }
    
    #[test]
    fn test_sensitive_app_detection() {
        let filter = PrivacyFilter::new(PrivacyConfig::default());
        // 1Password is in sensitive_app_list by default, so it should still be monitored but with enhanced privacy
        assert!(filter.should_monitor_app("1Password"));
        assert!(filter.is_sensitive_window("Login - 1Password", "1Password"));
    }
    
    #[test]
    fn test_privacy_mode_detection() {
        assert_eq!(detect_privacy_mode("1Password", ""), PrivacyMode::Strict);
        assert_eq!(detect_privacy_mode("Safari", "Login"), PrivacyMode::Balanced);
        assert_eq!(detect_privacy_mode("TextEdit", "Document"), PrivacyMode::Minimal);
    }
}