//! TypeScript Bridge Module
//! 
//! Handles communication between Rust modules and TypeScript modules
//! (Gamification and Cute Figurine) via IPC mechanisms.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

use skelly_jelly_event_bus::{Event, EventBus};

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeScriptMessage {
    pub module: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub focus_coins: u32,
    pub current_streak: u32,
    pub interventions_today: u32,
    pub achievements_unlocked: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkeletonAnimation {
    pub animation_type: String,
    pub duration_ms: u32,
    pub message: Option<String>,
    pub emotion: String,
}

pub struct TypeScriptBridge {
    event_bus: Arc<EventBus>,
    gamification_sender: Option<mpsc::UnboundedSender<TypeScriptMessage>>,
    figurine_sender: Option<mpsc::UnboundedSender<TypeScriptMessage>>,
}

impl TypeScriptBridge {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            gamification_sender: None,
            figurine_sender: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("🌉 Starting TypeScript bridge...");

        // Start Gamification module
        self.start_gamification_module().await?;
        
        // Start Cute Figurine module  
        self.start_figurine_module().await?;

        // Start event forwarding
        self.start_event_forwarding().await?;

        info!("✅ TypeScript bridge ready");
        Ok(())
    }

    async fn start_gamification_module(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.gamification_sender = Some(tx);

        let gamification_path = "./modules/gamification";
        let mut child = Command::new("npm")
            .args(&["run", "start:ipc"])
            .current_dir(gamification_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()) 
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start Gamification module")?;

        info!("🎮 Gamification module started (PID: {})", child.id());

        // Handle messages to gamification
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&message) {
                    debug!("→ Gamification: {}", json);
                    // Send via stdin to the TypeScript process
                    // Implementation depends on the IPC protocol
                }
            }
        });

        Ok(())
    }

    async fn start_figurine_module(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.figurine_sender = Some(tx);

        let figurine_path = "./modules/cute-figurine";
        let mut child = Command::new("npm")
            .args(&["run", "start:ipc"])
            .current_dir(figurine_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start Cute Figurine module")?;

        info!("💀 Cute Figurine module started (PID: {})", child.id());

        // Handle messages to figurine
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&message) {
                    debug!("→ Figurine: {}", json);
                    // Send via stdin to the TypeScript process
                }
            }
        });

        Ok(())
    }

    async fn start_event_forwarding(&self) -> Result<()> {
        let event_bus = self.event_bus.clone();
        let gamification_sender = self.gamification_sender.clone();
        let figurine_sender = self.figurine_sender.clone();

        tokio::spawn(async move {
            loop {
                match event_bus.receive().await {
                    Ok(event) => {
                        Self::forward_event_to_typescript(
                            &event,
                            &gamification_sender,
                            &figurine_sender
                        ).await;
                    },
                    Err(e) => {
                        error!("TypeScript bridge event error: {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        Ok(())
    }

    async fn forward_event_to_typescript(
        event: &Event,
        gamification_sender: &Option<mpsc::UnboundedSender<TypeScriptMessage>>,
        figurine_sender: &Option<mpsc::UnboundedSender<TypeScriptMessage>>,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match event {
            Event::StateDetected(state) => {
                // Send to Gamification for scoring
                if let Some(sender) = gamification_sender {
                    let message = TypeScriptMessage {
                        module: "gamification".to_string(),
                        action: "state_detected".to_string(),
                        payload: serde_json::to_value(state).unwrap_or_default(),
                        timestamp,
                    };
                    let _ = sender.send(message);
                }

                // Send to Figurine for animation
                if let Some(sender) = figurine_sender {
                    let animation = match state.state {
                        skelly_jelly_analysis_engine::AdhdState::Flow => "happy_focused",
                        skelly_jelly_analysis_engine::AdhdState::Distracted => "gentle_wave",
                        skelly_jelly_analysis_engine::AdhdState::Hyperfocus => "celebration",
                    };

                    let message = TypeScriptMessage {
                        module: "cute-figurine".to_string(),
                        action: "play_animation".to_string(),
                        payload: serde_json::json!({
                            "animation": animation,
                            "state": state
                        }),
                        timestamp,
                    };
                    let _ = sender.send(message);
                }
            },

            Event::InterventionGenerated(intervention) => {
                // Send to Figurine for display
                if let Some(sender) = figurine_sender {
                    let message = TypeScriptMessage {
                        module: "cute-figurine".to_string(),
                        action: "show_intervention".to_string(),
                        payload: serde_json::to_value(intervention).unwrap_or_default(),
                        timestamp,
                    };
                    let _ = sender.send(message);
                }
            },

            _ => {
                // Other events can be forwarded as needed
                debug!("Event not forwarded to TypeScript: {:?}", event);
            }
        }
    }

    pub async fn send_to_gamification(&self, action: &str, payload: serde_json::Value) -> Result<()> {
        if let Some(sender) = &self.gamification_sender {
            let message = TypeScriptMessage {
                module: "gamification".to_string(),
                action: action.to_string(),
                payload,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            sender.send(message).context("Failed to send to Gamification")?;
        }
        Ok(())
    }

    pub async fn send_to_figurine(&self, action: &str, payload: serde_json::Value) -> Result<()> {
        if let Some(sender) = &self.figurine_sender {
            let message = TypeScriptMessage {
                module: "cute-figurine".to_string(),
                action: action.to_string(),
                payload,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            sender.send(message).context("Failed to send to Figurine")?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down TypeScript bridge...");
        
        // Send shutdown signals to TypeScript modules
        if let Err(e) = self.send_to_gamification("shutdown", serde_json::json!({})).await {
            warn!("Failed to shutdown Gamification gracefully: {}", e);
        }

        if let Err(e) = self.send_to_figurine("shutdown", serde_json::json!({})).await {
            warn!("Failed to shutdown Figurine gracefully: {}", e);
        }

        // Give modules time to clean up
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        info!("✅ TypeScript bridge shutdown complete");
        Ok(())
    }
}