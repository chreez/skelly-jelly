//! Basic orchestration example
//! 
//! This example demonstrates how to use the orchestrator to manage
//! the lifecycle of system modules.

use orchestrator::{
    create_orchestrator, OrchestratorConfig, ModuleDescriptor,
    SystemStatus, ModuleState,
};
use skelly_jelly_event_bus::{create_event_bus, ModuleId};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting orchestrator example");

    // Create event bus
    let event_bus = create_event_bus()?;
    info!("Event bus created");

    // Create orchestrator configuration
    let config = OrchestratorConfig {
        startup_timeout: Duration::from_secs(30),
        module_start_delay: Duration::from_millis(500),
        parallel_startup: false,
        health_check_interval: Duration::from_secs(10),
        health_check_timeout: Duration::from_secs(3),
        unhealthy_threshold: 2,
        auto_recovery: true,
        max_recovery_attempts: 3,
        recovery_backoff: Duration::from_secs(5),
        resource_check_interval: Duration::from_secs(5),
        throttle_threshold: 0.8,
    };

    // Create orchestrator
    let orchestrator = create_orchestrator(config, event_bus).await?;
    info!("Orchestrator created");

    // Register additional modules (optional)
    let custom_module = ModuleDescriptor::new(
        ModuleId::CuteFigurine,
        "cute-figurine".to_string()
    )
    .with_dependencies(vec![ModuleId::EventBus, ModuleId::AiIntegration])
    .with_required(false);

    orchestrator.register_module(custom_module).await?;
    info!("Custom module registered");

    // Start the system
    info!("Starting system...");
    orchestrator.start_system().await?;
    info!("System started successfully");

    // Monitor system for a while
    for i in 1..=5 {
        sleep(Duration::from_secs(2)).await;
        
        let health = orchestrator.get_system_health().await;
        info!("Health check {}: Status = {:?}, Uptime = {:?}", 
              i, health.status, health.uptime);

        // Print module states
        for (module_id, health_report) in &health.module_health {
            info!("  Module {}: {:?}", module_id, health_report.status);
        }

        // Print resource usage
        info!("  System resources: CPU={:.1}%, Memory={}MB", 
              health.resource_usage.total_cpu_usage,
              health.resource_usage.total_memory_mb);

        // Show any active issues
        if !health.active_issues.is_empty() {
            info!("  Active issues: {}", health.active_issues.len());
            for issue in &health.active_issues {
                info!("    - {:?}: {}", issue.severity, issue.description);
            }
        }
    }

    // Test module restart
    info!("Testing module restart...");
    orchestrator.restart_module(ModuleId::DataCapture).await?;
    sleep(Duration::from_secs(2)).await;

    let state = orchestrator.get_module_state(ModuleId::DataCapture).await;
    info!("Data capture state after restart: {:?}", state);

    // Test configuration update
    info!("Testing configuration update...");
    let new_config = serde_json::json!({
        "capture_interval": 1000,
        "enable_screenshots": true,
        "privacy_mode": false
    });
    
    orchestrator.update_config(ModuleId::DataCapture, new_config).await?;
    info!("Configuration updated");

    // Final health check
    let final_health = orchestrator.get_system_health().await;
    info!("Final system status: {:?}", final_health.status);

    match final_health.status {
        SystemStatus::Healthy => {
            info!("✅ System is healthy!");
        }
        SystemStatus::Degraded { reason } => {
            info!("⚠️  System is degraded: {}", reason);
        }
        SystemStatus::Critical { failing_modules } => {
            info!("❌ System is critical - failing modules: {:?}", failing_modules);
        }
        _ => {
            info!("ℹ️  System status: {:?}", final_health.status);
        }
    }

    // Stop the system
    info!("Stopping system...");
    orchestrator.stop_system(Duration::from_secs(15)).await?;
    info!("System stopped gracefully");

    Ok(())
}

/// Demonstrate module state monitoring
async fn monitor_module_states(orchestrator: &dyn orchestrator::OrchestratorTrait) {
    let modules = [
        ModuleId::EventBus,
        ModuleId::Storage,
        ModuleId::DataCapture,
        ModuleId::AnalysisEngine,
        ModuleId::Gamification,
        ModuleId::AiIntegration,
        ModuleId::CuteFigurine,
    ];

    for module_id in modules {
        if let Some(state) = orchestrator.get_module_state(module_id).await {
            match state {
                ModuleState::NotStarted => {
                    info!("{}: Not started", module_id);
                }
                ModuleState::Starting { since } => {
                    info!("{}: Starting (since {:?})", module_id, since.elapsed());
                }
                ModuleState::Running { since } => {
                    info!("{}: Running (uptime {:?})", module_id, since.elapsed());
                }
                ModuleState::Stopping { since } => {
                    info!("{}: Stopping (since {:?})", module_id, since.elapsed());
                }
                ModuleState::Stopped { reason } => {
                    info!("{}: Stopped ({:?})", module_id, reason);
                }
                ModuleState::Failed { error, attempts } => {
                    info!("{}: Failed - {} (attempts: {})", module_id, error, attempts);
                }
            }
        } else {
            info!("{}: Unknown state", module_id);
        }
    }
}

/// Demonstrate error injection and recovery
async fn test_error_recovery(orchestrator: &dyn orchestrator::OrchestratorTrait) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing error recovery...");

    // Simulate a module failure by restarting it
    info!("Simulating module failure...");
    orchestrator.restart_module(ModuleId::AnalysisEngine).await?;

    // Wait for recovery
    sleep(Duration::from_secs(3)).await;

    // Check if module recovered
    if let Some(state) = orchestrator.get_module_state(ModuleId::AnalysisEngine).await {
        match state {
            ModuleState::Running { .. } => {
                info!("✅ Module recovered successfully");
            }
            ModuleState::Failed { error, attempts } => {
                info!("❌ Module recovery failed: {} (attempts: {})", error, attempts);
            }
            _ => {
                info!("ℹ️  Module in state: {:?}", state);
            }
        }
    }

    Ok(())
}