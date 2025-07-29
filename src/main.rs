//! Skelly-Jelly: Your ADHD companion with a melty skeleton friend
//! 
//! This is the main entry point that orchestrates all modules for real-time
//! ADHD state detection and supportive interventions.

use anyhow::{Context, Result};
use std::{sync::Arc, time::Duration};
use tokio::{signal, time::sleep};
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::Deserialize;

// Import all modules
use skelly_jelly_event_bus::{EventBus, Event};
use orchestrator::{Orchestrator, ModuleConfig};
use skelly_jelly_data_capture::{DataCapture, BehaviorEvent};
use skelly_jelly_storage::{Storage, StorageConfig};
use skelly_jelly_analysis_engine::{AnalysisEngine, AdhdState};
use ai_integration::{AiIntegration, InterventionRequest};

#[derive(Debug, Deserialize)]
struct Config {
    event_bus: EventBusConfig,
    orchestrator: OrchestratorConfig,
    storage: StorageConfig,
    data_capture: DataCaptureConfig,
    analysis_engine: AnalysisConfig,
    ai_integration: AiConfig,
}

#[derive(Debug, Deserialize)]
struct EventBusConfig {
    max_queue_size: usize,
    message_timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
struct OrchestratorConfig {
    health_check_interval_ms: u64,
    startup_timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
struct DataCaptureConfig {
    sample_rate_hz: f64,
    window_size_seconds: u32,
}

#[derive(Debug, Deserialize)]
struct AnalysisConfig {
    model_path: String,
    confidence_threshold: f64,
}

#[derive(Debug, Deserialize)]
struct AiConfig {
    privacy_level: String,
    use_local_model: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            event_bus: EventBusConfig {
                max_queue_size: 10000,
                message_timeout_ms: 5000,
            },
            orchestrator: OrchestratorConfig {
                health_check_interval_ms: 30000,
                startup_timeout_ms: 60000,
            },
            storage: StorageConfig {
                database_path: "./data/skelly.db".to_string(),
                max_batch_size: 1000,
                batch_timeout_ms: 30000,
                retention_days: 30,
                enable_compression: true,
            },
            data_capture: DataCaptureConfig {
                sample_rate_hz: 10.0,
                window_size_seconds: 30,
            },
            analysis_engine: AnalysisConfig {
                model_path: "./models/adhd_classifier.onnx".to_string(),
                confidence_threshold: 0.7,
            },
            ai_integration: AiConfig {
                privacy_level: "LocalOnly".to_string(),
                use_local_model: true,
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,skelly_jelly=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🦴 Skelly-Jelly Starting!");
    info!("Your melty skeleton companion is awakening...");
    
    // Load configuration
    let config = load_config().await?;
    
    // Initialize the system
    let system = SkellyJellySystem::new(config).await?;
    
    // Start all modules
    system.start().await?;
    
    // Wait for shutdown signal
    info!("✨ System ready! Press Ctrl+C to stop.");
    signal::ctrl_c().await?;
    
    // Graceful shutdown
    info!("🛑 Shutting down gracefully...");
    system.shutdown().await?;
    
    info!("👋 Your skeleton friend will miss you!");
    Ok(())
}

struct SkellyJellySystem {
    event_bus: Arc<EventBus>,
    orchestrator: Arc<Orchestrator>,
    data_capture: Arc<DataCapture>,
    storage: Arc<Storage>,
    analysis_engine: Arc<AnalysisEngine>,
    ai_integration: Arc<AiIntegration>,
}

impl SkellyJellySystem {
    async fn new(config: Config) -> Result<Self> {
        info!("🔧 Initializing Skelly-Jelly modules...");
        
        // Create event bus first - all modules need it
        let event_bus = Arc::new(
            EventBus::new(config.event_bus.max_queue_size)
                .context("Failed to create event bus")?
        );
        
        // Initialize storage
        let storage = Arc::new(
            Storage::new(config.storage).await
                .context("Failed to initialize storage")?
        );
        
        // Initialize data capture
        let data_capture = Arc::new(
            DataCapture::new(
                config.data_capture.sample_rate_hz,
                config.data_capture.window_size_seconds,
                event_bus.clone()
            ).await?
        );
        
        // Initialize analysis engine
        let analysis_engine = Arc::new(
            AnalysisEngine::new(
                &config.analysis_engine.model_path,
                config.analysis_engine.confidence_threshold,
                event_bus.clone()
            ).await?
        );
        
        // Initialize AI integration
        let ai_integration = Arc::new(
            AiIntegration::new(
                config.ai_integration.use_local_model,
                &config.ai_integration.privacy_level,
                event_bus.clone()
            ).await?
        );
        
        // Initialize orchestrator last
        let orchestrator = Arc::new(
            Orchestrator::new(
                Duration::from_millis(config.orchestrator.health_check_interval_ms),
                event_bus.clone()
            ).await?
        );
        
        Ok(Self {
            event_bus,
            orchestrator,
            data_capture,
            storage,
            analysis_engine,
            ai_integration,
        })
    }
    
    async fn start(&self) -> Result<()> {
        info!("🚀 Starting all modules...");
        
        // Start modules in dependency order
        self.event_bus.start().await?;
        info!("✅ Event Bus ready");
        
        self.storage.start().await?;
        info!("✅ Storage ready");
        
        self.data_capture.start().await?;
        info!("✅ Data Capture monitoring");
        
        self.analysis_engine.start().await?;
        info!("✅ Analysis Engine ready");
        
        self.ai_integration.start().await?;
        info!("✅ AI Integration ready");
        
        // Start main event processing loop
        let event_bus = self.event_bus.clone();
        let storage = self.storage.clone();
        let analysis_engine = self.analysis_engine.clone();
        let ai_integration = self.ai_integration.clone();
        
        tokio::spawn(async move {
            Self::process_events(event_bus, storage, analysis_engine, ai_integration).await
        });
        
        // Start orchestrator last (it monitors everything)
        self.orchestrator.start().await?;
        info!("✅ Orchestrator monitoring");
        
        info!("🎉 All modules started successfully!");
        Ok(())
    }
    
    async fn process_events(
        event_bus: Arc<EventBus>,
        storage: Arc<Storage>,
        analysis_engine: Arc<AnalysisEngine>,
        ai_integration: Arc<AiIntegration>,
    ) {
        info!("🔄 Event processing loop started");
        
        loop {
            match event_bus.receive().await {
                Ok(event) => {
                    debug!("📨 Processing event: {:?}", event.event_type);
                    
                    match &event {
                        Event::BehaviorCaptured(behavior) => {
                            // Store behavior data
                            if let Err(e) = storage.store_behavior(behavior).await {
                                error!("Failed to store behavior: {}", e);
                            }
                            
                            // Trigger analysis
                            if let Err(e) = analysis_engine.analyze_behavior(behavior).await {
                                error!("Failed to analyze behavior: {}", e);
                            }
                        },
                        
                        Event::StateDetected(state) => {
                            info!("🧠 ADHD State: {:?} (confidence: {:.2})", 
                                  state.state, state.confidence);
                            
                            // Store state
                            if let Err(e) = storage.store_state(state).await {
                                error!("Failed to store state: {}", e);
                            }
                            
                            // Check if intervention needed
                            if state.state == skelly_jelly_analysis_engine::AdhdState::Distracted && 
                               state.confidence > 0.75 {
                                let request = InterventionRequest {
                                    state: state.clone(),
                                    context: "User seems distracted".to_string(),
                                };
                                
                                if let Err(e) = ai_integration.generate_intervention(&request).await {
                                    error!("Failed to generate intervention: {}", e);
                                }
                            }
                        },
                        
                        Event::InterventionGenerated(intervention) => {
                            info!("💬 Intervention: {}", intervention.message);
                            info!("💀 Skeleton: {}", intervention.animation);
                            
                            // Store intervention
                            if let Err(e) = storage.store_intervention(intervention).await {
                                error!("Failed to store intervention: {}", e);
                            }
                            
                            // TODO: Send to TypeScript modules (Gamification, Cute Figurine)
                            // This will be implemented when we set up the TypeScript bridge
                        },
                        
                        _ => {
                            debug!("Unhandled event type");
                        }
                    }
                },
                Err(e) => {
                    error!("Event processing error: {}", e);
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down modules...");
        
        // Shutdown in reverse order
        self.orchestrator.shutdown().await?;
        self.ai_integration.shutdown().await?;
        self.analysis_engine.shutdown().await?;
        self.data_capture.shutdown().await?;
        self.storage.shutdown().await?;
        self.event_bus.shutdown().await?;
        
        info!("✅ Clean shutdown complete");
        Ok(())
    }
}

async fn load_config() -> Result<Config> {
    // Try to load from config file, fallback to defaults
    let config_path = std::env::var("SKELLY_CONFIG")
        .unwrap_or_else(|_| "./config/default.toml".to_string());
    
    match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => {
            info!("📄 Loading config from {}", config_path);
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", config_path))
        },
        Err(_) => {
            warn!("⚠️  Config file not found, using defaults");
            Ok(Config::default())
        }
    }
}