pub mod capturer;
pub mod frame;
pub mod mock_capturer;

#[cfg(target_os = "macos")]
pub mod scap_capturer;

#[cfg(target_os = "windows")]
pub mod scap_capturer;

#[cfg(all(target_os = "linux", not(feature = "wayland")))]
pub mod x11_capturer;

#[cfg(all(target_os = "linux", feature = "wayland"))]
pub mod pipewire_capturer;
