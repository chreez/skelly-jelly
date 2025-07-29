//! Context Detection Module for Work-Type Classification
//! 
//! Analyzes window titles, application usage, and behavioral patterns to detect:
//! - Coding activities (IDEs, programming patterns)
//! - Writing activities (text editors, document patterns)  
//! - Design activities (creative applications, design patterns)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use chrono::{DateTime, Utc};

/// Detected work type with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkType {
    Coding {
        language: Option<String>,
        framework: Option<String>,
        confidence: f32,
    },
    Writing {
        document_type: DocumentType,
        confidence: f32,
    },
    Designing {
        design_type: DesignType,
        confidence: f32,
    },
    Communication {
        platform: String,
        confidence: f32,
    },
    Unknown {
        confidence: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    Technical,  // Documentation, specs
    Creative,   // Blog posts, articles
    Academic,   // Research, papers
    Business,   // Reports, emails
    Personal,   // Notes, journals
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DesignType {
    UI,         // Interface design
    Graphic,    // Visual design
    Web,        // Web design
    Product,    // Product design
    Architecture, // System design
    Unknown,
}

/// Context about the current work environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContext {
    pub work_type: WorkType,
    pub application: String,
    pub window_title: String,
    pub detected_patterns: Vec<DetectedPattern>,
    pub activity_duration: u64, // seconds
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

/// Main work-type detection engine
pub struct WorkTypeDetector {
    coding_detector: CodingDetector,
    writing_detector: WritingDetector,
    design_detector: DesignDetector,
    communication_detector: CommunicationDetector,
    pattern_cache: HashMap<String, (WorkType, DateTime<Utc>)>,
}

impl WorkTypeDetector {
    pub fn new() -> Self {
        Self {
            coding_detector: CodingDetector::new(),
            writing_detector: WritingDetector::new(),
            design_detector: DesignDetector::new(),
            communication_detector: CommunicationDetector::new(),
            pattern_cache: HashMap::new(),
        }
    }

    /// Detect work type from application and window title
    pub fn detect_work_type(
        &mut self,
        application: &str,
        window_title: &str,
        recent_text: Option<&str>,
    ) -> WorkContext {
        let cache_key = format!("{}::{}", application, window_title);
        
        // Check cache first (5 minute cache)
        if let Some((cached_type, cached_time)) = self.pattern_cache.get(&cache_key) {
            if Utc::now().signed_duration_since(*cached_time).num_minutes() < 5 {
                return WorkContext {
                    work_type: cached_type.clone(),
                    application: application.to_string(),
                    window_title: window_title.to_string(),
                    detected_patterns: vec![],
                    activity_duration: 0,
                    last_updated: Utc::now(),
                };
            }
        }

        let mut detected_patterns = Vec::new();
        let mut candidates = Vec::new();

        // Try each detector
        if let Some((work_type, patterns)) = self.coding_detector.detect(application, window_title, recent_text) {
            candidates.push(work_type);
            detected_patterns.extend(patterns);
        }

        if let Some((work_type, patterns)) = self.writing_detector.detect(application, window_title, recent_text) {
            candidates.push(work_type);
            detected_patterns.extend(patterns);
        }

        if let Some((work_type, patterns)) = self.design_detector.detect(application, window_title, recent_text) {
            candidates.push(work_type);
            detected_patterns.extend(patterns);
        }

        if let Some((work_type, patterns)) = self.communication_detector.detect(application, window_title, recent_text) {
            candidates.push(work_type);
            detected_patterns.extend(patterns);
        }

        // Select best candidate based on confidence
        let work_type = candidates.into_iter()
            .max_by(|a, b| self.get_confidence(a).partial_cmp(&self.get_confidence(b)).unwrap())
            .unwrap_or(WorkType::Unknown { confidence: 0.1 });

        // Cache result
        self.pattern_cache.insert(cache_key, (work_type.clone(), Utc::now()));

        WorkContext {
            work_type,
            application: application.to_string(),
            window_title: window_title.to_string(),
            detected_patterns,
            activity_duration: 0,
            last_updated: Utc::now(),
        }
    }

    fn get_confidence(&self, work_type: &WorkType) -> f32 {
        match work_type {
            WorkType::Coding { confidence, .. } => *confidence,
            WorkType::Writing { confidence, .. } => *confidence,
            WorkType::Designing { confidence, .. } => *confidence,
            WorkType::Communication { confidence, .. } => *confidence,
            WorkType::Unknown { confidence } => *confidence,
        }
    }
}

/// Coding activity detection
struct CodingDetector {
    ide_patterns: Vec<Regex>,
    language_patterns: HashMap<String, Vec<Regex>>,
    framework_patterns: HashMap<String, Vec<Regex>>,
}

impl CodingDetector {
    fn new() -> Self {
        let ide_patterns = vec![
            Regex::new(r"(?i)(vscode|visual studio code)").unwrap(),
            Regex::new(r"(?i)(intellij|idea|pycharm|webstorm|phpstorm|clion|rubymine)").unwrap(),
            Regex::new(r"(?i)(sublime|atom|vim|neovim|emacs)").unwrap(),
            Regex::new(r"(?i)(xcode|android studio)").unwrap(),
            Regex::new(r"(?i)(cursor|zed|nova)").unwrap(),
        ];

        let mut language_patterns = HashMap::new();
        language_patterns.insert("rust".to_string(), vec![
            Regex::new(r"\.rs\b").unwrap(),
            Regex::new(r"\bCargo\.toml\b").unwrap(),
            Regex::new(r"\brust\b").unwrap(),
        ]);
        language_patterns.insert("typescript".to_string(), vec![
            Regex::new(r"\.tsx?\b").unwrap(),
            Regex::new(r"\bpackage\.json\b").unwrap(),
            Regex::new(r"\btypescript\b").unwrap(),
        ]);
        language_patterns.insert("python".to_string(), vec![
            Regex::new(r"\.py\b").unwrap(),
            Regex::new(r"\brequirements\.txt\b").unwrap(),
            Regex::new(r"\bpython\b").unwrap(),
        ]);
        language_patterns.insert("javascript".to_string(), vec![
            Regex::new(r"\.jsx?\b").unwrap(),
            Regex::new(r"\bnode_modules\b").unwrap(),
            Regex::new(r"\bjavascript\b").unwrap(),
        ]);

        let mut framework_patterns = HashMap::new();
        framework_patterns.insert("react".to_string(), vec![
            Regex::new(r"\breact\b").unwrap(),
            Regex::new(r"\.jsx\b").unwrap(),
            Regex::new(r"\buseState\b").unwrap(),
        ]);
        framework_patterns.insert("next".to_string(), vec![
            Regex::new(r"\bnext\.js\b").unwrap(),
            Regex::new(r"\bnext\.config\b").unwrap(),
        ]);

        Self {
            ide_patterns,
            language_patterns,
            framework_patterns,
        }
    }

    fn detect(
        &self,
        application: &str,
        window_title: &str,
        recent_text: Option<&str>,
    ) -> Option<(WorkType, Vec<DetectedPattern>)> {
        let text = format!("{} {}", application, window_title);
        let mut confidence = 0.0f32;
        let mut patterns = Vec::new();

        // Check for IDE patterns
        for pattern in &self.ide_patterns {
            if pattern.is_match(&text) {
                confidence += 0.6;
                patterns.push(DetectedPattern {
                    pattern_type: "ide_detected".to_string(),
                    confidence: 0.8,
                    evidence: vec![pattern.to_string()],
                });
                break;
            }
        }

        // Check for file extensions and coding patterns
        let mut detected_language = None;
        for (language, lang_patterns) in &self.language_patterns {
            for pattern in lang_patterns {
                if pattern.is_match(&text) {
                    confidence += 0.4;
                    detected_language = Some(language.clone());
                    patterns.push(DetectedPattern {
                        pattern_type: "language_detected".to_string(),
                        confidence: 0.7,
                        evidence: vec![language.clone()],
                    });
                    break;
                }
            }
        }

        // Check for frameworks
        let mut detected_framework = None;
        for (framework, framework_patterns) in &self.framework_patterns {
            for pattern in framework_patterns {
                if pattern.is_match(&text) {
                    if detected_language.is_some() {
                        confidence += 0.3;
                    } else {
                        confidence += 0.2;
                    }
                    detected_framework = Some(framework.clone());
                    patterns.push(DetectedPattern {
                        pattern_type: "framework_detected".to_string(),
                        confidence: 0.6,
                        evidence: vec![framework.clone()],
                    });
                    break;
                }
            }
        }

        // Check recent text for coding patterns
        if let Some(text) = recent_text {
            if self.contains_code_patterns(text) {
                confidence += 0.3;
                patterns.push(DetectedPattern {
                    pattern_type: "code_content_detected".to_string(),
                    confidence: 0.5,
                    evidence: vec!["code_syntax_found".to_string()],
                });
            }
        }

        if confidence > 0.3 {
            Some((
                WorkType::Coding {
                    language: detected_language,
                    framework: detected_framework,
                    confidence,
                },
                patterns,
            ))
        } else {
            None
        }
    }

    fn contains_code_patterns(&self, text: &str) -> bool {
        let code_indicators = vec![
            Regex::new(r"\bfn\s+\w+\s*\(").unwrap(),  // Rust function
            Regex::new(r"\bfunction\s+\w+\s*\(").unwrap(),  // JS function
            Regex::new(r"\bdef\s+\w+\s*\(").unwrap(),  // Python function
            Regex::new(r"^\s*import\s+").unwrap(),  // Import statements
            Regex::new(r"^\s*from\s+\w+\s+import").unwrap(),  // Python imports
            Regex::new(r"\bconst\s+\w+\s*=").unwrap(),  // JS const
            Regex::new(r"\blet\s+\w+\s*=").unwrap(),  // JS let
            Regex::new(r"^\s*#\w+").unwrap(),  // Preprocessor directives
        ];

        code_indicators.iter().any(|pattern| pattern.is_match(text))
    }
}

/// Writing activity detection
struct WritingDetector {
    writing_apps: Vec<Regex>,
    document_patterns: HashMap<DocumentType, Vec<Regex>>,
}

impl WritingDetector {
    fn new() -> Self {
        let writing_apps = vec![
            Regex::new(r"(?i)(notion|obsidian|roam|logseq)").unwrap(),
            Regex::new(r"(?i)(google docs|microsoft word|pages)").unwrap(),
            Regex::new(r"(?i)(typora|mark text|macdown)").unwrap(),
            Regex::new(r"(?i)(draft|ulysses|scrivener|bear)").unwrap(),
            Regex::new(r"(?i)(textedit|notepad|notes)").unwrap(),
        ];

        let mut document_patterns = HashMap::new();
        document_patterns.insert(DocumentType::Technical, vec![
            Regex::new(r"(?i)(readme|documentation|api|spec|technical)").unwrap(),
            Regex::new(r"(?i)(\.md|markdown)").unwrap(),
        ]);
        document_patterns.insert(DocumentType::Creative, vec![
            Regex::new(r"(?i)(blog|article|story|creative|writing)").unwrap(),
        ]);
        document_patterns.insert(DocumentType::Academic, vec![
            Regex::new(r"(?i)(research|paper|thesis|academic|study)").unwrap(),
        ]);
        document_patterns.insert(DocumentType::Business, vec![
            Regex::new(r"(?i)(report|proposal|meeting|business|email)").unwrap(),
        ]);

        Self {
            writing_apps,
            document_patterns,
        }
    }

    fn detect(
        &self,
        application: &str,
        window_title: &str,
        _recent_text: Option<&str>,
    ) -> Option<(WorkType, Vec<DetectedPattern>)> {
        let text = format!("{} {}", application, window_title);
        let mut confidence = 0.0f32;
        let mut patterns = Vec::new();

        // Check for writing applications
        for pattern in &self.writing_apps {
            if pattern.is_match(&text) {
                confidence += 0.7;
                patterns.push(DetectedPattern {
                    pattern_type: "writing_app_detected".to_string(),
                    confidence: 0.8,
                    evidence: vec![application.to_string()],
                });
                break;
            }
        }

        // Detect document type
        let mut document_type = DocumentType::Unknown;
        for (doc_type, type_patterns) in &self.document_patterns {
            for pattern in type_patterns {
                if pattern.is_match(&text) {
                    confidence += 0.3;
                    document_type = doc_type.clone();
                    patterns.push(DetectedPattern {
                        pattern_type: "document_type_detected".to_string(),
                        confidence: 0.6,
                        evidence: vec![format!("{:?}", doc_type)],
                    });
                    break;
                }
            }
        }

        if confidence > 0.4 {
            Some((
                WorkType::Writing {
                    document_type,
                    confidence,
                },
                patterns,
            ))
        } else {
            None
        }
    }
}

/// Design activity detection
struct DesignDetector {
    design_apps: Vec<Regex>,
    design_patterns: HashMap<DesignType, Vec<Regex>>,
}

impl DesignDetector {
    fn new() -> Self {
        let design_apps = vec![
            Regex::new(r"(?i)(figma|sketch|adobe|photoshop|illustrator)").unwrap(),
            Regex::new(r"(?i)(canva|framer|principle|invision)").unwrap(),
            Regex::new(r"(?i)(blender|cinema 4d|maya|3ds max)").unwrap(),
        ];

        let mut design_patterns = HashMap::new();
        design_patterns.insert(DesignType::UI, vec![
            Regex::new(r"(?i)(ui|interface|wireframe|mockup)").unwrap(),
        ]);
        design_patterns.insert(DesignType::Graphic, vec![
            Regex::new(r"(?i)(logo|brand|graphic|poster|flyer)").unwrap(),
        ]);
        design_patterns.insert(DesignType::Web, vec![
            Regex::new(r"(?i)(web|website|landing|homepage)").unwrap(),
        ]);

        Self {
            design_apps,
            design_patterns,
        }
    }

    fn detect(
        &self,
        application: &str,
        window_title: &str,
        _recent_text: Option<&str>,
    ) -> Option<(WorkType, Vec<DetectedPattern>)> {
        let text = format!("{} {}", application, window_title);
        let mut confidence = 0.0f32;
        let mut patterns = Vec::new();

        // Check for design applications
        for pattern in &self.design_apps {
            if pattern.is_match(&text) {
                confidence += 0.8;
                patterns.push(DetectedPattern {
                    pattern_type: "design_app_detected".to_string(),
                    confidence: 0.9,
                    evidence: vec![application.to_string()],
                });
                break;
            }
        }

        // Detect design type
        let mut design_type = DesignType::Unknown;
        for (dtype, type_patterns) in &self.design_patterns {
            for pattern in type_patterns {
                if pattern.is_match(&text) {
                    confidence += 0.2;
                    design_type = dtype.clone();
                    patterns.push(DetectedPattern {
                        pattern_type: "design_type_detected".to_string(),
                        confidence: 0.5,
                        evidence: vec![format!("{:?}", dtype)],
                    });
                    break;
                }
            }
        }

        if confidence > 0.5 {
            Some((
                WorkType::Designing {
                    design_type,
                    confidence,
                },
                patterns,
            ))
        } else {
            None
        }
    }
}

/// Communication activity detection  
struct CommunicationDetector {
    communication_apps: Vec<Regex>,
}

impl CommunicationDetector {
    fn new() -> Self {
        let communication_apps = vec![
            Regex::new(r"(?i)(slack|discord|teams|zoom|meet)").unwrap(),
            Regex::new(r"(?i)(gmail|outlook|mail|thunderbird)").unwrap(),
            Regex::new(r"(?i)(whatsapp|telegram|signal|messenger)").unwrap(),
        ];

        Self {
            communication_apps,
        }
    }

    fn detect(
        &self,
        application: &str,
        window_title: &str,
        _recent_text: Option<&str>,
    ) -> Option<(WorkType, Vec<DetectedPattern>)> {
        let text = format!("{} {}", application, window_title);
        let mut confidence = 0.0f32;
        let mut patterns = Vec::new();

        // Check for communication applications
        for pattern in &self.communication_apps {
            if pattern.is_match(&text) {
                confidence = 0.9;
                patterns.push(DetectedPattern {
                    pattern_type: "communication_app_detected".to_string(),
                    confidence: 0.9,
                    evidence: vec![application.to_string()],
                });
                break;
            }
        }

        if confidence > 0.5 {
            Some((
                WorkType::Communication {
                    platform: application.to_string(),
                    confidence,
                },
                patterns,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coding_detection() {
        let mut detector = WorkTypeDetector::new();
        
        let context = detector.detect_work_type(
            "Visual Studio Code",
            "main.rs - skelly-jelly",
            Some("fn main() {\n    println!(\"Hello\");\n}")
        );
        
        match context.work_type {
            WorkType::Coding { language, confidence, .. } => {
                assert!(confidence > 0.8);
                assert_eq!(language, Some("rust".to_string()));
            },
            _ => panic!("Should detect coding activity"),
        }
    }

    #[test]
    fn test_writing_detection() {
        let mut detector = WorkTypeDetector::new();
        
        let context = detector.detect_work_type(
            "Notion",
            "Documentation - API Guide",
            None
        );
        
        match context.work_type {
            WorkType::Writing { document_type, confidence } => {
                assert!(confidence > 0.6);
                assert_eq!(document_type, DocumentType::Technical);
            },
            _ => panic!("Should detect writing activity"),
        }
    }

    #[test]
    fn test_design_detection() {
        let mut detector = WorkTypeDetector::new();
        
        let context = detector.detect_work_type(
            "Figma",
            "UI Design - Dashboard",
            None
        );
        
        match context.work_type {
            WorkType::Designing { design_type, confidence } => {
                assert!(confidence > 0.8);
                assert_eq!(design_type, DesignType::UI);
            },
            _ => panic!("Should detect design activity"),
        }
    }

    #[test]
    fn test_unknown_activity() {
        let mut detector = WorkTypeDetector::new();
        
        let context = detector.detect_work_type(
            "Calculator",
            "Calculator",
            None
        );
        
        match context.work_type {
            WorkType::Unknown { confidence } => {
                assert!(confidence < 0.5);
            },
            _ => panic!("Should detect unknown activity"),
        }
    }
}