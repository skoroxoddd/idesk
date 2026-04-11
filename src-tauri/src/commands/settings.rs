use crate::state::AppState;
use std::sync::atomic::Ordering;

#[tauri::command]
pub fn set_quality(state: tauri::State<AppState>, quality: u32) {
    state.quality.store(quality.min(2), Ordering::Relaxed);
}

#[tauri::command]
pub fn set_fps(state: tauri::State<AppState>, fps: u32) {
    let fps = fps.clamp(10, 60);
    state.fps.store(fps, Ordering::Relaxed);
}
