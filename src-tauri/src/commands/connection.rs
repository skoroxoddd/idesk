use crate::state::AppState;
use std::sync::Arc;

#[tauri::command]
pub fn get_session_id(state: tauri::State<AppState>) -> String {
    state.session_id.to_string()
}

#[tauri::command]
pub async fn connect_to_peer(
    _state: tauri::State<'_, AppState>,
    _peer_id: String,
) -> Result<(), String> {
    // Phase 4: WebRTC connection establishment
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub fn disconnect(_state: tauri::State<AppState>) {
    // Phase 4: disconnect from peer
}

#[tauri::command]
pub fn check_peer_online(
    _state: tauri::State<AppState>,
    _peer_id: String,
) -> Result<bool, String> {
    // Phase 4: check if peer is online via signaling server
    Ok(false)
}
