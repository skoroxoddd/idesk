use crate::capture::capturer::Capturer;
use crate::error::Result;

/// Create a platform-appropriate capturer
pub fn create_capturer() -> Result<Box<dyn Capturer>> {
    #[cfg(target_os = "linux")]
    {
        #[cfg(feature = "wayland")]
        {
            crate::capture::pipewire_capturer::PipeWireCapturer::new()
                .map(|c| Box::new(c) as Box<dyn Capturer>)
        }
        #[cfg(not(feature = "wayland"))]
        {
            crate::capture::x11_capturer::X11Capturer::new()
                .map(|c| Box::new(c) as Box<dyn Capturer>)
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        crate::capture::scap_capturer::ScapCapturer::new()
            .map(|c| Box::new(c) as Box<dyn Capturer>)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(crate::error::AppError::Capture("Unsupported platform".to_string()))
    }
}
