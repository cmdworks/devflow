import React, { useState } from "react";
import {
  Sparkles,
  Download,
  RotateCw,
  CheckCircle2,
  AlertCircle,
  ExternalLink,
  X,
  ArrowUpCircle,
  Loader2,
  ShieldCheck,
} from "lucide-react";
import type { UpdateCheckResponse } from "../types";

interface UpdateModalProps {
  isOpen: boolean;
  onClose: () => void;
  updateInfo: UpdateCheckResponse | null;
  onInstallUpdate: (downloadUrl: string) => Promise<{ success: boolean; message?: string; error?: string }>;
  onRestartApp: () => Promise<void>;
}

export const UpdateModal: React.FC<UpdateModalProps> = ({
  isOpen,
  onClose,
  updateInfo,
  onInstallUpdate,
  onRestartApp,
}) => {
  const [installState, setInstallState] = useState<"idle" | "installing" | "installed" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isRestarting, setIsRestarting] = useState<boolean>(false);

  if (!isOpen || !updateInfo) return null;

  const {
    current_version,
    latest_version,
    update_available,
    release_name,
    release_notes,
    published_at,
    download_url,
    html_url,
    target_platform,
  } = updateInfo;

  const formattedDate = published_at
    ? new Date(published_at).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      })
    : "";

  const handleInstall = async () => {
    if (!download_url) {
      setErrorMessage("No direct binary package found for platform " + target_platform);
      setInstallState("error");
      return;
    }

    setInstallState("installing");
    setErrorMessage(null);

    try {
      const res = await onInstallUpdate(download_url);
      if (res.success) {
        setInstallState("installed");
      } else {
        setInstallState("error");
        setErrorMessage(res.error || res.message || "Failed to download or install update");
      }
    } catch (err: unknown) {
      setInstallState("error");
      setErrorMessage(err instanceof Error ? err.message : "An unexpected error occurred during update");
    }
  };

  const handleRestart = async () => {
    setIsRestarting(true);
    try {
      await onRestartApp();
    } catch {
      setIsRestarting(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-content update-modal-box"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="modal-header update-modal-header">
          <div className="update-modal-title-group">
            <div className="update-badge-icon">
              <Sparkles size={16} color="#38bdf8" />
            </div>
            <div>
              <h2 className="modal-title">
                {update_available ? "Software Update Available" : "DevFlow Up to Date"}
              </h2>
              <span className="update-modal-subtitle">
                {update_available
                  ? `New release ${release_name || `v${latest_version}`} is ready`
                  : `You are running the latest version of DevFlow`}
              </span>
            </div>
          </div>
          <button className="btn-modal-close" onClick={onClose} title="Close">
            <X size={16} />
          </button>
        </div>

        {/* Body */}
        <div className="modal-body update-modal-body">
          {/* Version comparison card */}
          <div className="update-version-card">
            <div className="update-version-row">
              <div className="update-version-item">
                <span className="version-label">Current Version</span>
                <span className="version-value current font-mono">v{current_version}</span>
              </div>
              <div className="update-version-arrow">
                <ArrowUpCircle size={18} color="#38bdf8" />
              </div>
              <div className="update-version-item">
                <span className="version-label">Latest Version</span>
                <span className="version-value latest font-mono">v{latest_version}</span>
              </div>
            </div>

            <div className="update-meta-row">
              <span className="update-meta-pill">
                <ShieldCheck size={12} color="#10b981" />
                Verified GitHub Release
              </span>
              <span className="update-meta-pill font-mono">
                {target_platform}
              </span>
              {formattedDate && (
                <span className="update-meta-pill">
                  {formattedDate}
                </span>
              )}
            </div>
          </div>

          {/* Release Notes */}
          <div className="update-notes-container">
            <span className="update-notes-heading">Release Notes & Changelog</span>
            <div className="update-notes-content">
              {release_notes ? (
                <pre className="update-changelog-text">{release_notes}</pre>
              ) : (
                <p className="update-empty-notes">
                  No detailed release notes provided for this release. Visit GitHub Releases for full commit history.
                </p>
              )}
            </div>
          </div>

          {/* Error Banner */}
          {installState === "error" && (
            <div className="update-alert-banner error">
              <AlertCircle size={16} color="#ef4444" />
              <div className="update-alert-text">
                <strong>Update Error:</strong> {errorMessage || "Failed to download update."}
              </div>
            </div>
          )}

          {/* Installed Success Banner */}
          {installState === "installed" && (
            <div className="update-alert-banner success">
              <CheckCircle2 size={16} color="#10b981" />
              <div className="update-alert-text">
                <strong>Update Ready!</strong> DevFlow has been updated. Click <b>Restart DevFlow</b> to launch the new version.
              </div>
            </div>
          )}

          {/* Installing Progress Animation */}
          {installState === "installing" && (
            <div className="update-installing-status">
              <Loader2 size={16} className="animate-spin text-sky-400" />
              <span>Downloading & Extracting release package ({target_platform})...</span>
              <div className="update-progress-indeterminate">
                <div className="progress-bar-slider" />
              </div>
            </div>
          )}
        </div>

        {/* Footer Actions */}
        <div className="modal-footer update-modal-footer">
          <a
            href={html_url || "https://github.com/cmdworks/devflow/releases"}
            target="_blank"
            rel="noopener noreferrer"
            className="btn-glass btn-update-gh"
          >
            <ExternalLink size={13} />
            <span>GitHub Release</span>
          </a>

          <div className="update-actions-right">
            {installState === "installed" ? (
              <button
                className="btn-primary btn-update-action"
                onClick={handleRestart}
                disabled={isRestarting}
              >
                {isRestarting ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  <RotateCw size={14} />
                )}
                <span>{isRestarting ? "Restarting..." : "Restart DevFlow"}</span>
              </button>
            ) : update_available ? (
              <button
                className="btn-primary btn-update-action"
                onClick={handleInstall}
                disabled={installState === "installing" || !download_url}
              >
                {installState === "installing" ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  <Download size={14} />
                )}
                <span>
                  {installState === "installing"
                    ? "Installing Update..."
                    : download_url
                    ? "Download & Install Update"
                    : "No Package Available"}
                </span>
              </button>
            ) : (
              <button className="btn-secondary" onClick={onClose}>
                <span>Close</span>
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
