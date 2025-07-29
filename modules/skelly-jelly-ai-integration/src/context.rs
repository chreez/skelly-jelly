//! Context processing and analysis
//!
//! Analyzes work context, behavioral patterns, and user state to build
//! relevant context for AI generation.

use crate::error::{AIIntegrationError, Result};
use crate::types::{
    WorkContext, WorkType, BehavioralMetrics, ADHDState, LLMContext, 
    UserPreferences, TaskCategory, UrgencyLevel
};
use std::collections::HashMap;

/// Processes and analyzes context for AI generation
pub struct ContextProcessor {
    work_analyzer: WorkContextAnalyzer,
    behavioral_builder: BehavioralContextBuilder,
    context_compressor: ContextCompressor,
    privacy_filter: PrivacyFilter,
}

impl ContextProcessor {
    pub fn new() -> Self {
        Self {
            work_analyzer: WorkContextAnalyzer::new(),
            behavioral_builder: BehavioralContextBuilder::new(),
            context_compressor: ContextCompressor::new(),
            privacy_filter: PrivacyFilter::new(),
        }
    }

    /// Build comprehensive context for LLM generation
    pub async fn build_context(
        &self,
        intervention_type: &str,
        current_state: &ADHDState,
        state_history: &[ADHDState],
        metrics: &BehavioralMetrics,
        work_context: &WorkContext,
        user_preferences: &UserPreferences,
    ) -> Result<LLMContext> {
        // Build behavioral summary
        let behavioral_summary = self.behavioral_builder.summarize(
            state_history,
            metrics,
            current_state,
        )?;

        // Analyze work context
        let work_analysis = self.work_analyzer.analyze(work_context)?;

        // Apply privacy filtering
        let filtered_context = self.privacy_filter.filter(&work_analysis)?;

        // Determine max tokens based on complexity and user preferences
        let max_tokens = self.calculate_token_budget(intervention_type, &work_analysis);

        // Convert work analysis to string for compression
        let work_context_text = format!("{}: {}", filtered_context.task_description, filtered_context.relevant_context);

        // Compress to fit token budget
        let compressed = self.context_compressor.compress(
            &behavioral_summary,
            &work_context_text,
            max_tokens,
        )?;

        Ok(LLMContext {
            system_prompt: self.build_system_prompt(user_preferences),
            behavioral_context: compressed.behavioral,
            work_context: compressed.work,
            intervention_type: intervention_type.to_string(),
            user_preferences: user_preferences.clone(),
            max_tokens,
        })
    }

    fn build_system_prompt(&self, preferences: &UserPreferences) -> String {
        let base_prompt = r#"You are Skelly, a melty skeleton companion helping someone with ADHD stay focused.

Your personality:
- Chill and supportive, never patronizing or pushy
- Use casual language, occasional skeleton puns (very sparingly)
- Celebrate small wins, acknowledge struggles without judgment
- Brief messages (1-2 sentences usually)
- Focus on actionable, specific help that's easy to implement right now

Response guidelines:
- Be encouraging without being fake or over-the-top
- Offer specific, practical suggestions
- Respect flow states and current energy levels
- Use gentle language during transitions
- Keep it real and authentic"#;

        // Customize based on user preferences
        let mut prompt = base_prompt.to_string();

        match preferences.message_style {
            crate::types::MessageStyle::Minimal => {
                prompt.push_str("\n- Keep responses very brief and direct");
            }
            crate::types::MessageStyle::Humorous => {
                prompt.push_str("\n- Add a bit more humor and lightness when appropriate");
            }
            crate::types::MessageStyle::Informative => {
                prompt.push_str("\n- Include helpful context and explanations");
            }
            _ => {}
        }

        prompt
    }

    fn calculate_token_budget(&self, intervention_type: &str, analysis: &WorkAnalysis) -> usize {
        let base_budget = match intervention_type {
            "encouragement" => 200,
            "suggestion" => 300,
            "celebration" => 150,
            "gentle_nudge" => 250,
            _ => 200,
        };

        // Adjust based on work complexity
        let complexity_multiplier = match analysis.complexity_level {
            ComplexityLevel::Simple => 0.8,
            ComplexityLevel::Moderate => 1.0,
            ComplexityLevel::Complex => 1.3,
        };

        (base_budget as f32 * complexity_multiplier) as usize
    }
}

/// Analyzes work context to understand current activity
pub struct WorkContextAnalyzer {
    language_detector: LanguageDetector,
    task_classifier: TaskClassifier,
    complexity_analyzer: ComplexityAnalyzer,
}

impl WorkContextAnalyzer {
    pub fn new() -> Self {
        Self {
            language_detector: LanguageDetector::new(),
            task_classifier: TaskClassifier::new(),
            complexity_analyzer: ComplexityAnalyzer::new(),
        }
    }

    pub fn analyze(&self, context: &WorkContext) -> Result<WorkAnalysis> {
        let analysis = match &context.work_type {
            WorkType::Coding { language, framework } => {
                self.analyze_coding_context(language, framework, context)
            }
            WorkType::Writing { document_type } => {
                self.analyze_writing_context(document_type, context)
            }
            WorkType::Design { tool, project_type } => {
                self.analyze_design_context(tool, project_type, context)
            }
            WorkType::Research { topic } => {
                self.analyze_research_context(topic, context)
            }
            WorkType::Communication { platform } => {
                self.analyze_communication_context(platform, context)
            }
            WorkType::Unknown => Ok(WorkAnalysis::default()),
        }?;

        Ok(analysis)
    }

    fn analyze_coding_context(
        &self,
        language: &str,
        framework: &Option<String>,
        context: &WorkContext,
    ) -> Result<WorkAnalysis> {
        let task = self.task_classifier.classify_coding_task(
            &context.window_title,
            context.screenshot_text.as_deref(),
        )?;

        let complexity = self.complexity_analyzer.analyze_code_complexity(
            language,
            context.screenshot_text.as_deref(),
        )?;

        let stuck_indicators = self.detect_stuck_patterns(
            &context.window_title,
            context.screenshot_text.as_deref(),
        )?;

        Ok(WorkAnalysis {
            task_description: task,
            complexity_level: complexity,
            potential_issues: stuck_indicators,
            relevant_context: self.extract_relevant_code_context(
                context.screenshot_text.as_deref()
            )?,
            domain_suggestions: self.get_coding_suggestions(language, framework),
        })
    }

    fn analyze_writing_context(
        &self,
        document_type: &str,
        context: &WorkContext,
    ) -> Result<WorkAnalysis> {
        let task = format!("Writing {} document", document_type);
        let complexity = self.complexity_analyzer.analyze_writing_complexity(
            context.screenshot_text.as_deref(),
        )?;

        Ok(WorkAnalysis {
            task_description: task,
            complexity_level: complexity,
            potential_issues: Vec::new(),
            relevant_context: context.screenshot_text.clone().unwrap_or_default(),
            domain_suggestions: self.get_writing_suggestions(document_type),
        })
    }

    fn analyze_design_context(
        &self,
        tool: &str,
        project_type: &str,
        context: &WorkContext,
    ) -> Result<WorkAnalysis> {
        let task = format!("Design work in {} for {}", tool, project_type);
        
        Ok(WorkAnalysis {
            task_description: task,
            complexity_level: ComplexityLevel::Moderate,
            potential_issues: Vec::new(),
            relevant_context: format!("Design tool: {}, Project: {}", tool, project_type),
            domain_suggestions: self.get_design_suggestions(tool),
        })
    }

    fn analyze_research_context(
        &self,
        topic: &str,
        context: &WorkContext,
    ) -> Result<WorkAnalysis> {
        let task = format!("Researching {}", topic);
        
        Ok(WorkAnalysis {
            task_description: task,
            complexity_level: ComplexityLevel::Moderate,
            potential_issues: Vec::new(),
            relevant_context: format!("Research topic: {}", topic),
            domain_suggestions: self.get_research_suggestions(),
        })
    }

    fn analyze_communication_context(
        &self,
        platform: &str,
        context: &WorkContext,
    ) -> Result<WorkAnalysis> {
        let task = format!("Communication on {}", platform);
        
        Ok(WorkAnalysis {
            task_description: task,
            complexity_level: ComplexityLevel::Simple,
            potential_issues: Vec::new(),
            relevant_context: format!("Platform: {}", platform),
            domain_suggestions: self.get_communication_suggestions(platform),
        })
    }

    fn detect_stuck_patterns(
        &self,
        window_title: &str,
        screen_text: Option<&str>,
    ) -> Result<Vec<String>> {
        let mut indicators = Vec::new();

        // Check window title for stuck indicators
        let stuck_keywords = vec![
            "error", "exception", "failed", "not found", "undefined",
            "syntax error", "compile error", "debug", "troubleshoot"
        ];

        for keyword in stuck_keywords {
            if window_title.to_lowercase().contains(keyword) {
                indicators.push(format!("Possible {} issue detected", keyword));
            }
        }

        // Check screen text if available
        if let Some(text) = screen_text {
            if text.contains("404") || text.contains("Not Found") {
                indicators.push("Resource not found".to_string());
            }
            if text.contains("Exception") || text.contains("Error:") {
                indicators.push("Exception or error in code".to_string());
            }
        }

        Ok(indicators)
    }

    fn extract_relevant_code_context(&self, screen_text: Option<&str>) -> Result<String> {
        if let Some(text) = screen_text {
            // Extract key patterns like function names, class names, etc.
            let lines: Vec<&str> = text.lines().take(10).collect();
            Ok(lines.join(" "))
        } else {
            Ok("No code context available".to_string())
        }
    }

    fn get_coding_suggestions(&self, language: &str, framework: &Option<String>) -> Vec<String> {
        let mut suggestions = vec![
            "Check documentation".to_string(),
            "Try rubber duck debugging".to_string(),
            "Take a step back and review approach".to_string(),
        ];

        // Language-specific suggestions
        match language.to_lowercase().as_str() {
            "rust" => suggestions.push("Check the Rust Book or compiler errors".to_string()),
            "javascript" | "typescript" => suggestions.push("Check browser console for errors".to_string()),
            "python" => suggestions.push("Try running with verbose error output".to_string()),
            _ => {}
        }

        suggestions
    }

    fn get_writing_suggestions(&self, document_type: &str) -> Vec<String> {
        vec![
            "Break into smaller sections".to_string(),
            "Start with an outline".to_string(),
            "Write a rough first draft".to_string(),
            format!("Research {} best practices", document_type),
        ]
    }

    fn get_design_suggestions(&self, tool: &str) -> Vec<String> {
        vec![
            "Look for design inspiration".to_string(),
            "Start with rough sketches".to_string(),
            "Focus on user needs first".to_string(),
            format!("Check {} tutorials or communities", tool),
        ]
    }

    fn get_research_suggestions(&self) -> Vec<String> {
        vec![
            "Start with reliable sources".to_string(),
            "Take notes as you go".to_string(),
            "Set specific research questions".to_string(),
            "Use multiple source types".to_string(),
        ]
    }

    fn get_communication_suggestions(&self, platform: &str) -> Vec<String> {
        vec![
            "Keep messages clear and concise".to_string(),
            "Consider your audience".to_string(),
            "Proofread before sending".to_string(),
        ]
    }
}

/// Analysis result from work context
#[derive(Debug, Clone)]
pub struct WorkAnalysis {
    pub task_description: String,
    pub complexity_level: ComplexityLevel,
    pub potential_issues: Vec<String>,
    pub relevant_context: String,
    pub domain_suggestions: Vec<String>,
}

impl Default for WorkAnalysis {
    fn default() -> Self {
        Self {
            task_description: "General work".to_string(),
            complexity_level: ComplexityLevel::Simple,
            potential_issues: Vec::new(),
            relevant_context: "No specific context available".to_string(),
            domain_suggestions: vec!["Take regular breaks".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
}

/// Builds behavioral context summaries
pub struct BehavioralContextBuilder;

impl BehavioralContextBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn summarize(
        &self,
        state_history: &[ADHDState],
        metrics: &BehavioralMetrics,
        current_state: &ADHDState,
    ) -> Result<String> {
        let mut summary = Vec::new();

        // Current state summary
        summary.push(format!(
            "Current state: {:?} (confidence: {:.1})",
            current_state.state_type, current_state.confidence
        ));

        // Productivity summary
        summary.push(format!(
            "Productivity: {:.0}% focused time, {} sessions today",
            metrics.productive_time_ratio * 100.0,
            metrics.focus_session_count
        ));

        // Recent patterns
        if state_history.len() > 1 {
            let recent_states: Vec<String> = state_history
                .iter()
                .rev()
                .take(3)
                .map(|s| format!("{:?}", s.state_type))
                .collect();
            summary.push(format!("Recent pattern: {}", recent_states.join(" → ")));
        }

        // Recovery context
        if metrics.recovery_time > 0 {
            summary.push(format!(
                "Average recovery time: {}s",
                metrics.recovery_time / 1000
            ));
        }

        Ok(summary.join(". "))
    }
}

/// Compresses context to fit token budgets
pub struct ContextCompressor;

impl ContextCompressor {
    pub fn new() -> Self {
        Self
    }

    pub fn compress(
        &self,
        behavioral: &str,
        work: &str,
        max_tokens: usize,
    ) -> Result<CompressedContext> {
        // Estimate tokens (rough approximation: 1 token ≈ 4 characters)
        let estimate_tokens = |text: &str| text.len() / 4;

        let behavioral_tokens = estimate_tokens(behavioral);
        let work_tokens = estimate_tokens(work);
        let total_tokens = behavioral_tokens + work_tokens;

        if total_tokens <= max_tokens {
            return Ok(CompressedContext {
                behavioral: behavioral.to_string(),
                work: work.to_string(),
            });
        }

        // Need to compress - prioritize behavioral context
        let behavioral_budget = (max_tokens as f32 * 0.6) as usize;
        let work_budget = max_tokens - behavioral_budget;

        let compressed_behavioral = if behavioral_tokens > behavioral_budget {
            self.compress_text(behavioral, behavioral_budget)
        } else {
            behavioral.to_string()
        };

        let compressed_work = if work_tokens > work_budget {
            self.compress_text(work, work_budget)
        } else {
            work.to_string()
        };

        Ok(CompressedContext {
            behavioral: compressed_behavioral,
            work: compressed_work,
        })
    }

    fn compress_text(&self, text: &str, target_tokens: usize) -> String {
        let target_chars = target_tokens * 4;
        
        if text.len() <= target_chars {
            return text.to_string();
        }

        // Split into sentences and keep the most important ones
        let sentences: Vec<&str> = text.split('.').collect();
        let mut result = String::new();

        for sentence in sentences {
            if result.len() + sentence.len() + 1 <= target_chars {
                if !result.is_empty() {
                    result.push_str(". ");
                }
                result.push_str(sentence.trim());
            } else {
                break;
            }
        }

        result
    }
}

#[derive(Debug, Clone)]
pub struct CompressedContext {
    pub behavioral: String,
    pub work: String,
}

/// Filters context for privacy
pub struct PrivacyFilter;

impl PrivacyFilter {
    pub fn new() -> Self {
        Self
    }

    pub fn filter(&self, analysis: &WorkAnalysis) -> Result<WorkAnalysis> {
        let mut filtered = analysis.clone();

        // Remove potentially sensitive file paths, URLs, etc.
        filtered.relevant_context = self.sanitize_text(&filtered.relevant_context);
        filtered.task_description = self.sanitize_text(&filtered.task_description);

        // Filter domain suggestions for any sensitive content
        filtered.domain_suggestions = filtered
            .domain_suggestions
            .into_iter()
            .map(|s| self.sanitize_text(&s))
            .collect();

        Ok(filtered)
    }

    fn sanitize_text(&self, text: &str) -> String {
        // Basic sanitization - remove file paths, URLs, etc.
        let mut sanitized = text.to_string();

        // Remove file paths
        sanitized = regex::Regex::new(r"/[^\s]+")
            .unwrap()
            .replace_all(&sanitized, "[FILE_PATH]")
            .to_string();

        // Remove URLs
        sanitized = regex::Regex::new(r"https?://[^\s]+")
            .unwrap()
            .replace_all(&sanitized, "[URL]")
            .to_string();

        sanitized
    }
}

// Stub implementations for complex components
pub struct LanguageDetector;
impl LanguageDetector {
    pub fn new() -> Self { Self }
}

pub struct TaskClassifier;
impl TaskClassifier {
    pub fn new() -> Self { Self }
    
    pub fn classify_coding_task(&self, window_title: &str, _screen_text: Option<&str>) -> Result<String> {
        if window_title.contains("test") {
            Ok("Writing or running tests".to_string())
        } else if window_title.contains("debug") {
            Ok("Debugging code".to_string())
        } else if window_title.contains("README") || window_title.contains("doc") {
            Ok("Writing documentation".to_string())
        } else {
            Ok("Coding".to_string())
        }
    }
}

pub struct ComplexityAnalyzer;
impl ComplexityAnalyzer {
    pub fn new() -> Self { Self }
    
    pub fn analyze_code_complexity(&self, _language: &str, screen_text: Option<&str>) -> Result<ComplexityLevel> {
        if let Some(text) = screen_text {
            let line_count = text.lines().count();
            if line_count > 100 {
                Ok(ComplexityLevel::Complex)
            } else if line_count > 50 {
                Ok(ComplexityLevel::Moderate)
            } else {
                Ok(ComplexityLevel::Simple)
            }
        } else {
            Ok(ComplexityLevel::Simple)
        }
    }
    
    pub fn analyze_writing_complexity(&self, screen_text: Option<&str>) -> Result<ComplexityLevel> {
        if let Some(text) = screen_text {
            let word_count = text.split_whitespace().count();
            if word_count > 1000 {
                Ok(ComplexityLevel::Complex)
            } else if word_count > 500 {
                Ok(ComplexityLevel::Moderate)
            } else {
                Ok(ComplexityLevel::Simple)
            }
        } else {
            Ok(ComplexityLevel::Simple)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ADHDStateType, MessageStyle, InterventionFrequency, UserPrivacyLevel, APIConsent};

    #[test]
    fn test_work_analysis() {
        let analyzer = WorkContextAnalyzer::new();
        let context = WorkContext {
            work_type: WorkType::Coding {
                language: "rust".to_string(),
                framework: Some("tokio".to_string()),
            },
            application: "vscode".to_string(),
            window_title: "main.rs - test project".to_string(),
            screenshot_text: Some("fn main() { println!(\"Hello\"); }".to_string()),
            task_category: TaskCategory::Work,
            urgency: UrgencyLevel::Medium,
            time_of_day: crate::types::TimeOfDay::Afternoon,
        };

        let analysis = analyzer.analyze(&context).unwrap();
        assert!(analysis.task_description.contains("test"));
    }

    #[test]
    fn test_context_compression() {
        let compressor = ContextCompressor::new();
        let long_text = "This is a very long piece of text that should be compressed when it exceeds the token budget. ".repeat(50);
        
        let result = compressor.compress(&long_text, "short work context", 100).unwrap();
        
        // Should be compressed
        assert!(result.behavioral.len() < long_text.len());
    }

    #[test]
    fn test_behavioral_summary() {
        let builder = BehavioralContextBuilder::new();
        let metrics = BehavioralMetrics {
            productive_time_ratio: 0.8,
            distraction_frequency: 0.2,
            focus_session_count: 5,
            average_session_length: 1800,
            recovery_time: 300,
            transition_smoothness: 0.7,
        };

        let current_state = ADHDState {
            state_type: ADHDStateType::Flow { depth: 0.8 },
            confidence: 0.9,
            depth: Some(0.8),
            duration: 1000,
            metadata: std::collections::HashMap::new(),
        };

        let summary = builder.summarize(&[], &metrics, &current_state).unwrap();
        assert!(summary.contains("80%"));
        assert!(summary.contains("Flow"));
    }
}