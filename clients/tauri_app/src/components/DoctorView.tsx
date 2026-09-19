import React, { useState } from "react";
import {
  Activity,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  RotateCcw,
  Wrench,
  ShieldCheck,
  Check,
  Copy,
  Sparkles,
} from "lucide-react";
import type { DoctorReport } from "../types";

interface DoctorViewProps {
  report: DoctorReport | null;
  isLoading: boolean;
  onRefreshDoctor: () => void;
}

export const DoctorView: React.FC<DoctorViewProps> = ({
  report,
  isLoading,
  onRefreshDoctor,
}) => {
  const [copiedHint, setCopiedHint] = useState<string | null>(null);

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedHint(text);
    setTimeout(() => setCopiedHint(null), 1500);
  };

  const summary = report?.summary || { passed: 0, warnings: 0, failed: 0 };
  const checks = report?.checks || [];

  return (
    <div className="view-container">
      {/* Header Banner */}
      <div className="view-header">
        <div>
          <div className="view-title-group">
            <Activity size={22} color="#10b981" />
            <h1 className="view-title">Toolchain & Environment Diagnostics</h1>
            <span className="view-badge health-badge">
              {summary.failed === 0 ? "System Healthy" : `${summary.failed} Issues Found`}
            </span>
          </div>
          <p className="view-subtitle">
            Automated verification of your compilers, SDKs, device bridges, and workspace configurations.
          </p>
        </div>

        <div className="view-actions">
          <button
            className="btn-glass"
            onClick={onRefreshDoctor}
            disabled={isLoading}
            title="Re-run all toolchain diagnostics"
          >
            <RotateCcw size={14} className={isLoading ? "animate-spin" : ""} />
            <span>{isLoading ? "Diagnosing..." : "Run Diagnostics"}</span>
          </button>
        </div>
      </div>

      {/* Summary Score Bar */}
      <div className="doctor-stats-bar">
        <div className="stat-card passed">
          <div className="stat-top">
            <CheckCircle2 size={18} color="#10b981" />
            <span className="stat-label">Passed Checks</span>
          </div>
          <div className="stat-num">{summary.passed}</div>
          <span className="stat-sub">Compilers & tools operational</span>
        </div>

        <div className="stat-card warnings">
          <div className="stat-top">
            <AlertTriangle size={18} color="#f59e0b" />
            <span className="stat-label">Warnings</span>
          </div>
          <div className="stat-num">{summary.warnings}</div>
          <span className="stat-sub">Optional tools / minor configuration</span>
        </div>

        <div className="stat-card failed">
          <div className="stat-top">
            <XCircle size={18} color="#ef4444" />
            <span className="stat-label">Failed Requirements</span>
          </div>
          <div className="stat-num">{summary.failed}</div>
          <span className="stat-sub">Action required to build targets</span>
        </div>
      </div>

      {/* Diagnostics List */}
      <div className="section-title" style={{ marginTop: "28px" }}>
        <ShieldCheck size={15} color="#06b6d4" />
        <span>Diagnostic Results ({checks.length} Checks)</span>
      </div>

      {isLoading ? (
        <div className="empty-card-container">
          <RotateCcw size={32} color="#06b6d4" className="animate-spin" />
          <div className="empty-title">Running Diagnostics...</div>
          <p className="empty-desc">
            Inspecting compilers (Swift, Rust, Kotlin), Android SDK, Xcode, and device bridge bridges...
          </p>
        </div>
      ) : checks.length === 0 ? (
        <div className="empty-card-container">
          <Wrench size={32} color="#64748b" />
          <div className="empty-title">No Diagnostics Available</div>
          <p className="empty-desc">Click Run Diagnostics to perform an automated health audit of your system.</p>
          <button className="btn-primary" onClick={onRefreshDoctor}>
            <RotateCcw size={14} />
            <span>Start Health Audit</span>
          </button>
        </div>
      ) : (
        <div className="doctor-checks-grid">
          {checks.map((c, i) => {
            const status = (c.status || "pass").toLowerCase();
            const isPass = status === "pass";
            const isWarn = status === "warn";
            const isFail = status === "fail";

            return (
              <div key={i} className={`doctor-check-card ${status}-border`}>
                <div className="check-card-header">
                  <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                    {isPass && <CheckCircle2 size={16} color="#10b981" />}
                    {isWarn && <AlertTriangle size={16} color="#f59e0b" />}
                    {isFail && <XCircle size={16} color="#ef4444" />}
                    <span className="check-name">{c.name}</span>
                  </div>

                  <span className={`check-status-pill ${status}`}>
                    {c.status.toUpperCase()}
                  </span>
                </div>

                <div className="check-card-body">
                  <div className="check-msg">{c.message}</div>
                  {c.details && <div className="check-details font-mono">{c.details}</div>}

                  {c.fix_hint && (
                    <div className="check-fix-box">
                      <div className="fix-header">
                        <Sparkles size={13} color="#f59e0b" />
                        <span>Suggested Fix</span>
                      </div>
                      <div className="fix-content-row">
                        <code className="fix-code">{c.fix_hint}</code>
                        <button
                          className="copy-btn"
                          onClick={() => handleCopy(c.fix_hint || "")}
                          title="Copy fix command"
                        >
                          {copiedHint === c.fix_hint ? (
                            <Check size={12} color="#10b981" />
                          ) : (
                            <Copy size={12} />
                          )}
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
