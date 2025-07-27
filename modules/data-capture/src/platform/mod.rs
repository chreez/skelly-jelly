//! Platform-specific implementations for event monitoring

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

/// Platform capability detection
pub struct PlatformCapabilities {
    pub keystroke_monitoring: bool,
    pub mouse_monitoring: bool,
    pub window_monitoring: bool,
    pub screenshot_capture: bool,
    pub process_monitoring: bool,
    pub resource_monitoring: bool,
    pub permission_required: bool,
}

impl PlatformCapabilities {
    /// Detect current platform capabilities
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                keystroke_monitoring: true,
                mouse_monitoring: true,
                window_monitoring: true,
                screenshot_capture: true,
                process_monitoring: true,
                resource_monitoring: true,
                permission_required: true, // Accessibility permissions required
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            Self {
                keystroke_monitoring: true,
                mouse_monitoring: true,
                window_monitoring: true,
                screenshot_capture: true,
                process_monitoring: true,
                resource_monitoring: true,
                permission_required: false, // Usually no special permissions needed
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            Self {
                keystroke_monitoring: true, // Depends on X11/Wayland
                mouse_monitoring: true,
                window_monitoring: true,
                screenshot_capture: true,
                process_monitoring: true,
                resource_monitoring: true,
                permission_required: false, // Depends on session type
            }
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Self {
                keystroke_monitoring: false,
                mouse_monitoring: false,
                window_monitoring: false,
                screenshot_capture: false,
                process_monitoring: false,
                resource_monitoring: false,
                permission_required: false,
            }
        }
    }
}

/// Permission management utilities
pub mod permissions {
    use crate::{DataCaptureError, Result};
    
    /// Check if required permissions are available
    pub async fn check_permissions() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            super::macos::permissions::check_accessibility_permission().await
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows typically doesn't require special permissions
            Ok(())
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux permission checking depends on the environment
            Ok(())
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(DataCaptureError::NotSupported)
        }
    }
    
    /// Request required permissions from the user
    pub async fn request_permissions() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            super::macos::permissions::request_accessibility_permission().await
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }
}