import React, { useEffect, useState } from "react";
import {
  Loader2,
  CheckCircle2,
  AlertTriangle,
  X,
  Smartphone,
  AppWindow,
  Package,
  Trash2,
  PowerOff,
  ExternalLink,
  Globe,
  Unplug,
  FolderOpen,
} from "lucide-react";

export interface TargetActionFeedback {
  id: string;
  targetId: string;
  targetName: string;
  framework?: string;
  action: string;
  actionTitle: string;
  status: "pending" | "success" | "error";
  message?: string;
  timestamp: number;
}

interface ActionStatusToastProps {
  feedback: TargetActionFeedback | null;
  onDismiss: () => void;
}

export const ActionStatusToast: React.FC<ActionStatusToastProps> = ({
  feedback,
  onDismiss,
}) => {
  const [isHovered, setIsHovered] = useState(false);

  useEffect(() => {
    if (!feedback) return;
    if (feedback.status === "pending" || isHovered) return;

    const timeout = setTimeout(() => {
      onDismiss();
    }, 4500);

    return () => clearTimeout(timeout);
  }, [feedback, isHovered, onDismiss]);

  if (!feedback) return null;

  const getActionIcon = () => {
    switch (feedback.action) {
      case "launch_app":
        return feedback.framework?.toLowerCase().includes("android") ? (
          <Smartphone size={15} color="#10b981" />
        ) : (
          <AppWindow size={15} color="#f97316" />
        );
      case "reinstall_apk":
        return <Package size={15} color="#10b981" />;
      case "force_stop":
        return <PowerOff size={15} color="#f43f5e" />;
      case "clear_data":
      case "clean_cache":
        return <Trash2 size={15} color="#94a3b8" />;
      case "open_ide":
        return <ExternalLink size={15} color="#38bdf8" />;
      case "open_browser":
        return <Globe size={15} color="#3b82f6" />;
      case "free_port":
        return <Unplug size={15} color="#ef4444" />;
      case "reveal_finder":
        return <FolderOpen size={15} color="#38bdf8" />;
      default:
        return <Package size={15} color="#06b6d4" />;
    }
  };

  const getStatusIcon = () => {
    if (feedback.status === "pending") {
      return <Loader2 size={15} className="animate-spin text-cyan" />;
    }
    if (feedback.status === "success") {
      return <CheckCircle2 size={15} color="#10b981" />;
    }
    return <AlertTriangle size={15} color="#ef4444" />;
  };

  return (
    <div
      className={`action-status-toast ${feedback.status}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div className="action-toast-left">
        <div className="action-toast-icon-wrap">
          {getStatusIcon()}
        </div>
        <div className="action-toast-content">
          <div className="action-toast-title-row">
            {getActionIcon()}
            <span className="action-toast-action-name">
              {feedback.actionTitle}
            </span>
            <span className="action-toast-target-badge font-mono">
              {feedback.targetName}
            </span>
          </div>

          <div className="action-toast-message">
            {feedback.message ||
              (feedback.status === "pending"
                ? `Executing ${feedback.actionTitle}...`
                : feedback.status === "success"
                ? `${feedback.actionTitle} completed successfully`
                : `${feedback.actionTitle} failed`)}
          </div>
        </div>
      </div>

      <div className="action-toast-right">
        <button
          className="btn-toast-close"
          onClick={onDismiss}
          title="Dismiss notification"
        >
          <X size={12} />
        </button>
      </div>
    </div>
  );
};
