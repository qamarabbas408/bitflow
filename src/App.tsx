import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface NetworkSpeed {
  interface: string;
  rx_formatted: string;
  tx_formatted: string;
  rx_bytes: number;
  tx_bytes: number;
}

// Sparkline history buffer
const HISTORY_LEN = 40;

function useHistory(value: number) {
  const ref = useRef<number[]>(Array(HISTORY_LEN).fill(0));
  ref.current = [...ref.current.slice(1), value];
  return ref.current;
}

function Sparkline({ values, color }: { values: number[]; color: string }) {
  const max = Math.max(...values, 1);
  const w = 120, h = 32;
  const pts = values
    .map((v, i) => `${(i / (HISTORY_LEN - 1)) * w},${h - (v / max) * h}`)
    .join(" ");
  return (
    <svg width={w} height={h} style={{ display: "block" }}>
      <polyline
        points={pts}
        fill="none"
        stroke={color}
        strokeWidth="1.5"
        strokeLinejoin="round"
        strokeLinecap="round"
        opacity="0.8"
      />
    </svg>
  );
}

function SpeedBar({ bytes, max, color }: { bytes: number; max: number; color: string }) {
  const pct = max > 0 ? Math.min((bytes / max) * 100, 100) : 0;
  return (
    <div className="speed-bar-track">
      <div
        className="speed-bar-fill"
        style={{ width: `${pct}%`, background: color }}
      />
    </div>
  );
}

function NetworkCard({ net, globalMax }: { net: NetworkSpeed; globalMax: number }) {
  const rxHistory = useHistory(net.rx_bytes);
  const txHistory = useHistory(net.tx_bytes);

  return (
    <div className="card">
      <div className="card-header">
        <span className="iface-name">{net.interface}</span>
        <span className="live-dot" />
      </div>

      <div className="metrics">
        {/* Download */}
        <div className="metric-row">
          <div className="metric-label">
            <span className="arrow down">▼</span>
            <span className="metric-tag">RX</span>
          </div>
          <div className="metric-body">
            <SpeedBar bytes={net.rx_bytes} max={globalMax} color="#39ff8f" />
            <div className="metric-footer">
              <Sparkline values={rxHistory} color="#39ff8f" />
              <span className="speed-value" style={{ color: "#39ff8f" }}>
                {net.rx_formatted}
              </span>
            </div>
          </div>
        </div>

        {/* Upload */}
        <div className="metric-row">
          <div className="metric-label">
            <span className="arrow up">▲</span>
            <span className="metric-tag">TX</span>
          </div>
          <div className="metric-body">
            <SpeedBar bytes={net.tx_bytes} max={globalMax} color="#38bdf8" />
            <div className="metric-footer">
              <Sparkline values={txHistory} color="#38bdf8" />
              <span className="speed-value" style={{ color: "#38bdf8" }}>
                {net.tx_formatted}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function App() {
  const [networks, setNetworks] = useState<NetworkSpeed[]>([]);
  const [tick, setTick] = useState(0);

  useEffect(() => {
    const unlisten = listen<NetworkSpeed[]>("network-speed", (event) => {
      setNetworks(event.payload);
      setTick((t) => t + 1);
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  const globalMax = Math.max(
    ...networks.map((n) => Math.max(n.rx_bytes, n.tx_bytes)),
    1
  );

  return (
    <main className="app">
      <header className="app-header">
        <div className="logo-block">
          <svg width="22" height="22" viewBox="0 0 22 22" fill="none">
            <rect x="1" y="1" width="20" height="20" rx="3" stroke="#39ff8f" strokeWidth="1.5"/>
            <path d="M5 14 L9 8 L13 12 L17 6" stroke="#38bdf8" strokeWidth="1.5" strokeLinejoin="round"/>
          </svg>
          <span className="logo-text">BITFLOW</span>
        </div>
        <div className="header-meta">
          <span className="tick-counter">TICK #{String(tick).padStart(6, "0")}</span>
          <span className={`status-chip ${networks.length > 0 ? "active" : "idle"}`}>
            {networks.length > 0 ? "● LIVE" : "○ IDLE"}
          </span>
        </div>
      </header>

      <div className="grid">
        {networks.length === 0 ? (
          <div className="empty-state">
            <div className="spinner" />
            <p>Awaiting network data…</p>
          </div>
        ) : (
          networks.map((net) => (
            <NetworkCard key={net.interface} net={net} globalMax={globalMax} />
          ))
        )}
      </div>
    </main>
  );
}

export default App;