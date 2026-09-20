import React, { useState } from "react";
import {
  Settings,
  X,
  Terminal,
  Server,
  Check,
  Bot,
} from "lucide-react";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onInstallCli: () => Promise<void>;
  onUninstallCli: () => Promise<void>;
  onOpenMcp?: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  onInstallCli,
  onUninstallCli,
  onOpenMcp,
}) => {
  const [cliStatusMsg, setCliStatusMsg] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  if (!isOpen) return null;

  const handleInstall = async () => {
    setIsProcessing(true);
    try {
      await onInstallCli();
      setCliStatusMsg("Successfully linked 'devflow' to /usr/local/bin/devflow");
    } catch {
      setCliStatusMsg("Error linking CLI to /usr/local/bin");
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
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content settings-modal-box" onClick={(e) => e.stopPropagation()}>
        {/* Modal Header */}
        <div className="modal-header">
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Settings size={16} color="#06b6d4" />
            <h2 className="modal-title">DevFlow Settings</h2>
          </div>
          <button className="btn-modal-close" onClick={onClose}>
            <X size={16} />
          </button>
        </div>

        {/* Modal Body */}
        <div className="modal-body settings-body">
          {/* CLI Link Section */}
          <div className="settings-section">
            <div className="settings-section-title">
              <Terminal size={14} color="#a855f7" />
              <span>Shell CLI Integration</span>
            </div>
            <p className="settings-desc">
              Link the <code>devflow</code> binary into your system PATH (<code>/usr/local/bin</code>) so you can run <code>devflow doctor</code>, <code>devflow dev</code>, and TUI from any terminal.
            </p>

            {cliStatusMsg && (
              <div className="settings-status-box">
                <Check size={13} color="#10b981" />
                <span>{cliStatusMsg}</span>
              </div>
            )}

            <div style={{ display: "flex", gap: "8px", marginTop: "10px" }}>
              <button
                className="btn-primary-action"
                onClick={handleInstall}
                disabled={isProcessing}
              >
                <Terminal size={12} />
                <span>Install / Re-link CLI</span>
              </button>

              <button
                className="btn-danger-outline"
                onClick={handleUninstall}
                disabled={isProcessing}
              >
                <span>Uninstall CLI Link</span>
              </button>
            </div>
          </div>

          {/* Model Context Protocol (MCP) Section */}
          <div className="settings-section">
            <div className="settings-section-title">
              <Bot size={14} color="#c084fc" />
              <span>Model Context Protocol (MCP) Hub</span>
            </div>
            <p className="settings-desc">
              Expose DevFlow targets, device matrix, run lifecycle, and logs directly to AI coding agents (Claude Desktop, Cursor, Antigravity, VS Code).
            </p>
            {onOpenMcp && (
              <div style={{ marginTop: "10px" }}>
                <button
                  className="btn-primary-action"
                  style={{
                    background: "rgba(168, 85, 247, 0.15)",
                    borderColor: "rgba(168, 85, 247, 0.4)",
                    color: "#c084fc",
                  }}
                  onClick={() => {
                    onClose();
                    onOpenMcp();
                  }}
                >
                  <Bot size={12} />
                  <span>Configure MCP Server & Tools</span>
                </button>
              </div>
            )}
          </div>

          {/* Engine & Runtime Section */}
          <div className="settings-section">
            <div className="settings-section-title">
              <Server size={14} color="#06b6d4" />
              <span>Engine & Local Server</span>
            </div>
            <div className="settings-kv-row">
              <span className="kv-key">Backend Server:</span>
              <span className="kv-val">http://127.0.0.1:9292</span>
            </div>
            <div className="settings-kv-row">
              <span className="kv-key">Terminal Engine:</span>
              <span className="kv-val">@termaxjs/web (Canvas 2D + ANSI Truecolor)</span>
            </div>
            <div className="settings-kv-row">
              <span className="kv-key">Desktop Host Arch:</span>
              <span className="kv-val">aarch64-apple-darwin (macOS Native)</span>
            </div>
          </div>
        </div>

        {/* Modal Footer */}
        <div className="modal-footer">
          <button className="btn-modal-confirm" onClick={onClose}>
            Done
          </button>
        </div>
      </div>
    </div>
  );
};
