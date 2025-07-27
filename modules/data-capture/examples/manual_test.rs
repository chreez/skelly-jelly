//! Manual testing application for the data-capture module
//! 
//! This example demonstrates how to:
//! 1. Initialize the data capture module
//! 2. Configure different monitors
//! 3. Test privacy filtering
//! 4. Monitor events in real-time
//! 5. Test configuration changes

use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

use skelly_jelly_data_capture::{
    DataCaptureModule, DataCaptureConfig, MonitorConfig, PrivacyConfig, PerformanceConfig,
    EventBus
};
use skelly_jelly_data_capture::config::{
    KeystrokeConfig, MouseConfig, WindowConfig, ScreenshotConfig, ProcessConfig, ResourceConfig,
    PrivacyMode, PrivacyZone
};
use skelly_jelly_storage::types::BusMessage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("debug".parse()?))
        .init();

    info!("🚀 Starting Data Capture Module Manual Test");
    
    // Test 1: Basic Module Creation
    println!("\n=== Test 1: Module Creation ===");
    let config = create_test_config();
    let event_bus = std::sync::Arc::new(EventBus);
    
    let mut module = DataCaptureModule::new(config.clone(), event_bus.clone()).await?;
    info!("✅ Module created successfully");

    // Test 2: Module Lifecycle
    println!("\n=== Test 2: Module Lifecycle ===");
    test_module_lifecycle(&mut module).await?;

    // Test 3: Configuration Testing
    println!("\n=== Test 3: Configuration Updates ===");
    test_configuration_updates(&mut module).await?;

    // Test 4: Privacy Testing
    println!("\n=== Test 4: Privacy Filtering ===");
    test_privacy_features().await?;

    // Test 5: Performance Testing
    println!("\n=== Test 5: Performance Monitoring ===");
    test_performance_monitoring(&mut module).await?;

    // Test 6: Interactive Testing
    println!("\n=== Test 6: Interactive Testing ===");
    interactive_test(&mut module).await?;

    info!("🎉 All manual tests completed successfully!");
    Ok(())
}

fn create_test_config() -> DataCaptureConfig {
    DataCaptureConfig {
        monitors: MonitorConfig {
            keystroke: KeystrokeConfig {
                enabled: true,
                buffer_size: 100,
                coalescence_ms: 50,
                capture_modifiers: true,
                capture_special_keys: true,
            },
            mouse: MouseConfig {
                enabled: true,
                buffer_size: 200,
                movement_threshold: 10.0,
                click_coalescence_ms: 100,
                capture_movement: true,
                capture_clicks: true,
                capture_scroll: true,
            },
            window: WindowConfig {
                enabled: true,
                capture_title: true,
                capture_app_name: true,
                switch_threshold_ms: 200,
            },
            screenshot: ScreenshotConfig {
                enabled: false, // Disabled for safety in testing
                capture_interval_ms: 60000,
                max_size_mb: 2,
                compression_quality: 75,
                capture_on_significant_change: true,
                change_threshold: 0.2,
                privacy_mode: PrivacyMode::Balanced,
            },
            process: ProcessConfig {
                enabled: true,
                sample_interval_ms: 5000,
                capture_command_line: false,
                capture_environment: false,
            },
            resource: ResourceConfig {
                enabled: true,
                sample_interval_ms: 2000,
                capture_cpu: true,
                capture_memory: true,
                capture_disk: false,
                capture_network: false,
            },
        },
        privacy: PrivacyConfig {
            pii_detection: true,
            sensitive_app_list: vec![
                "1Password".to_string(),
                "Terminal".to_string(),
                "SSH".to_string(),
            ],
            ignored_app_list: vec![
                "Activity Monitor".to_string(),
                "Console".to_string(),
            ],
            mask_passwords: true,
            mask_credit_cards: true,
            mask_ssn: true,
            mask_emails: true,
            screenshot_privacy_zones: vec![],
        },
        performance: PerformanceConfig {
            max_cpu_percent: 2.0, // Allow higher for testing
            max_memory_mb: 100,
            event_buffer_size: 1000,
            event_batch_size: 50,
            backpressure_threshold: 0.8,
            drop_on_overflow: true,
        },
    }
}

async fn test_module_lifecycle(module: &mut DataCaptureModule) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing module start...");
    module.start().await?;
    info!("✅ Module started successfully");

    // Let it run for a short time to capture some events
    info!("Running for 5 seconds to capture events...");
    sleep(Duration::from_secs(5)).await;

    // Check stats
    let stats = module.stats();
    info!("📊 Module Statistics:");
    info!("  Events captured: {}", stats.events_captured);
    info!("  Events dropped: {}", stats.events_dropped);
    info!("  Active monitors: {}", stats.active_monitors);
    info!("  CPU usage: {:.2}%", stats.cpu_usage);
    info!("  Memory usage: {} bytes", stats.memory_usage);

    info!("Testing module stop...");
    module.stop().await?;
    info!("✅ Module stopped successfully");

    Ok(())
}

async fn test_configuration_updates(module: &mut DataCaptureModule) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing configuration updates...");
    
    // Create new config with different settings
    let mut new_config = create_test_config();
    new_config.monitors.keystroke.enabled = false;
    new_config.monitors.mouse.buffer_size = 500;
    // Update privacy settings for strict mode
    new_config.privacy.mask_emails = true;
    new_config.privacy.mask_passwords = true;

    module.update_config(new_config).await?;
    info!("✅ Configuration updated successfully");

    // Test the new configuration
    module.start().await?;
    sleep(Duration::from_secs(2)).await;
    
    let stats = module.stats();
    info!("📊 Updated configuration stats:");
    info!("  Active monitors: {}", stats.active_monitors);
    
    module.stop().await?;
    Ok(())
}

async fn test_privacy_features() -> Result<(), Box<dyn std::error::Error>> {
    use skelly_jelly_data_capture::privacy::{PrivacyFilter, detect_privacy_mode};
    
    info!("Testing privacy features...");
    
    let config = PrivacyConfig {
        pii_detection: true,
        mask_emails: true,
        mask_ssn: true,
        mask_credit_cards: true,
        mask_passwords: true,
        sensitive_app_list: vec!["1Password".to_string()],
        ignored_app_list: vec!["Calculator".to_string()],
        screenshot_privacy_zones: vec![],
    };
    
    let filter = PrivacyFilter::new(config);
    
    // Test PII detection
    let test_texts = vec![
        ("Normal text", "This is just normal text"),
        ("Email", "Contact me at john.doe@example.com"),
        ("SSN", "My SSN is 123-45-6789"),
        ("Credit Card", "Card number: 4532 1234 5678 9012"),
        ("Password", "Password: **********"),
    ];
    
    for (label, text) in test_texts {
        let filtered = filter.filter_text(text);
        info!("🔒 {}: '{}' → '{}'", label, text, filtered);
    }
    
    // Test app monitoring decisions
    let test_apps = vec![
        ("1Password", "Sensitive app"),
        ("Calculator", "Ignored app"),
        ("Safari", "Normal app"),
        ("Terminal", "Normal app"),
    ];
    
    for (app, _description) in test_apps {
        let should_monitor = filter.should_monitor_app(app);
        let is_sensitive = filter.is_sensitive_window("Login", app);
        info!("📱 {}: Monitor={}, Sensitive={}", app, should_monitor, is_sensitive);
    }
    
    // Test privacy mode detection
    let mode1 = detect_privacy_mode("1Password", "Login");
    let mode2 = detect_privacy_mode("Safari", "Normal browsing");
    let mode3 = detect_privacy_mode("Terminal", "Password prompt");
    
    info!("🛡️ Privacy modes:");
    info!("  1Password Login: {:?}", mode1);
    info!("  Safari Normal: {:?}", mode2);
    info!("  Terminal Password: {:?}", mode3);
    
    info!("✅ Privacy features tested successfully");
    Ok(())
}

async fn test_performance_monitoring(module: &mut DataCaptureModule) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing performance monitoring...");
    
    module.start().await?;
    
    // Monitor performance over time
    for i in 1..=10 {
        sleep(Duration::from_millis(500)).await;
        let stats = module.stats();
        
        if i % 2 == 0 { // Print every second
            info!("⏱️  Iteration {}: CPU={:.2}%, Memory={}KB, Events={}", 
                i/2, stats.cpu_usage, stats.memory_usage / 1024, stats.events_captured);
        }
        
        // Check for performance issues
        if stats.cpu_usage > 5.0 {
            warn!("⚠️  High CPU usage detected: {:.2}%", stats.cpu_usage);
        }
        
        if stats.memory_usage > 100 * 1024 * 1024 { // 100MB
            warn!("⚠️  High memory usage detected: {}MB", stats.memory_usage / 1024 / 1024);
        }
        
        if stats.events_dropped > 0 {
            warn!("⚠️  Events being dropped: {}", stats.events_dropped);
        }
    }
    
    module.stop().await?;
    info!("✅ Performance monitoring completed");
    Ok(())
}

async fn interactive_test(module: &mut DataCaptureModule) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎮 Interactive Testing Mode");
    println!("The module will now run and capture events from your interactions.");
    println!("Try the following activities to test different monitors:");
    println!("  - Type on the keyboard (keystroke monitor)");
    println!("  - Move and click the mouse (mouse monitor)");
    println!("  - Switch between applications (window monitor)");
    println!("  - Open system applications (process monitor)");
    println!("\nPress Enter to start, then Ctrl+C to stop...");
    
    // Wait for user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    info!("🏁 Starting interactive capture...");
    module.start().await?;
    
    // Run for a configurable time with status updates
    let test_duration = Duration::from_secs(30);
    let mut elapsed = Duration::from_secs(0);
    let update_interval = Duration::from_secs(5);
    
    while elapsed < test_duration {
        let sleep_result = timeout(update_interval, sleep(update_interval)).await;
        
        if sleep_result.is_ok() {
            elapsed += update_interval;
            let stats = module.stats();
            println!("📊 Status ({}s): {} events captured, {} monitors active", 
                elapsed.as_secs(), stats.events_captured, stats.active_monitors);
        } else {
            break; // Timeout occurred, continue
        }
    }
    
    module.stop().await?;
    
    let final_stats = module.stats();
    println!("\n🏆 Interactive test completed!");
    println!("Final Statistics:");
    println!("  Total events captured: {}", final_stats.events_captured);
    println!("  Events dropped: {}", final_stats.events_dropped);
    println!("  Average CPU usage: {:.2}%", final_stats.cpu_usage);
    println!("  Peak memory usage: {}KB", final_stats.memory_usage / 1024);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_manual_test_config() {
        let config = create_test_config();
        assert!(config.monitors.keystroke.enabled);
        assert!(config.privacy.pii_detection);
        assert_eq!(config.performance.max_cpu_percent, 2.0);
    }
    
    #[tokio::test]
    async fn test_module_creation() {
        let config = create_test_config();
        let event_bus = std::sync::Arc::new(EventBus);
        
        let result = DataCaptureModule::new(config, event_bus).await;
        assert!(result.is_ok());
    }
}