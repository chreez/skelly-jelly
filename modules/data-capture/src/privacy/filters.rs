//! Content filtering and classification utilities

use regex::Regex;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use tracing::{debug, warn};

use crate::{config::PrivacyConfig, error::Result};

/// Content classification for privacy filtering
#[derive(Debug, Clone, PartialEq)]
pub enum ContentSensitivity {
    Public,      // Safe to capture and store
    Internal,    // Internal content, basic privacy needed
    Confidential,// Confidential content, enhanced privacy
    Restricted,  // Restricted content, maximum privacy or block
}

/// Content filter for text and metadata
pub struct ContentFilter {
    config: PrivacyConfig,
    sensitive_patterns: Vec<Regex>,
    safe_domains: HashSet<String>,
    blocked_domains: HashSet<String>,
}

impl ContentFilter {
    pub fn new(config: PrivacyConfig) -> Self {
        let mut filter = Self {
            config,
            sensitive_patterns: Vec::new(),
            safe_domains: HashSet::new(),
            blocked_domains: HashSet::new(),
        };
        
        filter.initialize_patterns();
        filter.initialize_domain_lists();
        filter
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: PrivacyConfig) {
        self.config = config;
        self.initialize_patterns();
        self.initialize_domain_lists();
    }
    
    /// Classify content sensitivity level
    pub fn classify_content(&self, text: &str, app_name: &str, window_title: &str) -> ContentSensitivity {
        // Check for explicitly blocked/sensitive apps
        if self.is_blocked_app(app_name) {
            return ContentSensitivity::Restricted;
        }
        
        if self.is_sensitive_app(app_name) {
            return ContentSensitivity::Confidential;
        }
        
        // Check for sensitive patterns in text
        if self.contains_sensitive_patterns(text) {
            return ContentSensitivity::Confidential;
        }
        
        // Check window title for sensitive content
        if self.is_sensitive_window_title(window_title) {
            return ContentSensitivity::Confidential;
        }
        
        // Check for financial or medical terms
        if self.contains_financial_terms(text) || self.contains_medical_terms(text) {
            return ContentSensitivity::Confidential;
        }
        
        // Check for work-related content
        if self.contains_work_terms(text) {
            return ContentSensitivity::Internal;
        }
        
        ContentSensitivity::Public
    }
    
    /// Filter window title for privacy
    pub fn filter_window_title(&self, title: &str, app_name: &str) -> String {
        let sensitivity = self.classify_content(title, app_name, title);
        
        match sensitivity {
            ContentSensitivity::Public => title.to_string(),
            ContentSensitivity::Internal => {
                // Redact specific sensitive parts but keep general structure
                self.redact_sensitive_parts(title)
            },
            ContentSensitivity::Confidential => {
                // Replace with generic title
                format!("{} - [CONFIDENTIAL]", app_name)
            },
            ContentSensitivity::Restricted => {
                // Block completely
                "[BLOCKED]".to_string()
            },
        }
    }
    
    /// Check if URL should be blocked
    pub fn should_block_url(&self, url: &str) -> bool {
        // Extract domain from URL
        if let Some(domain) = self.extract_domain(url) {
            if self.blocked_domains.contains(&domain) {
                return true;
            }
            
            // Check for sensitive patterns in URL
            if self.contains_sensitive_url_patterns(url) {
                return true;
            }
        }
        
        false
    }
    
    /// Get privacy recommendations for content
    pub fn get_privacy_recommendations(&self, text: &str, app_name: &str) -> PrivacyRecommendations {
        let sensitivity = self.classify_content(text, app_name, "");
        
        PrivacyRecommendations {
            should_mask_screenshots: matches!(sensitivity, 
                ContentSensitivity::Confidential | ContentSensitivity::Restricted),
            should_redact_text: matches!(sensitivity, 
                ContentSensitivity::Internal | ContentSensitivity::Confidential | ContentSensitivity::Restricted),
            should_block_capture: matches!(sensitivity, ContentSensitivity::Restricted),
            mask_level: match sensitivity {
                ContentSensitivity::Public => 0,
                ContentSensitivity::Internal => 1,
                ContentSensitivity::Confidential => 2,
                ContentSensitivity::Restricted => 3,
            },
            sensitivity,
        }
    }
    
    fn initialize_patterns(&mut self) {
        self.sensitive_patterns.clear();
        
        // Financial patterns
        if let Ok(regex) = Regex::new(r"(?i)(account\s+number|routing\s+number|swift\s+code|iban)") {
            self.sensitive_patterns.push(regex);
        }
        
        // Authentication patterns
        if let Ok(regex) = Regex::new(r"(?i)(api\s+key|access\s+token|private\s+key|secret)") {
            self.sensitive_patterns.push(regex);
        }
        
        // Personal identifiers
        if let Ok(regex) = Regex::new(r"(?i)(driver\s+license|passport|national\s+id)") {
            self.sensitive_patterns.push(regex);
        }
        
        // Medical patterns
        if let Ok(regex) = Regex::new(r"(?i)(medical\s+record|patient\s+id|diagnosis|prescription)") {
            self.sensitive_patterns.push(regex);
        }
    }
    
    fn initialize_domain_lists(&mut self) {
        // Safe domains (public sites)
        self.safe_domains.extend([
            "wikipedia.org".to_string(),
            "github.com".to_string(),
            "stackoverflow.com".to_string(),
            "google.com".to_string(),
        ]);
        
        // Blocked domains (sensitive/adult content)
        self.blocked_domains.extend([
            "example-blocked-site.com".to_string(),
        ]);
    }
    
    fn is_blocked_app(&self, app_name: &str) -> bool {
        // Apps that should never be monitored
        let blocked_apps = ["Keychain Access", "Activity Monitor", "Console"];
        blocked_apps.iter().any(|&blocked| app_name.contains(blocked))
    }
    
    fn is_sensitive_app(&self, app_name: &str) -> bool {
        self.config.sensitive_app_list.iter()
            .any(|sensitive| app_name.contains(sensitive))
    }
    
    fn contains_sensitive_patterns(&self, text: &str) -> bool {
        self.sensitive_patterns.iter()
            .any(|pattern| pattern.is_match(text))
    }
    
    fn is_sensitive_window_title(&self, title: &str) -> bool {
        let sensitive_indicators = [
            "password", "login", "sign in", "authentication",
            "private", "confidential", "restricted",
            "banking", "finance", "medical", "health",
            "incognito", "private browsing"
        ];
        
        let title_lower = title.to_lowercase();
        sensitive_indicators.iter()
            .any(|&indicator| title_lower.contains(indicator))
    }
    
    fn contains_financial_terms(&self, text: &str) -> bool {
        let financial_terms = [
            "bank account", "credit card", "debit card",
            "social security", "tax return", "investment",
            "mortgage", "loan", "insurance"
        ];
        
        let text_lower = text.to_lowercase();
        financial_terms.iter()
            .any(|&term| text_lower.contains(term))
    }
    
    fn contains_medical_terms(&self, text: &str) -> bool {
        let medical_terms = [
            "medical", "health", "patient", "doctor",
            "hospital", "medication", "prescription",
            "diagnosis", "treatment", "insurance"
        ];
        
        let text_lower = text.to_lowercase();
        medical_terms.iter()
            .any(|&term| text_lower.contains(term))
    }
    
    fn contains_work_terms(&self, text: &str) -> bool {
        let work_terms = [
            "confidential", "internal", "proprietary",
            "company", "corporate", "business",
            "client", "customer", "contract"
        ];
        
        let text_lower = text.to_lowercase();
        work_terms.iter()
            .any(|&term| text_lower.contains(term))
    }
    
    fn redact_sensitive_parts(&self, text: &str) -> String {
        let mut redacted = text.to_string();
        
        // Redact numbers that might be sensitive
        let number_regex = Regex::new(r"\b\d{4,}\b").unwrap();
        redacted = number_regex.replace_all(&redacted, "[REDACTED]").to_string();
        
        // Redact email addresses
        let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        redacted = email_regex.replace_all(&redacted, "[EMAIL]").to_string();
        
        redacted
    }
    
    fn extract_domain(&self, url: &str) -> Option<String> {
        // Simple domain extraction
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.host_str().map(|s| s.to_string())
        } else {
            None
        }
    }
    
    fn contains_sensitive_url_patterns(&self, url: &str) -> bool {
        let sensitive_patterns = [
            "/login", "/signin", "/auth", "/password",
            "/admin", "/private", "/secure",
            "/banking", "/finance", "/medical"
        ];
        
        let url_lower = url.to_lowercase();
        sensitive_patterns.iter()
            .any(|&pattern| url_lower.contains(pattern))
    }
}

/// Privacy recommendations for content
#[derive(Debug, Clone)]
pub struct PrivacyRecommendations {
    pub sensitivity: ContentSensitivity,
    pub should_mask_screenshots: bool,
    pub should_redact_text: bool,
    pub should_block_capture: bool,
    pub mask_level: u8, // 0 = none, 3 = maximum
}

/// Domain classification for URL filtering
static DOMAIN_CATEGORIES: Lazy<DomainCategories> = Lazy::new(|| {
    DomainCategories::new()
});

struct DomainCategories {
    social_media: HashSet<String>,
    productivity: HashSet<String>,
    entertainment: HashSet<String>,
    financial: HashSet<String>,
    medical: HashSet<String>,
}

impl DomainCategories {
    fn new() -> Self {
        let mut categories = Self {
            social_media: HashSet::new(),
            productivity: HashSet::new(),
            entertainment: HashSet::new(),
            financial: HashSet::new(),
            medical: HashSet::new(),
        };
        
        // Social media
        categories.social_media.extend([
            "facebook.com", "twitter.com", "linkedin.com",
            "instagram.com", "snapchat.com", "tiktok.com"
        ].iter().map(|s| s.to_string()));
        
        // Productivity
        categories.productivity.extend([
            "gmail.com", "outlook.com", "slack.com",
            "notion.so", "trello.com", "asana.com"
        ].iter().map(|s| s.to_string()));
        
        // Financial
        categories.financial.extend([
            "chase.com", "bankofamerica.com", "wellsfargo.com",
            "paypal.com", "mint.com", "personalcapital.com"
        ].iter().map(|s| s.to_string()));
        
        categories
    }
    
    fn categorize_domain(&self, domain: &str) -> Vec<&'static str> {
        let mut categories = Vec::new();
        
        if self.social_media.contains(domain) {
            categories.push("social_media");
        }
        if self.productivity.contains(domain) {
            categories.push("productivity");
        }
        if self.financial.contains(domain) {
            categories.push("financial");
        }
        
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_classification() {
        let filter = ContentFilter::new(PrivacyConfig::default());
        
        assert_eq!(
            filter.classify_content("Hello world", "TextEdit", "Document"),
            ContentSensitivity::Public
        );
        
        assert_eq!(
            filter.classify_content("API key: abc123", "Terminal", "bash"),
            ContentSensitivity::Confidential
        );
        
        assert_eq!(
            filter.classify_content("Login password", "Safari", "Login"),
            ContentSensitivity::Confidential
        );
    }
    
    #[test]
    fn test_window_title_filtering() {
        let filter = ContentFilter::new(PrivacyConfig::default());
        
        let filtered = filter.filter_window_title("My Bank Account - Chase", "Safari");
        assert!(filtered.contains("[CONFIDENTIAL]") || filtered.contains("Chase"));
        
        let public_title = filter.filter_window_title("Wikipedia - Rust", "Safari");
        assert_eq!(public_title, "Wikipedia - Rust");
    }
    
    #[test]
    fn test_sensitive_patterns() {
        let filter = ContentFilter::new(PrivacyConfig::default());
        
        assert!(filter.contains_sensitive_patterns("My account number is 123456"));
        assert!(filter.contains_sensitive_patterns("API key for the service"));
        assert!(!filter.contains_sensitive_patterns("Just a normal message"));
    }
}