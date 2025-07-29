# AI Integration Module

Privacy-first AI assistance module for Skelly Jelly with local LLM support and secure API fallback.

## Overview

The AI Integration module provides context-aware, personality-driven assistance using a local-first approach with optional API fallback. It maintains Skelly's chill, supportive skeleton companion personality while ensuring user privacy and data security.

## Key Features

### 🔒 Privacy-First Design
- **Local Processing**: Defaults to local LLM inference, no data leaves device
- **PII Detection**: Comprehensive personally identifiable information detection
- **Prompt Sanitization**: Removes sensitive data before any external processing
- **Consent Management**: Explicit user consent required for API usage

### 🤖 Local LLM Support
- **Multiple Models**: Support for Mistral 7B, Phi-3 Mini, TinyLlama, and custom models
- **Hardware Optimization**: Automatic GPU detection and memory management
- **Efficient Inference**: 4-bit quantization, memory pooling, and context caching
- **Fast Response**: <500ms typical response time for local inference

### 🛡️ Security Features
- **Prompt Injection Protection**: Detects and blocks malicious prompt manipulation
- **Secure API Handling**: Encrypted key storage and request sanitization
- **Audit Logging**: Comprehensive privacy-compliant logging
- **Error Handling**: Security-conscious error messages that don't leak information

### 🎭 Personality Engine
- **Consistent Character**: Maintains Skelly's supportive, chill personality
- **Context Awareness**: Adapts tone based on user state and work context
- **Skeleton Puns**: Occasional, appropriate skeleton-themed humor
- **Brevity Focus**: Keeps messages helpful but concise

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Gamification   │───▶│ AI Integration  │───▶│ Cute Figurine   │
│     Module      │    │     Module      │    │     Module      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                               │
                       ┌───────┼───────┐
                       ▼       ▼       ▼
                 ┌─────────┐ ┌───┐ ┌─────────┐
                 │Local LLM│ │API│ │Templates│
                 └─────────┘ └───┘ └─────────┘
```

### Core Components

1. **Context Processor**: Analyzes work context and behavioral patterns
2. **LLM Manager**: Handles local model loading and API fallback
3. **Privacy Guardian**: Protects user data and enforces privacy policies
4. **Personality Engine**: Applies Skelly's character to all responses
5. **Suggestion Generator**: Creates helpful, contextual advice

## Configuration

### Privacy-Focused Setup (Recommended)
```rust
use ai_integration::AIIntegrationConfig;

let config = AIIntegrationConfig::privacy_focused();
// - Local-only processing
// - No API fallback
// - PII detection enabled
// - Audit logging active
```

### Performance-Focused Setup
```rust
let config = AIIntegrationConfig::performance_focused();
// - GPU acceleration enabled
// - Memory optimization active
// - Parallel processing enabled
// - Background model loading
```

### Minimal Resources Setup
```rust
let config = AIIntegrationConfig::minimal_resources();
// - TinyLlama model (1GB)
// - Reduced context window
// - Lower memory usage
```

## Usage

### Basic Usage
```rust
use ai_integration::{AIIntegration, AIIntegrationImpl, AIIntegrationConfig};

// Create and initialize
let config = AIIntegrationConfig::default();
let mut ai = AIIntegrationImpl::new(config);
ai.initialize().await?;

// Process intervention request
let response = ai.process_intervention(request).await?;
println!("Skelly says: {}", response.response_text);
```

### Custom Personality
```rust
use ai_integration::PersonalityTraits;

let traits = PersonalityTraits {
    cheerfulness: 0.8,
    humor: 0.6,
    supportiveness: 0.9,
    casualness: 0.7,
    pun_frequency: 0.15, // More puns!
    directness: 0.5,
};

ai.update_personality(traits).await?;
```

### Health Monitoring
```rust
let health = ai.health_check().await;
println!("Local model: {:?}", health.local_model_status);
println!("Memory usage: {}MB", health.memory_usage_mb);
println!("Response time: {}ms", health.response_time_p95_ms);
```

## Local Model Setup

### Supported Models

| Model | Size | Memory | Performance | Use Case |
|-------|------|--------|-------------|----------|
| TinyLlama | 1GB | 2GB RAM | Fast | Low-resource devices |
| Phi-3 Mini | 2GB | 4GB RAM | Balanced | General use |
| Mistral 7B | 4GB | 8GB RAM | High Quality | High-end devices |

### Model Installation
1. Download GGUF format model from HuggingFace
2. Place in `~/.cache/skelly-jelly/models/`
3. Update config with model path
4. Restart application

Example:
```bash
# Download Phi-3 Mini (example - actual URLs may vary)
wget https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_0.gguf

# Move to cache directory
mkdir -p ~/.cache/skelly-jelly/models
mv Phi-3-mini-4k-instruct-q4_0.gguf ~/.cache/skelly-jelly/models/
```

## API Configuration (Optional)

### OpenAI Setup
```rust
let mut config = AIIntegrationConfig::default();
config.api_config.openai_key = Some("sk-...".to_string());
config.api_config.max_monthly_cost = Some(5.0); // $5 limit
config.privacy.allow_api_fallback = true;
```

### Anthropic Setup
```rust
config.api_config.anthropic_key = Some("sk-ant-...".to_string());
config.api_config.anthropic_model = "claude-3-haiku-20240307".to_string();
```

## Privacy Controls

### Local-Only Mode
```rust
config.privacy.default_privacy_level = UserPrivacyLevel::LocalOnly;
config.privacy.allow_api_fallback = false;
// All processing stays on device
```

### Sanitized API Mode
```rust
config.privacy.default_privacy_level = UserPrivacyLevel::SanitizedAPI;
config.privacy.sanitize_prompts = true;
// PII removed before API calls
```

### Consent-Based Mode
```rust
config.privacy.api_consent_prompt = true;
// User prompted before first API use
```

## Performance Optimization

### Memory Management
- **Model Quantization**: 4-bit models reduce memory by 75%
- **Context Compression**: Intelligent token budget management
- **Memory Pooling**: Pre-allocated buffers for efficiency
- **GPU Offloading**: Automatic Metal/CUDA acceleration

### Response Time Optimization
- **Template Fallback**: Instant responses for common scenarios
- **Response Caching**: Cache similar prompts and responses
- **Parallel Processing**: Concurrent request handling
- **Batch Operations**: Group multiple requests efficiently

## Security Considerations

### Threat Model
- **Data Privacy**: No sensitive data leaves device without consent
- **Prompt Injection**: Detection and blocking of malicious inputs
- **Model Security**: Checksum verification of downloaded models
- **API Security**: Encrypted storage of API keys

### Security Features
- **PII Detection**: Email, phone, SSN, credit card detection
- **Prompt Sanitization**: Remove file paths, URLs, personal info
- **Audit Logging**: Track what data (if any) leaves device
- **Rate Limiting**: Prevent API abuse and cost overruns

## Error Handling

### Graceful Degradation
1. **Local Model Fails** → Template responses
2. **API Unavailable** → Cached responses or templates
3. **Context Too Long** → Intelligent compression
4. **Memory Pressure** → Switch to smaller model
5. **All Systems Fail** → Simple encouraging messages

### Error Recovery
- **Exponential Backoff**: For temporary API failures
- **Model Reloading**: Automatic recovery from corruption
- **Memory Cleanup**: Clear caches and retry
- **User Notification**: Clear, helpful error messages

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test integration_test
```

### Performance Tests
```bash
cargo test --release --test performance_test
```

## Examples

See the `examples/` directory for:
- `local_inference_example.rs` - Local model usage
- `api_fallback_example.rs` - API integration
- `privacy_demo.rs` - Privacy features demonstration

## Contributing

1. Follow privacy-first principles
2. Add comprehensive tests for new features
3. Update documentation for API changes
4. Ensure security review for any external communication

## License

MIT License - see LICENSE file for details.

## Support

- Check logs in `~/.local/share/skelly-jelly/logs/`
- Health check endpoint for monitoring
- Comprehensive error messages with recovery suggestions