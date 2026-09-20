import React, { useState, useMemo } from "react";
import {
  Smartphone,
  Laptop,
  Play,
  RotateCcw,
  Copy,
  Check,
  Sparkles,
  Search,
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
  const [searchQuery, setSearchQuery] = useState("");
  const [platformFilter, setPlatformFilter] = useState<"all" | "android" | "apple" | "desktop" | "emulators">("all");
  const [bootingName, setBootingName] = useState<string | null>(null);

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

  const handleBoot = async (name: string) => {
    const trimmed = name.trim();
    if (trimmed) {
      setBootingName(trimmed);
      try {
        await onBootEmulator(trimmed);
      } finally {
        setTimeout(() => setBootingName(null), 3000);
      }
    }
  };

  // Metrics computation
  const metrics = useMemo(() => {
    const total = devices.length;
    const online = devices.filter(
      (d) => d.online || d.state === "Connected" || d.state === "connected" || d.state === "Booted" || d.state === "device"
    ).length;
    const androidCount = devices.filter((d) => d.platform.toLowerCase() === "android").length;
    const appleCount = devices.filter((d) => {
      const p = d.platform.toLowerCase();
      return p === "ios" || p === "macos" || p === "apple";
    }).length;
    const emulators = devices.filter((d) => d.is_emulator).length;

    return { total, online, androidCount, appleCount, emulators };
  }, [devices]);

  // Filtered devices
  const filteredDevices = useMemo(() => {
    return devices.filter((d) => {
      const plat = d.platform.toLowerCase();
      const isAndroid = plat === "android";
      const isApple = plat === "ios" || plat === "macos" || plat === "apple";
      const isDesktop = plat === "desktop" || plat === "macos" || plat === "linux" || plat === "windows";

      if (platformFilter === "android" && !isAndroid) return false;
      if (platformFilter === "apple" && !isApple) return false;
      if (platformFilter === "desktop" && !isDesktop) return false;
      if (platformFilter === "emulators" && !d.is_emulator) return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesName = d.name.toLowerCase().includes(q);
        const matchesId = d.id.toLowerCase().includes(q);
        const matchesPlat = d.platform.toLowerCase().includes(q);
        const matchesArch = (d.target_arch || "").toLowerCase().includes(q);
        const matchesOs = (d.os_version || "").toLowerCase().includes(q);
        return matchesName || matchesId || matchesPlat || matchesArch || matchesOs;
      }
      return true;
    });
  }, [devices, platformFilter, searchQuery]);

  // Common preset AVDs and Simulators
  const presetEmulators = [
    { name: "Pixel_8_API_35", display: "Pixel 8 Pro (API 35)", platform: "Android" },
    { name: "Pixel_7_API_34", display: "Pixel 7 (API 34)", platform: "Android" },
    { name: "Medium_Phone_API_35", display: "Medium Phone (API 35)", platform: "Android" },
    { name: "iPhone 15 Pro", display: "iPhone 15 Pro (iOS 17.5)", platform: "iOS" },
  ];

  return (
    <div className="view-container">
      {/* 1. Header Banner */}
      <div className="view-header">
        <div>
          <div className="view-title-group">
            <Smartphone size={22} color="#38bdf8" />
            <h1 className="view-title">Devices & Emulators Hub</h1>
            <span className="view-badge badge-cyan">{metrics.online} Online</span>
          </div>
          <p className="view-subtitle">
            Physical mobile devices, desktop local runtimes, and Android / iOS virtual machine targets.
          </p>
        </div>

        <div className="view-actions">
          <button
            className="btn-glass"
            onClick={handleRefresh}
            disabled={isRefreshing}
            title="Scan for connected USB/Wireless devices"
          >
            <RotateCcw size={14} className={isRefreshing ? "animate-spin" : ""} />
            <span>Rescan Devices</span>
          </button>
        </div>
      </div>

      {/* 2. Metric Scoreboard Cards */}
      <div className="devices-metrics-strip">
        <div className="device-metric-card">
          <span className="metric-label">TOTAL DETECTED</span>
          <div className="metric-number">{metrics.total}</div>
        </div>

        <div className="device-metric-card highlight-green">
          <span className="metric-label">ONLINE & READY</span>
          <div className="metric-number font-emerald">{metrics.online}</div>
        </div>

        <div className="device-metric-card">
          <span className="metric-label">ANDROID TARGETS</span>
          <div className="metric-number">{metrics.androidCount}</div>
        </div>

        <div className="device-metric-card">
          <span className="metric-label">APPLE / MACOS / IOS</span>
          <div className="metric-number">{metrics.appleCount}</div>
        </div>

        <div className="device-metric-card">
          <span className="metric-label">VIRTUAL EMULATORS</span>
          <div className="metric-number">{metrics.emulators}</div>
        </div>
      </div>

      {/* 3. Search & Filter Bar */}
      <div className="devices-toolbar">
        <div className="search-input-wrapper">
          <Search size={14} color="#64748b" />
          <input
            type="text"
            placeholder="Search devices by name, UDID, serial, platform, or OS version..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="glass-input"
          />
        </div>

        <div className="device-filter-chips">
          <button
            className={`filter-chip ${platformFilter === "all" ? "active" : ""}`}
            onClick={() => setPlatformFilter("all")}
          >
            ALL ({devices.length})
          </button>
          <button
            className={`filter-chip ${platformFilter === "android" ? "active" : ""}`}
            onClick={() => setPlatformFilter("android")}
          >
            Android ({metrics.androidCount})
          </button>
          <button
            className={`filter-chip ${platformFilter === "apple" ? "active" : ""}`}
            onClick={() => setPlatformFilter("apple")}
          >
            Apple / iOS ({metrics.appleCount})
          </button>
          <button
            className={`filter-chip ${platformFilter === "desktop" ? "active" : ""}`}
            onClick={() => setPlatformFilter("desktop")}
          >
            Desktop Host
          </button>
          <button
            className={`filter-chip ${platformFilter === "emulators" ? "active" : ""}`}
            onClick={() => setPlatformFilter("emulators")}
          >
            Emulators ({metrics.emulators})
          </button>
        </div>
      </div>

      {/* 4. Grid of Detected Devices */}
      {filteredDevices.length === 0 ? (
        <div className="empty-card-container">
          <Smartphone size={32} color="#64748b" />
          <div className="empty-title">
            {searchQuery ? "No Devices Match Your Filter" : "No Connected Devices Found"}
          </div>
          <p className="empty-desc">
            Connect an Android phone via USB/Wi-Fi with USB Debugging enabled, or launch an iOS Simulator / Android AVD below.
          </p>
          <button className="btn-primary" onClick={handleRefresh}>
            <RotateCcw size={14} />
            <span>Rescan Device Bus</span>
          </button>
        </div>
      ) : (
        <div className="device-cards-grid">
          {filteredDevices.map((d) => {
            const isOnline =
              d.state === "Connected" ||
              d.state === "connected" ||
              d.state === "Booted" ||
              d.state === "device" ||
              d.online;
            const isDesktop =
              d.platform.toLowerCase() === "desktop" ||
              d.platform.toLowerCase() === "macos" ||
              d.platform.toLowerCase() === "linux";
            const isAndroid = d.platform.toLowerCase() === "android";

            return (
              <div key={d.id} className={`rich-device-card ${isOnline ? "online-glow" : "offline"}`}>
                <div className="rich-card-top">
                  <div className="card-icon-wrap">
                    {isDesktop ? (
                      <Laptop size={22} color="#38bdf8" />
                    ) : isAndroid ? (
                      <Smartphone size={22} color="#10b981" />
                    ) : (
                      <Smartphone size={22} color="#c084fc" />
                    )}
                  </div>

                  <div className="card-header-info">
                    <div className="card-title-row">
                      <span className="card-title truncate" title={d.name}>
                        {d.name}
                      </span>
                      {d.is_default && <span className="default-pill">DEFAULT TARGET</span>}
                    </div>
                    <div className="card-tags-row">
                      <span className={`platform-tag ${d.platform.toLowerCase()}`}>
                        {d.platform.toUpperCase()}
                      </span>
                      {d.is_emulator && <span className="emulator-tag">VIRTUAL</span>}
                    </div>
                  </div>

                  <div className={`status-pill ${isOnline ? "online" : "offline"}`}>
                    <span className={`pulse-dot ${isOnline ? "green" : "gray"}`} />
                    <span>{isOnline ? "Online" : d.state || "Offline"}</span>
                  </div>
                </div>

                <div className="rich-card-body">
                  <div className="card-meta-row">
                    <span className="meta-label">Architecture</span>
                    <span className="meta-value font-mono">{d.target_arch || "Universal / aarch64"}</span>
                  </div>

                  <div className="card-meta-row">
                    <span className="meta-label">OS API Level</span>
                    <span className="meta-value">{d.os_version || "Host Native OS"}</span>
                  </div>

                  <div className="card-meta-row">
                    <span className="meta-label">Target ID / Serial</span>
                    <div
                      className="copyable-id"
                      onClick={() => handleCopy(d.id)}
                      title="Click to copy Target ID"
                    >
                      <span className="font-mono truncate">{d.id}</span>
                      {copiedId === d.id ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
                    </div>
                  </div>
                </div>

                <div className="rich-card-footer">
                  <span className="footer-status-text">
                    {isOnline ? "✓ Ready for deployment & log stream" : "Device unreachable"}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* 5. Android / iOS Virtual Device Launchpad */}
      <div className="section-title" style={{ marginTop: "36px" }}>
        <Sparkles size={16} color="#f59e0b" />
        <span>Virtual Device & Simulator Launchpad</span>
      </div>

      <div className="emulator-launcher-card">
        <div className="launcher-left">
          <div className="launcher-title">Quick 1-Click Launch Presets</div>
          <p className="launcher-desc">
            Instantly spin up virtual devices using your local Android SDK `emulator` or Xcode `xcrun simctl` toolchains.
          </p>

          <div className="preset-avds-row">
            {presetEmulators.map((avd) => {
              const isBooting = bootingName === avd.name;
              return (
                <button
                  key={avd.name}
                  className="preset-avd-btn"
                  onClick={() => handleBoot(avd.name)}
                  disabled={isBooting}
                  title={`Launch ${avd.name}`}
                >
                  <Play size={12} fill="currentColor" color={isBooting ? "#94a3b8" : "#10b981"} />
                  <span>{isBooting ? `Booting ${avd.name}...` : avd.display}</span>
                </button>
              );
            })}
          </div>
        </div>

        <div className="launcher-right">
          <div className="launcher-title">Boot Custom Emulator by Name</div>
          <p className="launcher-desc">Enter any registered AVD name or iOS Simulator UUID:</p>
          <div className="custom-boot-input-group">
            <input
              type="text"
              placeholder="e.g. Pixel_8_API_35 or iPhone 15 Pro"
              value={avdInput}
              onChange={(e) => setAvdInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleBoot(avdInput)}
              className="glass-input"
            />
            <button
              className="btn-primary"
              onClick={() => {
                handleBoot(avdInput);
                setAvdInput("");
              }}
              disabled={!avdInput.trim() || bootingName === avdInput.trim()}
            >
              <Play size={13} fill="currentColor" />
              <span>{bootingName === avdInput.trim() ? "Booting..." : "Launch"}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
