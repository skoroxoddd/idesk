import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  onConnect: () => void;
  onConnected: () => void;
  onCancel: () => void;
}

export default function ConnectionScreen({ onConnect, onConnected, onCancel }: Props) {
  const [sessionId, setSessionId] = useState("...");
  const [peerId, setPeerId] = useState("");
  const [connecting, setConnecting] = useState(false);

  useEffect(() => {
    invoke<string>("get_session_id").then(setSessionId).catch(console.error);
  }, []);

  const handleConnect = async () => {
    if (!peerId.trim()) return;
    setConnecting(true);
    onConnect();
    try {
      await invoke("connect_to_peer", { peerId: peerId.trim() });
      onConnected();
    } catch (e) {
      console.error("Connection failed:", e);
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div className="connection-screen">
      <div className="card">
        <h1>Remote Desktop</h1>

        <div className="your-id">
          <p>Your ID:</p>
          <code className="session-id">{sessionId}</code>
          <button className="copy-btn" onClick={() => navigator.clipboard.writeText(sessionId)}>
            Copy
          </button>
        </div>

        <div className="connect-to">
          <label>Connect to:</label>
          <input
            type="text"
            placeholder="XXX-XXX-XXX"
            value={peerId}
            onChange={(e) => setPeerId(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleConnect()}
            maxLength={11}
          />
          <button className="connect-btn" onClick={handleConnect} disabled={connecting || !peerId.trim()}>
            {connecting ? "Connecting..." : "Connect"}
          </button>
        </div>
      </div>
    </div>
  );
}
