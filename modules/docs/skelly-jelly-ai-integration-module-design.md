# AI Integration Module Design

## Module Purpose and Responsibilities

The AI Integration Module provides context-aware, helpful suggestions and personality-driven interactions for Skelly-Jelly. It uses local LLM inference as the primary method (privacy-first) with fallback to OpenAI/Anthropic APIs when needed. The module maintains the skeleton companion's personality while generating genuinely useful work-specific assistance.

### Core Responsibilities
- **Local LLM Management**: Run and manage local language models efficiently
- **Context Understanding**: Analyze work context to provide relevant help
- **Suggestion Generation**: Create helpful, non-patronizing suggestions
- **Personality Consistency**: Maintain the chill, supportive skeleton personality
- **API Fallback**: Seamlessly use cloud APIs when local models insufficient
- **Privacy Protection**: Ensure no sensitive data leaves device unless explicitly allowed

## Key Components and Their Functions

### 1. LLM Manager
```rust
pub struct LLMManager {
    // Local model instance
    local_model: Option<LocalLLM>,
    
    // API clients for fallback
    openai_client: Option<OpenAIClient>,
    anthropic_client: Option<AnthropicClient>,
    
    // Model selector
    model_selector: ModelSelector,
    
    // Context manager
    context_manager: ContextManager,
    
    // Response cache
    response_cache: LRUCache<String, GeneratedResponse>,
}

pub struct LocalLLM {
    // llama.cpp instance
    model: LlamaModel,
    
    // Model configuration
    config: LocalModelConfig,
    
    // Memory pool for efficient allocation
    memory_pool: MemoryPool,
    
    // KV cache for conversation continuity
    kv_cache: KVCache,
}

pub struct LocalModelConfig {
    pub model_path: PathBuf,           // Path to GGUF file
    pub n_gpu_layers: i32,             // Layers to offload to GPU
    pub context_length: usize,         // Default: 4096
    pub batch_size: usize,             // Default: 512
    pub threads: usize,                // CPU threads
    pub use_mmap: bool,                // Memory-mapped file loading
    pub use_mlock: bool,               // Lock model in RAM
}

impl LLMManager {
    pub async fn initialize(&mut self) -> Result<()> {
        // Try to load local model first
        match self.load_local_model().await {
            Ok(model) => {
                self.local_model = Some(model);
                info!("Local LLM loaded successfully");
            }
            Err(e) => {
                warn!("Failed to load local model: {}, will use API fallback", e);
            }
        }
        
        // Initialize API clients if keys are available
        self.initialize_api_clients().await?;
        
        Ok(())
    }
    
    async fn load_local_model(&self) -> Result<LocalLLM> {
        // Detect available resources
        let gpu_available = self.detect_gpu_support();
        let available_memory = self.get_available_memory();
        
        // Select appropriate model based on resources
        let model_path = self.select_model_variant(available_memory, gpu_available)?;
        
        // Configure based on hardware
        let config = LocalModelConfig {
            model_path,
            n_gpu_layers: if gpu_available { 32 } else { 0 },
            context_length: 4096,
            batch_size: 512,
            threads: num_cpus::get_physical(),
            use_mmap: true,
            use_mlock: available_memory > 8 * 1024 * 1024 * 1024, // Lock if >8GB RAM
        };
        
        // Load model
        let model = LlamaModel::load(&config)?;
        
        Ok(LocalLLM {
            model,
            config,
            memory_pool: MemoryPool::new(100 * 1024 * 1024), // 100MB pool
            kv_cache: KVCache::new(config.context_length),
        })
    }
}
```

### 2. Context Processor
```rust
pub struct ContextProcessor {
    // Work context analyzer
    work_analyzer: WorkContextAnalyzer,
    
    // Behavioral context builder
    behavioral_builder: BehavioralContextBuilder,
    
    // Context compression
    context_compressor: ContextCompressor,
    
    // Privacy filter
    privacy_filter: PrivacyFilter,
}

impl ContextProcessor {
    pub async fn build_context(
        &self,
        intervention_request: &InterventionRequest,
        state_history: &[ADHDState],
        work_context: &WorkContext,
    ) -> Result<LLMContext> {
        // Build behavioral summary
        let behavioral_summary = self.behavioral_builder.summarize(
            state_history,
            &intervention_request.metrics,
        )?;
        
        // Analyze work context
        let work_analysis = self.work_analyzer.analyze(work_context)?;
        
        // Apply privacy filtering
        let filtered_context = self.privacy_filter.filter(&work_analysis)?;
        
        // Compress to fit token budget
        let compressed = self.context_compressor.compress(
            &behavioral_summary,
            &filtered_context,
            MAX_CONTEXT_TOKENS,
        )?;
        
        Ok(LLMContext {
            system_prompt: self.build_system_prompt(),
            behavioral_context: compressed.behavioral,
            work_context: compressed.work,
            intervention_type: intervention_request.intervention_type,
            user_preferences: intervention_request.user_preferences,
        })
    }
    
    fn build_system_prompt(&self) -> String {
        r#"You are Skelly, a melty skeleton companion helping someone with ADHD stay focused.

Your personality:
- Chill and supportive, never patronizing
- Use casual language, occasional skeleton puns (sparingly)
- Celebrate small wins, acknowledge struggles without judgment
- Brief messages (1-2 sentences usually)
- Focus on actionable, specific help

Current context will include:
- User's current focus state and work type
- Recent behavioral patterns
- Specific area where they might need help

Respond with helpful suggestions that are:
- Specific to their current task
- Easy to implement right now
- Encouraging without being over the top
"#.to_string()
    }
}

pub struct WorkContextAnalyzer {
    // Programming language detector
    language_detector: LanguageDetector,
    
    // Task classifier
    task_classifier: TaskClassifier,
    
    // Complexity analyzer
    complexity_analyzer: ComplexityAnalyzer,
}

impl WorkContextAnalyzer {
    pub fn analyze(&self, context: &WorkContext) -> Result<WorkAnalysis> {
        let analysis = match &context.work_type {
            WorkType::Coding { language, framework } => {
                self.analyze_coding_context(language, framework, &context.screenshot_text)
            }
            WorkType::Writing { document_type } => {
                self.analyze_writing_context(document_type, &context.screenshot_text)
            }
            WorkType::Design { tool, project_type } => {
                self.analyze_design_context(tool, project_type)
            }
            _ => WorkAnalysis::default(),
        }?;
        
        Ok(analysis)
    }
    
    fn analyze_coding_context(
        &self,
        language: &str,
        framework: &Option<String>,
        code_text: &str,
    ) -> Result<WorkAnalysis> {
        // Detect what they're working on
        let task = self.task_classifier.classify_coding_task(code_text)?;
        
        // Assess complexity
        let complexity = self.complexity_analyzer.analyze_code(code_text)?;
        
        // Identify potential stuck points
        let stuck_indicators = self.detect_stuck_patterns(code_text)?;
        
        Ok(WorkAnalysis {
            task_description: task,
            complexity_level: complexity,
            potential_issues: stuck_indicators,
            relevant_context: self.extract_relevant_code(code_text)?,
        })
    }
}
```

### 3. Suggestion Generator
```rust
pub struct SuggestionGenerator {
    // Template manager for quick responses
    template_manager: TemplateManager,
    
    // LLM interface
    llm_interface: LLMInterface,
    
    // Suggestion validator
    validator: SuggestionValidator,
    
    // Personality engine
    personality: PersonalityEngine,
}

impl SuggestionGenerator {
    pub async fn generate(
        &self,
        context: LLMContext,
        urgency: SuggestionUrgency,
    ) -> Result<InterventionResponse> {
        // Decide whether to use template or LLM
        let use_template = self.should_use_template(&context, urgency);
        
        let raw_suggestion = if use_template {
            self.template_manager.get_suggestion(&context)?
        } else {
            self.generate_llm_suggestion(&context).await?
        };
        
        // Apply personality modifications
        let personalized = self.personality.apply(raw_suggestion, &context)?;
        
        // Validate suggestion
        let validated = self.validator.validate(personalized)?;
        
        Ok(InterventionResponse {
            message: validated.message,
            animation_hints: validated.animation_hints,
            follow_up_available: validated.has_follow_up,
            confidence: validated.confidence,
        })
    }
    
    async fn generate_llm_suggestion(&self, context: &LLMContext) -> Result<String> {
        // Build prompt
        let prompt = self.build_prompt(context)?;
        
        // Try local model first
        if let Some(ref local_model) = self.llm_interface.local_model {
            match self.generate_local(local_model, &prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("Local generation failed: {}, trying API", e);
                }
            }
        }
        
        // Fall back to API
        self.generate_via_api(&prompt).await
    }
    
    async fn generate_local(
        &self,
        model: &LocalLLM,
        prompt: &str,
    ) -> Result<String> {
        let params = GenerationParams {
            max_tokens: 150,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            stop_sequences: vec!["\n\n".to_string()],
        };
        
        let response = model.generate(prompt, params).await?;
        
        Ok(response.text)
    }
}

pub struct TemplateManager {
    templates: HashMap<(InterventionType, WorkType), Vec<String>>,
}

impl TemplateManager {
    fn get_suggestion(&self, context: &LLMContext) -> Result<String> {
        let templates = self.templates
            .get(&(context.intervention_type.clone(), context.work_context.work_type.clone()))
            .ok_or_else(|| anyhow!("No templates for this context"))?;
        
        // Select random template
        let template = templates.choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow!("No templates available"))?;
        
        // Fill in variables
        self.fill_template(template, context)
    }
}
```

### 4. API Fallback Manager
```rust
pub struct APIFallbackManager {
    // API clients
    openai: Option<OpenAIClient>,
    anthropic: Option<AnthropicClient>,
    
    // Usage tracker
    usage_tracker: UsageTracker,
    
    // Privacy guardian
    privacy_guardian: PrivacyGuardian,
    
    // Request queue for rate limiting
    request_queue: RequestQueue,
}

impl APIFallbackManager {
    pub async fn generate(
        &self,
        prompt: &str,
        context: &LLMContext,
    ) -> Result<String> {
        // Check privacy settings
        if !self.privacy_guardian.allow_api_request(context)? {
            return Err(anyhow!("API request blocked by privacy settings"));
        }
        
        // Sanitize prompt
        let sanitized = self.privacy_guardian.sanitize_prompt(prompt)?;
        
        // Select API based on availability and cost
        let api_choice = self.select_api()?;
        
        // Queue request for rate limiting
        self.request_queue.enqueue(api_choice.clone()).await?;
        
        // Make API call
        let response = match api_choice {
            APIChoice::OpenAI => {
                self.call_openai(&sanitized).await?
            }
            APIChoice::Anthropic => {
                self.call_anthropic(&sanitized).await?
            }
        };
        
        // Track usage
        self.usage_tracker.record(api_choice, response.tokens_used)?;
        
        Ok(response.text)
    }
    
    async fn call_openai(&self, prompt: &str) -> Result<APIResponse> {
        let client = self.openai.as_ref()
            .ok_or_else(|| anyhow!("OpenAI client not initialized"))?;
        
        let request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: SKELLY_SYSTEM_PROMPT.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: 150,
            temperature: 0.7,
        };
        
        client.complete(request).await
    }
}

pub struct PrivacyGuardian {
    // PII detector
    pii_detector: PIIDetector,
    
    // Sensitive pattern matcher
    pattern_matcher: SensitivePatternMatcher,
    
    // Anonymizer
    anonymizer: DataAnonymizer,
}

impl PrivacyGuardian {
    pub fn sanitize_prompt(&self, prompt: &str) -> Result<String> {
        let mut sanitized = prompt.to_string();
        
        // Remove PII
        for pii in self.pii_detector.detect(&sanitized)? {
            sanitized = sanitized.replace(&pii.text, &pii.placeholder);
        }
        
        // Remove sensitive patterns
        sanitized = self.pattern_matcher.redact_sensitive(&sanitized)?;
        
        // Anonymize remaining identifiers
        sanitized = self.anonymizer.anonymize(&sanitized)?;
        
        Ok(sanitized)
    }
}
```

### 5. Personality Engine
```rust
pub struct PersonalityEngine {
    // Personality traits
    traits: PersonalityTraits,
    
    // Mood tracker
    mood_tracker: MoodTracker,
    
    // Expression generator
    expression_generator: ExpressionGenerator,
    
    // Skeleton pun generator (used sparingly)
    pun_generator: SkeletonPunGenerator,
}

pub struct PersonalityTraits {
    pub cheerfulness: f32,      // 0.0-1.0, default 0.7
    pub humor: f32,             // 0.0-1.0, default 0.5  
    pub supportiveness: f32,    // 0.0-1.0, default 0.9
    pub casualness: f32,        // 0.0-1.0, default 0.8
    pub pun_frequency: f32,     // 0.0-1.0, default 0.1 (rare)
}

impl PersonalityEngine {
    pub fn apply(&self, suggestion: String, context: &LLMContext) -> Result<String> {
        let mut modified = suggestion;
        
        // Adjust tone based on user state
        modified = self.adjust_tone(&modified, &context.behavioral_context)?;
        
        // Add personality flair
        modified = self.add_personality(&modified)?;
        
        // Maybe add a skeleton pun (rarely)
        if self.should_add_pun() {
            modified = self.add_skeleton_pun(&modified)?;
        }
        
        // Ensure appropriate length
        modified = self.ensure_brevity(&modified)?;
        
        Ok(modified)
    }
    
    fn adjust_tone(&self, message: &str, context: &BehavioralContext) -> Result<String> {
        match context.current_state {
            ADHDState::Flow { .. } => {
                // More subdued, don't break flow
                Ok(self.make_gentle(message))
            }
            ADHDState::Distracted { severity, .. } if severity > 0.7 => {
                // More encouraging, less pressure
                Ok(self.make_encouraging(message))
            }
            _ => Ok(message.to_string()),
        }
    }
    
    fn add_skeleton_pun(&self, message: &str) -> Result<String> {
        let puns = vec![
            ("bone", "I've got a bone to pick with distractions!"),
            ("skull", "Let's put our skulls together on this"),
            ("spine", "You've got the spine for this!"),
            ("marrow", "Getting to the marrow of it..."),
        ];
        
        // Only add if message doesn't already have a pun
        if puns.iter().any(|(word, _)| message.contains(word)) {
            return Ok(message.to_string());
        }
        
        // Rarely add a pun
        if let Some((_, pun)) = puns.choose(&mut rand::thread_rng()) {
            Ok(format!("{} {}", message, pun))
        } else {
            Ok(message.to_string())
        }
    }
}
```

## Integration Points with Other Modules

### Input Sources
- **Gamification Module**: Receives InterventionRequest with context
- **Analysis Engine**: Gets work context and behavioral state

### Output Consumers
- **Cute Figurine**: Sends AnimationCommand based on generated content
- **Gamification Module**: Returns InterventionResponse

### Data Flow
```
Gamification --InterventionRequest--> AI Integration
                                          |
                                          ├─> Process Context
                                          ├─> Generate Suggestion
                                          ├─> Apply Personality
                                          └─> Return Response
                                                |
                                                v
                                          Cute Figurine
```

## Technology Choices

### Core Technology: Rust
- **Reasoning**: Performance for local inference, memory safety, good FFI for llama.cpp
- **Key Libraries**:
  - `llama-cpp-rs`: Rust bindings for llama.cpp
  - `candle`: Alternative lightweight inference
  - `reqwest`: HTTP client for API calls
  - `tiktoken-rs`: Token counting

### Local Model Strategy
- **Primary Model**: Mistral 7B Instruct (4-bit quantized, ~4GB)
- **Fallback Model**: Phi-3 Mini (3.8B params, ~2GB)
- **Quantization**: 4-bit via GGUF format
- **Acceleration**: Metal on macOS, CUDA on supported GPUs

### API Integration
- **OpenAI**: GPT-3.5-turbo for cost efficiency
- **Anthropic**: Claude Haiku for complex reasoning
- **Caching**: Aggressive caching of similar prompts

## Data Structures and Interfaces

### Public API
```rust
#[async_trait]
pub trait AIIntegration: Send + Sync {
    /// Process an intervention request
    async fn process_intervention(
        &self,
        request: InterventionRequest,
    ) -> Result<InterventionResponse>;
    
    /// Generate animation command from text
    async fn generate_animation(
        &self,
        text: &str,
        mood: CompanionMood,
    ) -> Result<AnimationCommand>;
    
    /// Update personality settings
    async fn update_personality(
        &self,
        traits: PersonalityTraits,
    ) -> Result<()>;
    
    /// Get usage statistics
    async fn get_usage_stats(&self) -> UsageStatistics;
}

pub struct InterventionRequest {
    pub intervention_type: InterventionType,
    pub current_state: ADHDState,
    pub behavioral_metrics: BehavioralMetrics,
    pub work_context: WorkContext,
    pub state_history: Vec<ADHDState>,
    pub user_preferences: UserPreferences,
}

pub struct InterventionResponse {
    pub message: String,
    pub animation_hints: Vec<AnimationHint>,
    pub follow_up_available: bool,
    pub confidence: f32,
    pub tokens_used: Option<u32>,
}
```

### Model Configuration
```rust
pub struct AIIntegrationConfig {
    // Local model settings
    pub local_model: LocalModelSettings,
    
    // API settings
    pub api_config: APIConfig,
    
    // Privacy settings
    pub privacy: PrivacySettings,
    
    // Performance settings
    pub performance: PerformanceSettings,
}

pub struct LocalModelSettings {
    pub model_path: Option<PathBuf>,
    pub auto_download: bool,           // Download model if missing
    pub model_variant: ModelVariant,   // Which model to use
    pub max_memory_gb: f32,           // Max memory for model
    pub use_gpu: bool,                // Enable GPU acceleration
    pub gpu_layers: Option<i32>,      // Layers to offload
}

pub enum ModelVariant {
    Mistral7B,      // Best quality, 4GB
    Phi3Mini,       // Smaller, 2GB  
    TinyLlama,      // Tiny, 1GB
    Custom(String), // User-provided model
}

pub struct APIConfig {
    pub openai_key: Option<String>,
    pub anthropic_key: Option<String>,
    pub max_monthly_cost: Option<f32>,
    pub prefer_local: bool,            // Always try local first
}

pub struct PrivacySettings {
    pub allow_api_fallback: bool,      // Default: false
    pub api_consent_prompt: bool,      // Ask before first API use
    pub sanitize_prompts: bool,        // Remove PII
    pub log_prompts: bool,             // For debugging
}
```

## Performance Considerations

### Local Inference Performance
- **Model Loading**: 2-5 seconds on first launch
- **Inference Speed**: 10-30 tokens/second on M3 Pro
- **Memory Usage**: 4-6GB for model + overhead
- **Response Time**: <500ms for typical suggestions

### Optimization Strategies
1. **Model Quantization**: 4-bit models for 4x size reduction
2. **KV Caching**: Reuse context across messages
3. **Prompt Caching**: Cache similar prompts/responses
4. **Batch Processing**: Group multiple requests
5. **GPU Offloading**: Use Metal/CUDA when available

### Resource Management
- **Memory Pool**: Pre-allocated buffers for inference
- **Model Swapping**: Unload model during idle periods
- **Context Window**: Sliding window for conversation history
- **Token Budgeting**: Strict limits on prompt/response size

## Error Handling Strategies

### Failure Modes
```rust
pub enum AIIntegrationError {
    // Model loading failures
    ModelLoadFailed { reason: String },
    ModelNotFound { path: PathBuf },
    InsufficientMemory { required: usize, available: usize },
    
    // Inference failures
    InferenceFailed { model: String, error: String },
    ContextTooLong { tokens: usize, max: usize },
    
    // API failures
    APIKeyMissing { service: String },
    APIRateLimited { retry_after: Duration },
    APIError { service: String, status: u16, message: String },
    
    // Privacy failures
    PrivacyViolation { reason: String },
    ConsentRequired { action: String },
}
```

### Graceful Degradation
1. **Local Model Fails**: Fall back to templates
2. **API Unavailable**: Use cached responses
3. **Context Too Long**: Compress or truncate
4. **Memory Pressure**: Use smaller model
5. **All Fails**: Simple templated messages

### Recovery Procedures
- **Model Corruption**: Re-download model
- **Memory Issues**: Clear caches and retry
- **API Errors**: Exponential backoff
- **Privacy Blocks**: Respect and inform user

## Security Considerations

### Model Security
- **Model Validation**: Checksum verification
- **Secure Download**: HTTPS only, verify signatures
- **Sandboxing**: Run inference in restricted environment
- **Input Sanitization**: Prevent prompt injection

### Privacy Protection
- **Local First**: Default to local processing
- **Data Minimization**: Only send necessary context
- **PII Scrubbing**: Remove identifiable information
- **Audit Logging**: Track what data leaves device

### API Security
- **Key Storage**: Encrypted local storage
- **Request Signing**: Verify API authenticity
- **Rate Limiting**: Prevent abuse
- **Cost Controls**: Hard limits on API spend

## Testing Strategy

### Unit Tests
- Context processing accuracy
- Template generation variety
- Personality consistency
- Privacy filter effectiveness

### Integration Tests
- Local model loading and inference
- API fallback behavior
- Context size management
- Error recovery flows

### Performance Tests
- Inference speed benchmarks
- Memory usage profiling
- API response times
- Cache hit rates

### Quality Tests
- Suggestion relevance scoring
- Personality trait adherence
- Pun frequency validation
- Message brevity checks