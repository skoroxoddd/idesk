import { useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  onDisconnect: () => void;
}

export default function RemoteDesktopView({ onDisconnect }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);

  return (
    <div className="remote-desktop">
      <div className="toolbar">
        <span className="status">Connected</span>
        <button onClick={onDisconnect}>Disconnect</button>
      </div>
      <div className="video-container">
        <video ref={videoRef} autoPlay playsInline muted />
      </div>
    </div>
  );
}
