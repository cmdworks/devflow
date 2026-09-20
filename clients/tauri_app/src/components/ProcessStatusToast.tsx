import React from "react";
import { Loader2, CheckCircle2, X, Square, Play, Zap, RotateCw } from "lucide-react";

export interface BatchProgressInfo {
  active: boolean;
  type: "start" | "stop" | "reload" | "restart";
  step: number;
  total: number;
  targetName?: string;
  completed?: boolean;
  message?: string;
}

interface ProcessStatusToastProps {
  progress: BatchProgressInfo | null;
  onDismiss: () => void;
}

export const ProcessStatusToast: React.FC<ProcessStatusToastProps> = ({
  progress,
  onDismiss,
}) => {
  if (!progress || !progress.active) return null;

  const isDone = !!progress.completed;
  const percent =
    progress.total > 0
      ? Math.min(100, Math.round((progress.step / progress.total) * 100))
      : 100;

  const getIcon = () => {
    if (isDone) {
      return <CheckCircle2 size={15} color="#10b981" />;
    }
    switch (progress.type) {
      case "stop":
        return <Square size={14} color="#f87171" fill="currentColor" />;
      case "start":
        return <Play size={14} color="#38bdf8" fill="currentColor" />;
      case "reload":
        return <Zap size={14} color="#f59e0b" />;
      case "restart":
        return <RotateCw size={14} color="#06b6d4" className="animate-spin" />;
    }
  };

  const getTitle = () => {
    if (isDone) {
      switch (progress.type) {
        case "stop":
          return "All processes stopped cleanly";
        case "start":
          return "All targets started successfully";
        case "reload":
          return "Hot reload completed";
        case "restart":
          return "All targets restarted cleanly";
      }
    }

    switch (progress.type) {
      case "stop":
        return `Stopping processes (${progress.step}/${progress.total})`;
      case "start":
        return `Building & starting targets (${progress.step}/${progress.total})`;
      case "reload":
        return `Hot reloading active targets (${progress.step}/${progress.total})`;
      case "restart":
        return `Restarting targets (${progress.step}/${progress.total})`;
    }
  };

  return (
    <div className={`process-status-toast ${progress.type} ${isDone ? "completed" : ""}`}>
      <div className="toast-left">
        {!isDone ? (
          <Loader2 size={15} className="animate-spin toast-spinner" />
        ) : (
          getIcon()
        )}
        <div className="toast-text-wrap">
          <div className="toast-title">{getTitle()}</div>
          {progress.targetName && !isDone && (
            <div className="toast-target-name">Target: <span>{progress.targetName}</span></div>
          )}
          {progress.message && <div className="toast-message">{progress.message}</div>}
        </div>
      </div>

      <div className="toast-right">
        {!isDone && progress.total > 1 && (
          <div className="toast-progress-bar-wrap">
            <div className="toast-progress-bar-fill" style={{ width: `${percent}%` }} />
          </div>
        )}
        <button className="btn-toast-close" onClick={onDismiss} title="Dismiss">
          <X size={12} />
        </button>
      </div>
    </div>
  );
};
