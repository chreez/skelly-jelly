//! Skelly-Jelly: Your ADHD Companion
//! 
//! A privacy-first, local-only ADHD assistance system with a melty skeleton companion.
//! This is the main entry point that orchestrates all modules.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import all modules
use skelly_jelly_event_bus::{EventBus, BusConfig, ModuleId};
use skelly_jelly_orchestrator::{Orchestrator, OrchestratorConfig};
use skelly_jelly_data_capture::{DataCapture, DataCaptureConfig};
use skelly_jelly_storage::{Storage, StorageConfig};
use skelly_jelly_analysis_engine::{AnalysisEngine, AnalysisEngineConfig};
use skelly_jelly_gamification::{createGamificationModule, GamificationConfig};
use skelly_jelly_ai_integration::{AIIntegrationImpl, AIIntegrationConfig};
use skelly_jelly_cute_figurine::{CuteFigurine, FigurineConfig};

#[derive(Parser)]
#[command(name = "skelly-jelly")]
#[command(author = "Skelly Team")]
#[command(version = "1.0.0")]
#[command(about = "Your ADHD companion with a melty skeleton friend", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config/skelly-jelly.toml")]
    config: PathBuf,

    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Data directory
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Skelly-Jelly system
    Start {
        /// Start in demo mode with simulated data
        #[arg(long)]
        demo: bool,
    },
    /// Check system health
    Health,
    /// Reset all data and start fresh
    Reset {
        /// Confirm reset without prompt
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.debug)?;

    info!("🦴 Skelly-Jelly starting up! Your melty companion is awakening...");

    match &cli.command {
        Some(Commands::Start { demo }) => {
            start_system(&cli, *demo).await?;
        }
        Some(Commands::Health) => {
            check_health(&cli).await?;
        }
        Some(Commands::Reset { force }) => {
            reset_system(&cli, *force).await?;
        }
        None => {
            // Default to start
            start_system(&cli, false).await?;
        }
    }

    Ok(())
}

async fn start_system(cli: &Cli, demo_mode: bool) -> Result<()> {
    info!("Loading configuration from {:?}", cli.config);
    
    // Load main configuration
    let config = load_configuration(&cli.config)?;
    
    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&cli.data_dir)?;
    
    // Initialize Event Bus
    info!("🚌 Initializing Event Bus...");
    let event_bus = Arc::new(
        EventBus::new(config.event_bus.clone())
            .await
            .context("Failed to create event bus")?
    );
    
    // Initialize Orchestrator
    info!("🎭 Initializing Orchestrator...");
    let orchestrator = Arc::new(
        Orchestrator::new(config.orchestrator.clone(), event_bus.clone())
            .await
            .context("Failed to create orchestrator")?
    );
    
    // Initialize modules in correct order
    info!("📦 Initializing modules...");
    
    // 1. Storage (needed by other modules)
    info!("💾 Starting Storage module...");
    let storage_config = config.storage.clone();
    let storage = Storage::new(storage_config, event_bus.clone())
        .await
        .context("Failed to create storage module")?;
    orchestrator.register_module(ModuleId::Storage, Box::new(storage)).await?;
    
    // 2. Data Capture (generates events)
    info!("📊 Starting Data Capture module...");
    let capture_config = if demo_mode {
        DataCaptureConfig::demo_mode()
    } else {
        config.data_capture.clone()
    };
    let data_capture = DataCapture::new(capture_config, event_bus.clone())
        .await
        .context("Failed to create data capture module")?;
    orchestrator.register_module(ModuleId::DataCapture, Box::new(data_capture)).await?;
    
    // 3. Analysis Engine (processes events)
    info!("🧠 Starting Analysis Engine module...");
    let analysis_config = config.analysis_engine.clone();
    let analysis_engine = AnalysisEngine::new(analysis_config, event_bus.clone())
        .await
        .context("Failed to create analysis engine")?;
    orchestrator.register_module(ModuleId::AnalysisEngine, Box::new(analysis_engine)).await?;
    
    // 4. Gamification (decides on interventions)
    info!("🎮 Starting Gamification module...");
    let gamification = createGamificationModule(
        event_bus.clone(),
        create_logger("gamification"),
        Some(config.gamification.clone()),
        None, // Will use default user profile
    );
    orchestrator.register_module(ModuleId::Gamification, Box::new(gamification)).await?;
    
    // 5. AI Integration (generates responses)
    info!("🤖 Starting AI Integration module...");
    let ai_config = config.ai_integration.clone();
    let mut ai_integration = AIIntegrationImpl::new(ai_config);
    ai_integration.initialize().await
        .context("Failed to initialize AI integration")?;
    orchestrator.register_module(ModuleId::AIIntegration, Box::new(ai_integration)).await?;
    
    // 6. Cute Figurine (UI presentation)
    info!("💀 Starting Cute Figurine module...");
    let figurine_config = config.cute_figurine.clone();
    let cute_figurine = CuteFigurine::new(figurine_config, event_bus.clone())
        .await
        .context("Failed to create cute figurine module")?;
    orchestrator.register_module(ModuleId::CuteFigurine, Box::new(cute_figurine)).await?;
    
    // Start the orchestrator
    info!("🚀 Starting orchestration...");
    orchestrator.start().await?;
    
    info!("✨ Skelly-Jelly is ready! Your skeleton companion is here to help.");
    info!("Press Ctrl+C to stop...");
    
    // Wait for shutdown signal
    wait_for_shutdown().await;
    
    // Graceful shutdown
    info!("🛑 Shutting down Skelly-Jelly...");
    orchestrator.shutdown().await?;
    
    info!("👋 Goodbye! Your skeleton friend will be waiting for you.");
    
    Ok(())
}

async fn check_health(cli: &Cli) -> Result<()> {
    info!("🏥 Checking system health...");
    
    // Load configuration
    let config = load_configuration(&cli.config)?;
    
    // Create temporary event bus
    let event_bus = Arc::new(EventBus::new(config.event_bus.clone()).await?);
    
    // Create orchestrator
    let orchestrator = Orchestrator::new(config.orchestrator.clone(), event_bus.clone()).await?;
    
    // Perform health check
    let health_status = orchestrator.check_health().await?;
    
    // Display results
    println!("\n🏥 Skelly-Jelly Health Status");
    println!("============================");
    
    for (module, status) in health_status.module_status {
        let emoji = match status.as_str() {
            "healthy" => "✅",
            "degraded" => "⚠️",
            "unhealthy" => "❌",
            _ => "❓",
        };
        println!("{} {}: {}", emoji, module, status);
    }
    
    println!("\n📊 System Metrics:");
    println!("  CPU Usage: {:.1}%", health_status.cpu_usage);
    println!("  Memory Usage: {:.1}%", health_status.memory_usage);
    println!("  Uptime: {} seconds", health_status.uptime_seconds);
    
    Ok(())
}

async fn reset_system(cli: &Cli, force: bool) -> Result<()> {
    if !force {
        println!("⚠️  This will delete all Skelly-Jelly data!");
        println!("Are you sure? Type 'yes' to confirm:");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim() != "yes" {
            println!("Reset cancelled.");
            return Ok(());
        }
    }
    
    info!("🧹 Resetting Skelly-Jelly data...");
    
    // Remove data directory
    if cli.data_dir.exists() {
        std::fs::remove_dir_all(&cli.data_dir)?;
        info!("Removed data directory: {:?}", cli.data_dir);
    }
    
    // Recreate empty data directory
    std::fs::create_dir_all(&cli.data_dir)?;
    
    info!("✨ Reset complete! Skelly-Jelly is ready for a fresh start.");
    
    Ok(())
}

fn load_configuration(path: &PathBuf) -> Result<SystemConfig> {
    if !path.exists() {
        info!("Configuration file not found, creating default configuration...");
        create_default_config(path)?;
    }
    
    let contents = std::fs::read_to_string(path)?;
    let config: SystemConfig = toml::from_str(&contents)?;
    
    Ok(config)
}

fn create_default_config(path: &PathBuf) -> Result<()> {
    // Create config directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let default_config = include_str!("../config/default.toml");
    std::fs::write(path, default_config)?;
    
    Ok(())
}

fn init_logging(debug: bool) -> Result<()> {
    let filter = if debug {
        "debug,hyper=info,reqwest=info"
    } else {
        "info,skelly_jelly=debug"
    };
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}

fn create_logger(module_name: &str) -> winston::Logger {
    // Simple logger creation for TypeScript modules
    // In practice, this would integrate with the Rust logging system
    winston::Logger::new(module_name)
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// System-wide configuration
#[derive(Debug, Clone, serde::Deserialize)]
struct SystemConfig {
    event_bus: BusConfig,
    orchestrator: OrchestratorConfig,
    storage: StorageConfig,
    data_capture: DataCaptureConfig,
    analysis_engine: AnalysisEngineConfig,
    gamification: GamificationConfig,
    ai_integration: AIIntegrationConfig,
    cute_figurine: FigurineConfig,
}