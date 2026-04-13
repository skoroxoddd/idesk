use crate::state::AppState;
use remote_desktop_core::capture::factory::create_capturer;
use remote_desktop_core::encode::openh264_encoder::OpenH264Encoder;
use remote_desktop_core::input::enigo_injector::EnigoInjector;
use std::sync::atomic::Ordering;
use tracing::{info, error};

#[tauri::command]
pub fn get_session_id(state: tauri::State<AppState>) -> String {
    state.session_id.to_string()
}

#[tauri::command]
pub async fn connect_to_peer(
    state: tauri::State<'_, AppState>,
    peer_id: String,
) -> Result<String, String> {
    info!("Connecting to peer: {}", peer_id);

    let mut cm = state.connection_manager.lock().await;

    // Set up capturer, encoder, injector if not already done
    if cm.capturer.is_none() {
        let capturer = create_capturer().map_err(|e| e.to_string())?;
        cm.set_capturer(capturer);
    }

    if cm.encoder.is_none() {
        let w = 1280;
        let h = 720;
        let encoder = OpenH264Encoder::new(w, h, 2_000_000).map_err(|e| e.to_string())?;
        cm.set_encoder(Box::new(encoder));
    }

    if cm.injector.is_none() {
        let injector = EnigoInjector::new().map_err(|e| e.to_string())?;
        cm.set_injector(Box::new(injector));
    }

    cm.set_fps(state.fps.load(Ordering::Relaxed));

    // Connect to signaling
    let session_id = state.session_id.to_string();
    cm.connect_signaling(session_id).await.map_err(|e| e.to_string())?;

    // Start streaming (controlled side)
    cm.start_streaming(peer_id.clone()).await.map_err(|e| e.to_string())?;

    state.connected.store(true, Ordering::Relaxed);
    info!("Connected to peer: {}", peer_id);

    Ok("Connected".to_string())
}

#[tauri::command]
pub async fn disconnect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut cm = state.connection_manager.lock().await;
    cm.disconnect().await;
    state.connected.store(false, Ordering::Relaxed);
    info!("Disconnected");
    Ok(())
}

#[tauri::command]
pub async fn check_peer_online(
    state: tauri::State<'_, AppState>,
    peer_id: String,
) -> Result<bool, String> {
    // For now, we can't check without a signaling connection
    // In Phase 7 this will query the signaling server
    let _ = state;
    let _ = peer_id;
    Ok(false)
}
