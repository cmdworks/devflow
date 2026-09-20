import React, { useState } from "react";
import {
  Settings,
  Terminal,
  Server,
  Bot,
  CheckCircle2,
  Sparkles,
  RefreshCw,
  Download,
  ShieldCheck,
  Loader2,
} from "lucide-react";
import type { UpdateCheckResponse } from "../types";

interface SettingsViewProps {
  onInstallCli: () => Promise<void>;
  onUninstallCli: () => Promise<void>;
  onNavigateToMcp: () => void;
  workspacePath: string;
  updateInfo: UpdateCheckResponse | null;
  isCheckingUpdate: boolean;
  onCheckUpdates: () => Promise<void>;
  onOpenUpdateModal: () => void;
}

export const SettingsView: React.FC<SettingsViewProps> = ({
  onInstallCli,
  onUninstallCli,
  onNavigateToMcp,
  workspacePath,
  updateInfo,
  isCheckingUpdate,
  onCheckUpdates,
  onOpenUpdateModal,
}) => {
  const [cliStatusMsg, setCliStatusMsg] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  const handleInstall = async () => {
    setIsProcessing(true);
    try {
      await onInstallCli();
      setCliStatusMsg("Successfully linked 'devflow' to /usr/local/bin/devflow");
    } catch {
      setCliStatusMsg("Error linking CLI to /usr/local/bin. Ensure you have proper permissions.");
    } finally {
      setIsProcessing(false);
    }
  };

  const handleUninstall = async () => {
    setIsProcessing(true);
    try {
      await onUninstallCli();
      setCliStatusMsg("Uninstalled 'devflow' from /usr/local/bin");
    } catch {
      setCliStatusMsg("Error removing CLI from /usr/local/bin");
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="view-container">
      {/* Header Banner */}
      <div className="view-header">
        <div>
          <div className="view-title-group">
            <Settings size={22} color="#94a3b8" />
            <h1 className="view-title">Settings & System Environment</h1>
            <span className="view-badge badge-slate">Preferences</span>
          </div>
          <p className="view-subtitle">
            Configure system terminal PATH integration, inspect the local runtime engine, and manage updates.
          </p>
        </div>
      </div>

      <div className="settings-cards-grid">
        {/* Card 0: Version & Auto-Updates */}
        <div className="settings-card-large">
          <div className="settings-card-header">
            <div className="settings-card-title-group">
              <Sparkles size={18} color="#38bdf8" />
              <div>
                <h3 className="settings-card-title">Version & Software Updates</h3>
                <p className="settings-card-subtitle">
                  Direct in-app release updates powered by GitHub Releases.
                </p>
              </div>
            </div>
            {updateInfo?.update_available ? (
              <span className="badge-update-alert">
                ✨ Update v{updateInfo.latest_version}
              </span>
            ) : (
              <span className="badge-update-ok">
                <ShieldCheck size={12} /> Up to Date
              </span>
            )}
          </div>

          <div className="settings-card-body">
            <div className="settings-meta-table">
              <div className="settings-meta-row">
                <span className="meta-label">Installed Version</span>
                <span className="meta-val font-mono">
                  v{updateInfo?.current_version || "0.1.0"}
                </span>
              </div>
              <div className="settings-meta-row">
                <span className="meta-label">Latest Available</span>
                <span className="meta-val font-mono">
                  v{updateInfo?.latest_version || updateInfo?.current_version || "0.1.0"}
                  {updateInfo?.update_available && (
                    <span style={{ color: "#38bdf8", marginLeft: "8px", fontWeight: 600 }}>
                      (New Version Ready)
                    </span>
                  )}
                </span>
              </div>
              <div className="settings-meta-row">
                <span className="meta-label">Update Channel</span>
                <span className="meta-val font-mono">GitHub Releases (cmdworks/devflow)</span>
              </div>
              {updateInfo?.target_platform && (
                <div className="settings-meta-row">
                  <span className="meta-label">Platform Architecture</span>
                  <span className="meta-val font-mono">{updateInfo.target_platform}</span>
                </div>
              )}
            </div>

            <div className="settings-btn-row">
              <button
                className="btn-glass"
                onClick={onCheckUpdates}
                disabled={isCheckingUpdate}
                title="Check GitHub Releases for new updates"
              >
                {isCheckingUpdate ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  <RefreshCw size={14} />
                )}
                <span>{isCheckingUpdate ? "Checking Releases..." : "Check for Updates"}</span>
              </button>

              {updateInfo?.update_available && (
                <button
                  className="btn-primary"
                  onClick={onOpenUpdateModal}
                  style={{
                    background: "linear-gradient(135deg, #0284c7, #38bdf8)",
                    boxShadow: "0 0 15px rgba(56, 189, 248, 0.3)",
                  }}
                >
                  <Download size={14} />
                  <span>View Update & Install</span>
                </button>
              )}
            </div>
          </div>
        </div>

        {/* Card 1: Shell CLI PATH Integration */}
        <div className="settings-card-large">
          <div className="settings-card-header">
            <div className="settings-card-title-group">
              <Terminal size={18} color="#a855f7" />
              <div>
                <h3 className="settings-card-title">Shell CLI PATH Integration</h3>
                <p className="settings-card-subtitle">
                  Expose <code>devflow</code> globally across your macOS / Linux terminal environments.
                </p>
              </div>
            </div>
          </div>

          <div className="settings-card-body">
            <p className="settings-desc-text">
              Linking <code>devflow</code> into <code>/usr/local/bin/devflow</code> allows you to run commands like{" "}
              <code>devflow doctor</code>, <code>devflow dev</code>, <code>devflow mcp connect all</code>, and TUI from any terminal emulator without specifying binary paths.
            </p>

            {cliStatusMsg && (
              <div className="settings-alert-box">
                <CheckCircle2 size={15} color="#10b981" />
                <span>{cliStatusMsg}</span>
              </div>
            )}

            <div className="settings-btn-row">
              <button
                className="btn-primary"
                onClick={handleInstall}
                disabled={isProcessing}
              >
                <Terminal size={14} />
                <span>Install / Re-link Shell CLI</span>
              </button>

              <button
                className="btn-danger-outline"
                onClick={handleUninstall}
                disabled={isProcessing}
              >
                <span>Remove CLI Symlink</span>
              </button>
            </div>
          </div>
        </div>

        {/* Card 2: MCP Hub Quick Navigation */}
        <div className="settings-card-large">
          <div className="settings-card-header">
            <div className="settings-card-title-group">
              <Bot size={18} color="#c084fc" />
              <div>
                <h3 className="settings-card-title">Model Context Protocol (MCP) Hub</h3>
                <p className="settings-card-subtitle">
                  AI coding agent integration and real-time activity tracing.
                </p>
              </div>
            </div>
          </div>

          <div className="settings-card-body">
            <p className="settings-desc-text">
              DevFlow exposes an integrated 11-tool Model Context Protocol server for Claude Desktop, Cursor, Antigravity, and VS Code with live request duration tracing and parameter inspection.
            </p>

            <div className="settings-btn-row">
              <button
                className="btn-glass"
                style={{
                  background: "rgba(168, 85, 247, 0.12)",
                  borderColor: "rgba(168, 85, 247, 0.35)",
                  color: "#c084fc",
                }}
                onClick={onNavigateToMcp}
              >
                <Bot size={14} />
                <span>Open MCP Workspace Tab</span>
              </button>
            </div>
          </div>
        </div>

        {/* Card 3: Engine & Runtime Architecture */}
        <div className="settings-card-large">
          <div className="settings-card-header">
            <div className="settings-card-title-group">
              <Server size={18} color="#06b6d4" />
              <div>
                <h3 className="settings-card-title">Engine & Runtime Specs</h3>
                <p className="settings-card-subtitle">
                  Diagnostics for the native companion host backend.
                </p>
              </div>
            </div>
          </div>

          <div className="settings-card-body">
            <div className="settings-meta-table">
              <div className="settings-meta-row">
                <span className="meta-label">Local Companion Port</span>
                <span className="meta-val font-mono">http://127.0.0.1:9292 (Auto-fallback to next free)</span>
              </div>
              <div className="settings-meta-row">
                <span className="meta-label">Terminal Rendering Engine</span>
                <span className="meta-val font-mono">@termaxjs/web (Canvas 2D + WebGL Truecolor)</span>
              </div>
              <div className="settings-meta-row">
                <span className="meta-label">Host Architecture</span>
                <span className="meta-val font-mono">aarch64-apple-darwin (macOS Native)</span>
              </div>
              <div className="settings-meta-row">
                <span className="meta-label">Active Workspace</span>
                <span className="meta-val font-mono truncate" title={workspacePath}>
                  {workspacePath || "No workspace loaded"}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
