import React from "react";
import { X, CheckCircle, AlertTriangle, XCircle, Wrench } from "lucide-react";
import type { DoctorReport } from "../types";

interface DoctorModalProps {
  report: DoctorReport | null;
  isLoading: boolean;
  onClose: () => void;
}

export const DoctorModal: React.FC<DoctorModalProps> = ({ report, isLoading, onClose }) => {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Wrench size={16} color="#10b981" />
            <span className="modal-title">DevFlow Environment Doctor</span>
          </div>
          <button
            onClick={onClose}
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-muted)",
              cursor: "pointer",
            }}
          >
            <X size={18} />
          </button>
        </div>

        <div className="modal-body">
          {isLoading ? (
            <div style={{ textAlign: "center", padding: "30px 0", color: "var(--text-secondary)" }}>
              Running full toolchain and workspace diagnostics...
            </div>
          ) : report ? (
            <div>
              <div
                style={{
                  display: "flex",
                  gap: "12px",
                  marginBottom: "16px",
                  padding: "10px 14px",
                  background: "rgba(15, 23, 42, 0.6)",
                  borderRadius: "6px",
                  fontSize: "12px",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "5px", color: "#34d399" }}>
                  <CheckCircle size={14} />
                  <span>{report.passed_count ?? report.summary?.passed ?? 0} Passed</span>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "5px", color: "#fbbf24" }}>
                  <AlertTriangle size={14} />
                  <span>{report.warning_count ?? report.summary?.warnings ?? 0} Warnings</span>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "5px", color: "#f87171" }}>
                  <XCircle size={14} />
                  <span>{report.failure_count ?? report.summary?.failed ?? 0} Failed</span>
                </div>
              </div>

              {report.checks.map((c, i) => {
                const rawStatus = (c.status || "pass").toLowerCase();
                const isPass = rawStatus === "pass" || rawStatus === "passed";
                const isWarn = rawStatus === "warn" || rawStatus === "warning";
                const statusClass = isPass ? "pass" : isWarn ? "warn" : "fail";
                const statusLabel = isPass ? "PASSED" : isWarn ? "WARNING" : "FAILED";

                return (
                  <div key={i} className="doctor-card">
                    <div className="doctor-card-top">
                      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                        <span className="doctor-name">{c.name}</span>
                        {c.category && (
                          <span style={{ fontSize: "10px", opacity: 0.6 }}>({c.category})</span>
                        )}
                      </div>
                      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                        {c.detected_version && (
                          <span style={{ fontSize: "10px", opacity: 0.75, fontFamily: "monospace" }}>
                            {c.detected_version}
                          </span>
                        )}
                        <span className={`doctor-status-pill ${statusClass}`}>{statusLabel}</span>
                      </div>
                    </div>
                    <div className="doctor-msg">{c.message}</div>
                    {c.details && <div className="doctor-msg" style={{ opacity: 0.8 }}>{c.details}</div>}
                    {c.fix_hint && (
                      <div className="doctor-hint" style={{ marginTop: "6px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <span>💡 <strong>Fix:</strong> <code>{c.fix_hint}</code></span>
                        <button
                          onClick={() => navigator.clipboard.writeText(c.fix_hint || "")}
                          style={{
                            background: "rgba(255, 255, 255, 0.08)",
                            border: "none",
                            color: "var(--text-primary)",
                            padding: "2px 6px",
                            borderRadius: "4px",
                            fontSize: "11px",
                            cursor: "pointer",
                          }}
                        >
                          Copy
                        </button>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          ) : (
            <div style={{ color: "var(--text-muted)", textAlign: "center" }}>No diagnostic results available</div>
          )}
        </div>
      </div>
    </div>
  );
};
