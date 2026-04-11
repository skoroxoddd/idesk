import { useState, useEffect } from "react";
import ConnectionScreen from "./components/ConnectionScreen";
import RemoteDesktopView from "./components/RemoteDesktopView";

type AppMode = "idle" | "connecting" | "connected";

function App() {
  const [mode, setMode] = useState<AppMode>("idle");

  return (
    <div className="app">
      {mode === "connected" ? (
        <RemoteDesktopView onDisconnect={() => setMode("idle")} />
      ) : (
        <ConnectionScreen
          onConnect={() => setMode("connecting")}
          onConnected={() => setMode("connected")}
          onCancel={() => setMode("idle")}
        />
      )}
    </div>
  );
}

export default App;
