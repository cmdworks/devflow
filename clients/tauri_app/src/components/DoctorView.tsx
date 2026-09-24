import React, { useState, useMemo } from "react";
import {
  Activity,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  RotateCcw,
  Wrench,
  ShieldCheck,
  Copy,
  Sparkles,
  Smartphone,
  Layers,
  Globe,
  Terminal,
  Cpu,
  Search,
  CheckCheck,
} from "lucide-react";
import type { DoctorReport, DoctorCheck } from "../types";
import { copyToClipboard } from "../utils/clipboard";

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
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | "passed" | "warning" | "failed">("all");

  const handleCopy = async (text: string) => {
    const ok = await copyToClipboard(text);
    if (ok) {
      setCopiedHint(text);
      setTimeout(() => setCopiedHint(null), 1800);
    }
  };

  const summary = useMemo(() => {
    const passed = report?.passed_count ?? report?.summary?.passed ?? 0;
    const warnings = report?.warning_count ?? report?.summary?.warnings ?? 0;
    const failed = report?.failure_count ?? report?.summary?.failed ?? 0;
    return { passed, warnings, failed };
  }, [report]);

  const rawChecks = report?.checks || [];

  const filteredChecks = useMemo(() => {
    return rawChecks.filter((c) => {
      const status = (c.status || "pass").toLowerCase();
      const isPass = status === "pass" || status === "passed";
      const isWarn = status === "warn" || status === "warning";
      const isFail = status === "fail" || status === "failed";

      if (statusFilter === "passed" && !isPass) return false;
      if (statusFilter === "warning" && !isWarn) return false;
      if (statusFilter === "failed" && !isFail) return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesName = c.name.toLowerCase().includes(q);
        const matchesMsg = c.message.toLowerCase().includes(q);
        const matchesCategory = (c.category || "").toLowerCase().includes(q);
        const matchesFix = (c.fix_hint || "").toLowerCase().includes(q);
        return matchesName || matchesMsg || matchesCategory || matchesFix;
      }

      return true;
    });
  }, [rawChecks, statusFilter, searchQuery]);

  // Group checks by category in consistent order
  const categorizedChecks = useMemo(() => {
    const categoryOrder = [
      "Apple & Swift Toolchain",
      "Android & Kotlin / Java",
      "Rust & Tauri Toolchain",
      "Core Web & Scripting Runtimes",
      "Workspace & Project Setup",
      "General & System Tools",
    ];

    const map: Record<string, DoctorCheck[]> = {};

    for (const c of filteredChecks) {
      const cat = c.category || "General & System Tools";
      if (!map[cat]) {
        map[cat] = [];
      }
      map[cat].push(c);
    }

    // Sort categories according to preferred sequence
    return Object.entries(map).sort(([catA], [catB]) => {
      const idxA = categoryOrder.indexOf(catA);
      const idxB = categoryOrder.indexOf(catB);
      const posA = idxA >= 0 ? idxA : 999;
      const posB = idxB >= 0 ? idxB : 999;
      return posA - posB;
    });
  }, [filteredChecks]);

  const getCategoryIcon = (category: string) => {
    const cat = category.toLowerCase();
    if (cat.includes("apple") || cat.includes("swift") || cat.includes("xcode")) {
      return <Cpu size={16} color="#38bdf8" />;
    }
    if (cat.includes("android") || cat.includes("kotlin") || cat.includes("java")) {
      return <Smartphone size={16} color="#34d399" />;
    }
    if (cat.includes("rust") || cat.includes("tauri") || cat.includes("cargo")) {
      return <Terminal size={16} color="#fb923c" />;
    }
    if (cat.includes("web") || cat.includes("scripting") || cat.includes("node")) {
      return <Globe size={16} color="#a78bfa" />;
    }
    return <Layers size={16} color="#06b6d4" />;
  };

  return (
    <div className="view-container">
      {/* Header Banner */}
      <div className="view-header">
        <div>
          <div className="view-title-group">
            <Activity size={22} color="#10b981" />
            <h1 className="view-title">Toolchain & Environment Diagnostics</h1>
            <span
              className={`view-badge health-badge ${
                summary.failed === 0 ? "healthy" : "issues"
              }`}
              style={{
                background:
                  summary.failed === 0
                    ? "rgba(16, 185, 129, 0.15)"
                    : "rgba(239, 68, 68, 0.15)",
                color: summary.failed === 0 ? "#10b981" : "#ef4444",
                border: `1px solid ${
                  summary.failed === 0
                    ? "rgba(16, 185, 129, 0.3)"
                    : "rgba(239, 68, 68, 0.3)"
                }`,
              }}
            >
              {summary.failed === 0 ? "System Healthy" : `${summary.failed} Actionable Issues`}
            </span>
          </div>
          <p className="view-subtitle">
            Automated verification of compilers, platform SDKs (Xcode, Swift, Android, JDK, Cargo, Tauri), and workspace manifests.
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
        <div
          className={`stat-card passed ${statusFilter === "passed" ? "active-filter" : ""}`}
          onClick={() => setStatusFilter(statusFilter === "passed" ? "all" : "passed")}
          style={{ cursor: "pointer" }}
        >
          <div className="stat-top">
            <CheckCircle2 size={18} color="#10b981" />
            <span className="stat-label">Passed Checks</span>
          </div>
          <div className="stat-num">{summary.passed}</div>
          <span className="stat-sub">Compilers & tools operational</span>
        </div>

        <div
          className={`stat-card warnings ${statusFilter === "warning" ? "active-filter" : ""}`}
          onClick={() => setStatusFilter(statusFilter === "warning" ? "all" : "warning")}
          style={{ cursor: "pointer" }}
        >
          <div className="stat-top">
            <AlertTriangle size={18} color="#f59e0b" />
            <span className="stat-label">Warnings</span>
          </div>
          <div className="stat-num">{summary.warnings}</div>
          <span className="stat-sub">Optional tools / minor configuration</span>
        </div>

        <div
          className={`stat-card failed ${statusFilter === "failed" ? "active-filter" : ""}`}
          onClick={() => setStatusFilter(statusFilter === "failed" ? "all" : "failed")}
          style={{ cursor: "pointer" }}
        >
          <div className="stat-top">
            <XCircle size={18} color="#ef4444" />
            <span className="stat-label">Failed Requirements</span>
          </div>
          <div className="stat-num">{summary.failed}</div>
          <span className="stat-sub">Action required to build targets</span>
        </div>
      </div>

      {/* Filter and Search Controls */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginTop: "28px",
          marginBottom: "16px",
          gap: "12px",
          flexWrap: "wrap",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <ShieldCheck size={16} color="#06b6d4" />
          <span style={{ fontWeight: 600, fontSize: "14px", color: "var(--text-primary)" }}>
            Diagnostic Results ({filteredChecks.length} of {rawChecks.length})
          </span>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "6px",
              background: "rgba(15, 23, 42, 0.6)",
              border: "1px solid rgba(255, 255, 255, 0.08)",
              borderRadius: "6px",
              padding: "4px 10px",
            }}
          >
            <Search size={13} color="var(--text-muted)" />
            <input
              type="text"
              placeholder="Search checks, SDKs, tools..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{
                background: "transparent",
                border: "none",
                outline: "none",
                color: "var(--text-primary)",
                fontSize: "12px",
                width: "180px",
              }}
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery("")}
                style={{
                  background: "transparent",
                  border: "none",
                  color: "var(--text-muted)",
                  cursor: "pointer",
                  fontSize: "11px",
                }}
              >
                ✕
              </button>
            )}
          </div>

          <div style={{ display: "flex", gap: "4px" }}>
            {(["all", "passed", "warning", "failed"] as const).map((filter) => (
              <button
                key={filter}
                onClick={() => setStatusFilter(filter)}
                style={{
                  padding: "4px 10px",
                  borderRadius: "5px",
                  fontSize: "11px",
                  fontWeight: 500,
                  cursor: "pointer",
                  border:
                    statusFilter === filter
                      ? "1px solid rgba(6, 182, 212, 0.4)"
                      : "1px solid rgba(255, 255, 255, 0.06)",
                  background:
                    statusFilter === filter
                      ? "rgba(6, 182, 212, 0.15)"
                      : "rgba(15, 23, 42, 0.4)",
                  color:
                    statusFilter === filter
                      ? "#06b6d4"
                      : "var(--text-muted)",
                  transition: "all 0.15s ease",
                }}
              >
                {filter.charAt(0).toUpperCase() + filter.slice(1)}
              </button>
            ))}
          </div>
        </div>
      </div>

      {isLoading ? (
        <div className="empty-card-container">
          <RotateCcw size={32} color="#06b6d4" className="animate-spin" />
          <div className="empty-title">Running Diagnostics...</div>
          <p className="empty-desc">
            Inspecting compilers (Swift, Xcode, Rust, Kotlin, JDK), Android SDK, ADB, and project manifests...
          </p>
        </div>
      ) : categorizedChecks.length === 0 ? (
        <div className="empty-card-container">
          <Wrench size={32} color="#64748b" />
          <div className="empty-title">No Diagnostics Matching Filter</div>
          <p className="empty-desc">Try clearing your search query or status filter.</p>
          <button className="btn-primary" onClick={() => { setSearchQuery(""); setStatusFilter("all"); }}>
            <RotateCcw size={14} />
            <span>Reset Filters</span>
          </button>
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
          {categorizedChecks.map(([category, items]) => {
            const catPassed = items.filter(
              (c) => (c.status || "").toLowerCase().startsWith("pass")
            ).length;
            const catWarnings = items.filter(
              (c) => (c.status || "").toLowerCase().startsWith("warn")
            ).length;
            const catFailed = items.filter(
              (c) => (c.status || "").toLowerCase().startsWith("fail")
            ).length;

            return (
              <div key={category} className="doctor-category-section">
                {/* Category Header */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: "8px 14px",
                    background: "rgba(15, 23, 42, 0.5)",
                    border: "1px solid rgba(255, 255, 255, 0.06)",
                    borderRadius: "8px 8px 0 0",
                    borderBottom: "none",
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    {getCategoryIcon(category)}
                    <span
                      style={{
                        fontWeight: 600,
                        fontSize: "13px",
                        color: "var(--text-primary)",
                        letterSpacing: "0.2px",
                      }}
                    >
                      {category}
                    </span>
                    <span
                      style={{
                        fontSize: "11px",
                        color: "var(--text-muted)",
                        background: "rgba(255, 255, 255, 0.05)",
                        padding: "1px 6px",
                        borderRadius: "10px",
                      }}
                    >
                      {items.length}
                    </span>
                  </div>

                  <div style={{ display: "flex", gap: "6px", fontSize: "11px" }}>
                    {catPassed > 0 && (
                      <span style={{ color: "#10b981", display: "flex", alignItems: "center", gap: "3px" }}>
                        <CheckCircle2 size={11} /> {catPassed}
                      </span>
                    )}
                    {catWarnings > 0 && (
                      <span style={{ color: "#f59e0b", display: "flex", alignItems: "center", gap: "3px" }}>
                        <AlertTriangle size={11} /> {catWarnings}
                      </span>
                    )}
                    {catFailed > 0 && (
                      <span style={{ color: "#ef4444", display: "flex", alignItems: "center", gap: "3px" }}>
                        <XCircle size={11} /> {catFailed}
                      </span>
                    )}
                  </div>
                </div>

                {/* Grid of checks in this category */}
                <div
                  className="doctor-checks-grid"
                  style={{
                    borderTopLeftRadius: 0,
                    borderTopRightRadius: 0,
                    marginTop: 0,
                  }}
                >
                  {items.map((c, i) => {
                    const rawStatus = (c.status || "pass").toLowerCase();
                    const isPass = rawStatus === "pass" || rawStatus === "passed";
                    const isWarn = rawStatus === "warn" || rawStatus === "warning";
                    const isFail = rawStatus === "fail" || rawStatus === "failed";
                    const statusClass = isPass ? "pass" : isWarn ? "warn" : "fail";

                    return (
                      <div key={i} className={`doctor-check-card ${statusClass}-border`}>
                        <div className="check-card-header">
                          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                            {isPass && <CheckCircle2 size={16} color="#10b981" />}
                            {isWarn && <AlertTriangle size={16} color="#f59e0b" />}
                            {isFail && <XCircle size={16} color="#ef4444" />}
                            <span className="check-name">{c.name}</span>
                          </div>

                          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                            {c.detected_version && (
                              <span
                                style={{
                                  fontSize: "10px",
                                  fontFamily: "var(--font-mono, monospace)",
                                  background: "rgba(255, 255, 255, 0.06)",
                                  color: "var(--text-secondary)",
                                  padding: "2px 6px",
                                  borderRadius: "4px",
                                  maxWidth: "160px",
                                  overflow: "hidden",
                                  textOverflow: "ellipsis",
                                  whiteSpace: "nowrap",
                                }}
                                title={c.detected_version}
                              >
                                {c.detected_version}
                              </span>
                            )}
                            <span className={`check-status-pill ${statusClass}`}>
                              {isPass ? "PASSED" : isWarn ? "WARNING" : "FAILED"}
                            </span>
                          </div>
                        </div>

                        <div className="check-card-body">
                          <div className="check-msg">{c.message}</div>
                          {c.details && <div className="check-details font-mono">{c.details}</div>}

                          {c.fix_hint && (
                            <div className="check-fix-box">
                              <div className="fix-header">
                                <Sparkles size={13} color="#f59e0b" />
                                <span>Suggested Remediation</span>
                              </div>
                              <div className="fix-content-row">
                                <code className="fix-code">{c.fix_hint}</code>
                                <button
                                  className="copy-btn"
                                  onClick={() => handleCopy(c.fix_hint || "")}
                                  title="Copy fix command to clipboard"
                                >
                                  {copiedHint === c.fix_hint ? (
                                    <CheckCheck size={13} color="#10b981" />
                                  ) : (
                                    <Copy size={13} />
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
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

