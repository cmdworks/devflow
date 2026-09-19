import React, { useState } from "react";
import {
  Smartphone,
  Laptop,
  Play,
  RotateCcw,
  Copy,
  Check,
  Layers,
  Sparkles,
} from "lucide-react";
import type { Device } from "../types";

interface DevicesViewProps {
  devices: Device[];
  onRefreshDevices: () => void;
  onBootEmulator: (name: string) => void;
}

export const DevicesView: React.FC<DevicesViewProps> = ({
  devices,
  onRefreshDevices,
  onBootEmulator,
}) => {
  const [avdInput, setAvdInput] = useState("");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await onRefreshDevices();
    } finally {
      setTimeout(() => setIsRefreshing(false), 500);
    }
  };

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(text);
    setTimeout(() => setCopiedId(null), 1500);
  };

  const handleBoot = () => {
    const trimmed = avdInput.trim();
    if (trimmed) {
      onBootEmulator(trimmed);
      setAvdInput("");
    }
  };

  // Pre-configured common AVD names for quick 1-click launch
  const presetAvds = [
    { name: "Pixel_7_API_34", display: "Pixel 7 (API 34)" },
    { name: "Pixel_6_API_33", display: "Pixel 6 (API 33)" },
    { name: "Medium_Phone_API_35", display: "Medium Phone (API 35)" },
  ];

  return (
    <div className="view-container">
      {/* Header Banner */}
      <div className="view-header">
        <div>
          <div className="view-title-group">
            <Smartphone size={22} color="#38bdf8" />
            <h1 className="view-title">Devices & Emulators Hub</h1>
            <span className="view-badge">{devices.length} Detected</span>
          </div>
          <p className="view-subtitle">
            Manage physical mobile devices, desktop runtime targets, and boot Android / iOS virtual machines.
          </p>
        </div>

        <div className="view-actions">
          <button
            className="btn-glass"
            onClick={handleRefresh}
            title="Scan for connected USB/Wireless devices"
          >
            <RotateCcw size={14} className={isRefreshing ? "animate-spin" : ""} />
            <span>Refresh Scan</span>
          </button>
        </div>
      </div>

      {/* Grid of Detected Devices */}
      <div className="section-title">
        <Layers size={15} color="#06b6d4" />
        <span>Connected Devices & Targets ({devices.length})</span>
      </div>

      {devices.length === 0 ? (
        <div className="empty-card-container">
          <Smartphone size={32} color="#64748b" />
          <div className="empty-title">No Connected Devices Found</div>
          <p className="empty-desc">
            Connect an Android phone via USB/Wi-Fi with USB Debugging enabled, or launch an iOS Simulator / Android AVD below.
          </p>
          <button className="btn-primary" onClick={handleRefresh}>
            <RotateCcw size={14} />
            <span>Rescan Devices</span>
          </button>
        </div>
      ) : (
        <div className="device-cards-grid">
          {devices.map((d) => {
            const isOnline =
              d.state === "Connected" ||
              d.state === "connected" ||
              d.state === "Booted" ||
              d.online;
            const isDesktop =
              d.platform.toLowerCase() === "desktop" ||
              d.platform.toLowerCase() === "macos";
            const isAndroid = d.platform.toLowerCase() === "android";

            return (
              <div key={d.id} className={`rich-card ${isOnline ? "online-border" : ""}`}>
                <div className="rich-card-top">
                  <div className="card-icon-wrap">
                    {isDesktop ? (
                      <Laptop size={20} color="#38bdf8" />
                    ) : (
                      <Smartphone size={20} color={isAndroid ? "#10b981" : "#a855f7"} />
                    )}
                  </div>

                  <div className="card-header-info">
                    <div className="card-title-row">
                      <span className="card-title">{d.name}</span>
                      {d.is_default && <span className="default-pill">DEFAULT</span>}
                    </div>
                    <span className="card-platform-tag">{d.platform.toUpperCase()}</span>
                  </div>

                  <div className="card-status-pill">
                    <span className={`pulse-dot ${isOnline ? "green" : "gray"}`} />
                    <span style={{ fontSize: "11px", fontWeight: 600, color: isOnline ? "#34d399" : "#94a3b8" }}>
                      {isOnline ? "Online" : d.state || "Offline"}
                    </span>
                  </div>
                </div>

                <div className="rich-card-body">
                  <div className="card-meta-row">
                    <span className="meta-label">Architecture</span>
                    <span className="meta-value font-mono">{d.target_arch || "Universal / aarch64"}</span>
                  </div>

                  <div className="card-meta-row">
                    <span className="meta-label">OS Version</span>
                    <span className="meta-value">{d.os_version || "Native Host"}</span>
                  </div>

                  <div className="card-meta-row">
                    <span className="meta-label">Device Type</span>
                    <span className="meta-value">
                      {d.is_emulator ? "Virtual Emulator (AVD)" : isDesktop ? "Desktop Host" : "Physical Device"}
                    </span>
                  </div>

                  <div className="card-meta-row">
                    <span className="meta-label">Target ID</span>
                    <div className="copyable-id" onClick={() => handleCopy(d.id)} title="Click to copy ID">
                      <span className="font-mono text-truncate">{d.id}</span>
                      {copiedId === d.id ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
                    </div>
                  </div>
                </div>

                <div className="rich-card-footer">
                  <span className="footer-hint">
                    {isOnline ? "Ready for build deployment & live log streaming" : "Device not responsive"}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Android Emulator Launchpad Section */}
      <div className="section-title" style={{ marginTop: "32px" }}>
        <Sparkles size={15} color="#f59e0b" />
        <span>Android Virtual Device (AVD) Launchpad</span>
      </div>

      <div className="emulator-launcher-card">
        <div className="launcher-left">
          <div className="launcher-title">Quick Boot AVD Presets</div>
          <p className="launcher-desc">
            Instantly spin up pre-configured Android Virtual Devices using your local Android SDK `emulator` toolchain.
          </p>

          <div className="preset-avds-row">
            {presetAvds.map((avd) => (
              <button
                key={avd.name}
                className="preset-avd-btn"
                onClick={() => onBootEmulator(avd.name)}
                title={`Launch ${avd.name}`}
              >
                <Play size={12} fill="currentColor" color="#10b981" />
                <span>{avd.display}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="launcher-right">
          <div className="launcher-title">Boot Custom AVD by Name</div>
          <div className="custom-boot-input-group">
            <input
              type="text"
              placeholder="e.g. Pixel_7_Pro_API_34"
              value={avdInput}
              onChange={(e) => setAvdInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleBoot()}
              className="glass-input"
            />
            <button className="btn-primary" onClick={handleBoot} disabled={!avdInput.trim()}>
              <Play size={13} fill="currentColor" />
              <span>Boot</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
