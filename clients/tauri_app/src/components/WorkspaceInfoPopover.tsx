import React, { useState } from "react";
import {
  Copy,
  Check,
  Trash2,
} from "lucide-react";
import type { KnownWorkspace } from "../types";
import { copyToClipboard } from "../utils/clipboard";

interface WorkspaceInfoPopoverProps {
  workspace: KnownWorkspace;
  onClose: () => void;
  onRemove: (path: string) => void;
  anchorRect: DOMRect | null;
}

export const WorkspaceInfoPopover: React.FC<WorkspaceInfoPopoverProps> = ({
  workspace,
  onClose,
  onRemove,
  anchorRect,
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopyPath = async (e: React.MouseEvent) => {
    e.stopPropagation();
    const ok = await copyToClipboard(workspace.path);
    if (ok) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleRemove = (e: React.MouseEvent) => {
    e.stopPropagation();
    onRemove(workspace.path);
    onClose();
  };

  // Compute position based on anchor
  const top = anchorRect ? Math.min(window.innerHeight - 260, Math.max(10, anchorRect.top - 20)) : 100;
  const left = anchorRect ? anchorRect.right + 12 : 230;

  return (
    <>
      {/* Backdrop to dismiss */}
      <div
        className="popover-backdrop"
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
      />

      <div
        className="workspace-info-card"
        style={{
          position: "fixed",
          top: `${top}px`,
          left: `${left}px`,
          zIndex: 9999,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="workspace-info-header">
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <div className="workspace-avatar-small">
              {(workspace.custom_name || workspace.name).slice(0, 2).toUpperCase()}
            </div>
            <div>
              <div className="workspace-info-title">{workspace.custom_name || workspace.name}</div>
              <div className="workspace-info-framework">
                <span className={`platform-tag ${(workspace.platform || "generic").toLowerCase()}`}>
                  {workspace.platform || "Generic"}
                </span>
                <span>{workspace.framework || "Workspace"}</span>
              </div>
            </div>
          </div>
        </div>

        <div className="workspace-info-body">
          <div className="workspace-info-field">
            <span className="field-label">Filesystem Path</span>
            <div className="field-path-box">
              <span className="field-path-text" title={workspace.path}>
                {workspace.path}
              </span>
              <button
                className="btn-path-copy"
                onClick={handleCopyPath}
                title="Copy Path to Clipboard"
              >
                {copied ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
              </button>
            </div>
          </div>

          <div className="workspace-info-field" style={{ marginTop: "8px" }}>
            <span className="field-label">Last Accessed</span>
            <span className="field-value">
              {workspace.last_opened ? new Date(workspace.last_opened).toLocaleString() : "Recently"}
            </span>
          </div>
        </div>

        <div className="workspace-info-footer">
          <button
            className="btn-info-action remove"
            onClick={handleRemove}
            title="Remove from recent workspaces list"
          >
            <Trash2 size={12} />
            <span>Remove</span>
          </button>

          <button
            className="btn-info-action copy"
            onClick={handleCopyPath}
            title="Copy Path"
          >
            <Copy size={12} />
            <span>{copied ? "Copied" : "Copy Path"}</span>
          </button>
        </div>
      </div>
    </>
  );
};
