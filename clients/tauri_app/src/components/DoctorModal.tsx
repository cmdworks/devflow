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
                  <span>{report.summary.passed} Passed</span>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "5px", color: "#fbbf24" }}>
                  <AlertTriangle size={14} />
                  <span>{report.summary.warnings} Warnings</span>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "5px", color: "#f87171" }}>
                  <XCircle size={14} />
                  <span>{report.summary.failed} Failed</span>
                </div>
              </div>

              {report.checks.map((c, i) => {
                const statusClass = c.status.toLowerCase();
                return (
                  <div key={i} className="doctor-card">
                    <div className="doctor-card-top">
                      <span className="doctor-name">{c.name}</span>
                      <span className={`doctor-status-pill ${statusClass}`}>{c.status}</span>
                    </div>
                    <div className="doctor-msg">{c.message}</div>
                    {c.details && <div className="doctor-msg" style={{ opacity: 0.8 }}>{c.details}</div>}
                    {c.fix_hint && (
                      <div className="doctor-hint">
                        💡 <strong>Fix:</strong> {c.fix_hint}
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
