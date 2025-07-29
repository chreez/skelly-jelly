//! Skelly-Jelly: Demo with Actual Cute Figurine
//! 
//! This demo actually launches the TypeScript modules and shows the skeleton companion

use anyhow::{Context, Result};
use std::{process::Command, sync::Arc, time::Duration};
use tokio::{signal, time::sleep, process::Child};
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod typescript_bridge;
use typescript_bridge::TypeScriptBridge;

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

    info!("🦴 Skelly-Jelly Demo with Cute Figurine!");
    info!("Your melty skeleton companion is about to appear...");
    
    // Check if TypeScript modules are built
    if !check_typescript_modules_built().await {
        info!("🔨 Building TypeScript modules first...");
        build_typescript_modules().await?;
    }
    
    // Start the demo with actual figurine
    let demo = SkeletonDemo::new().await?;
    demo.run().await?;
    
    Ok(())
}

struct SkeletonDemo {
    figurine_process: Option<Child>,
    gamification_process: Option<Child>,
}

impl SkeletonDemo {
    async fn new() -> Result<Self> {
        Ok(Self {
            figurine_process: None,
            gamification_process: None,
        })
    }
    
    async fn run(mut self) -> Result<()> {
        info!("🚀 Starting skeleton companion demo...");
        
        // Start TypeScript modules
        self.start_typescript_modules().await?;
        
        // Give modules time to initialize
        sleep(Duration::from_secs(2)).await;
        
        // Run the behavioral simulation
        self.simulate_adhd_states().await?;
        
        // Wait for user to stop
        info!("✨ Demo running! Watch your desktop for the skeleton companion.");
        info!("Press Ctrl+C to stop the demo...");
        signal::ctrl_c().await?;
        
        // Clean shutdown
        self.shutdown().await?;
        
        Ok(())
    }
    
    async fn start_typescript_modules(&mut self) -> Result<()> {
        info!("🎮 Starting Gamification module...");
        
        // Start Gamification IPC server
        let gamification = Command::new("node")
            .args(&["dist/ipc_server.js"])
            .current_dir("./modules/gamification")
            .spawn()
            .context("Failed to start Gamification module")?;
        self.gamification_process = Some(gamification);
        
        info!("💀 Starting Cute Figurine module...");
        
        // Start Cute Figurine IPC server (this creates the desktop window)
        let figurine = Command::new("node")
            .args(&["dist/ipc_server.js"])
            .current_dir("./modules/cute-figurine")
            .spawn()
            .context("Failed to start Cute Figurine module")?;
        self.figurine_process = Some(figurine);
        
        info!("✅ TypeScript modules started!");
        Ok(())
    }
    
    async fn simulate_adhd_states(&self) -> Result<()> {
        info!("🧠 Starting ADHD state simulation...");
        sleep(Duration::from_secs(1)).await;
        
        // Phase 1: Flow State
        info!("📊 Phase 1: User in flow state");
        self.send_state_to_figurine("Flow", 0.87, "Steady typing detected, deep focus").await;
        sleep(Duration::from_secs(3)).await;
        
        // Phase 2: Getting Distracted
        info!("📊 Phase 2: Distraction starting");
        self.send_state_to_figurine("Distracted", 0.65, "Window switching detected").await;
        sleep(Duration::from_secs(2)).await;
        
        // Phase 3: Full Distraction
        info!("📊 Phase 3: Fully distracted - intervention time");
        self.send_state_to_figurine("Distracted", 0.82, "Multiple distractions detected").await;
        self.send_intervention("Hey there! 👋 Looks like you might need a gentle nudge back to focus. No worries - happens to the best of us! 🦴✨").await;
        sleep(Duration::from_secs(4)).await;
        
        // Phase 4: Recovery
        info!("📊 Phase 4: Returning to flow");
        self.send_state_to_figurine("Flow", 0.91, "Back to focused work").await;
        sleep(Duration::from_secs(2)).await;
        
        // Phase 5: Achievement
        info!("🏆 Achievement unlocked: Quick Recovery!");
        self.send_achievement("Quick Recovery", "Got back to focus in under 2 minutes!").await;
        sleep(Duration::from_secs(3)).await;
        
        info!("✅ Simulation complete!");
        Ok(())
    }
    
    async fn send_state_to_figurine(&self, state: &str, confidence: f64, context: &str) {
        info!("🧠 ADHD State: {} (confidence: {:.2}) - {}", state, confidence, context);
        
        // In a real system, this would send via the TypeScript bridge
        // For now, we simulate the communication
        
        let animation = match state {
            "Flow" => "happy_focused",
            "Distracted" => "gentle_wave", 
            "Hyperfocus" => "celebration",
            _ => "idle"
        };
        
        info!("💀 → Skeleton: Playing '{}' animation", animation);
    }
    
    async fn send_intervention(&self, message: &str) {
        info!("💬 Intervention: {}", message);
        info!("💀 → Skeleton: Showing message bubble and gentle wave");
    }
    
    async fn send_achievement(&self, title: &str, description: &str) {
        info!("🏆 Achievement: {} - {}", title, description);
        info!("💀 → Skeleton: Celebration animation!");
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        info!("🛑 Shutting down demo...");
        
        // Send goodbye message to skeleton
        info!("💀 → Skeleton: Goodbye wave");
        sleep(Duration::from_millis(500)).await;
        
        // Terminate TypeScript processes
        if let Some(mut process) = self.figurine_process.take() {
            let _ = process.kill().await;
        }
        
        if let Some(mut process) = self.gamification_process.take() {
            let _ = process.kill().await;
        }
        
        info!("👋 See you later! Your skeleton friend will miss you!");
        Ok(())
    }
}

async fn check_typescript_modules_built() -> bool {
    tokio::fs::metadata("./modules/cute-figurine/dist/ipc_server.js").await.is_ok() &&
    tokio::fs::metadata("./modules/gamification/dist/ipc_server.js").await.is_ok()
}

async fn build_typescript_modules() -> Result<()> {
    info!("🔨 Building Gamification module...");
    let output = Command::new("npm")
        .args(&["run", "build"])
        .current_dir("./modules/gamification")
        .output()
        .await
        .context("Failed to build Gamification module")?;
    
    if !output.status.success() {
        error!("Gamification build failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Gamification build failed"));
    }
    
    info!("🔨 Building Cute Figurine module...");
    let output = Command::new("npm")
        .args(&["run", "build"])
        .current_dir("./modules/cute-figurine")
        .output()
        .await
        .context("Failed to build Cute Figurine module")?;
    
    if !output.status.success() {
        error!("Cute Figurine build failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Cute Figurine build failed"));
    }
    
    info!("✅ TypeScript modules built successfully!");
    Ok(())
}