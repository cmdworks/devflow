import React, { useState, useEffect, useCallback } from "react";
import { TopBar } from "./components/TopBar";
import { PrimarySidebar } from "./components/PrimarySidebar";
import { SecondarySidebar } from "./components/SecondarySidebar";
import { TerminalSecondaryBar } from "./components/TerminalSecondaryBar";
import { TerminalGrid } from "./components/TerminalGrid";
import { TerminalLogTree } from "./components/TerminalLogTree";
import { WorkspaceOverviewView } from "./components/WorkspaceOverviewView";
import { TargetsView } from "./components/TargetsView";
import { DevicesView } from "./components/DevicesView";
import { DoctorView } from "./components/DoctorView";
import { McpView } from "./components/McpView";
import { SettingsView } from "./components/SettingsView";
import { WorkspaceModal } from "./components/WorkspaceModal";
import { UpdateModal } from "./components/UpdateModal";
import { ProcessStatusToast } from "./components/ProcessStatusToast";
import { ActionStatusToast } from "./components/ActionStatusToast";
import { DevServerOptionsModal } from "./components/DevServerOptionsModal";
import { useDevFlowApi } from "./hooks/useDevFlowApi";
import { useAutoUpdater } from "./hooks/useAutoUpdater";
import { usePaneLogsState } from "./hooks/usePaneLogsState";
import { useWorkspaceState } from "./hooks/useWorkspaceState";
import { useTargetOperations } from "./hooks/useTargetOperations";
import { copyToClipboard } from "./utils/clipboard";
import type { DoctorReport, ViewSection } from "./types";

export const App: React.FC = () => {
  const api = useDevFlowApi();

  // Navigation View State (persisted across sessions)
  const [activeSection, setActiveSection] = useState<ViewSection>(() => {
    return (localStorage.getItem("devflow_active_section") as ViewSection) || "overview";
  });
  const [isPrimarySidebarCollapsed, setIsPrimarySidebarCollapsed] = useState<boolean>(() => {
    return localStorage.getItem("devflow_primary_sidebar_collapsed") === "true";
  });
  const [isSecondarySidebarCollapsed, setIsSecondarySidebarCollapsed] = useState<boolean>(() => {
    return localStorage.getItem("devflow_secondary_sidebar_collapsed") === "true";
  });

  useEffect(() => {
    localStorage.setItem("devflow_active_section", activeSection);
  }, [activeSection]);

  useEffect(() => {
    localStorage.setItem(
      "devflow_primary_sidebar_collapsed",
      isPrimarySidebarCollapsed ? "true" : "false"
    );
  }, [isPrimarySidebarCollapsed]);

  useEffect(() => {
    localStorage.setItem(
      "devflow_secondary_sidebar_collapsed",
      isSecondarySidebarCollapsed ? "true" : "false"
    );
  }, [isSecondarySidebarCollapsed]);

  // Terminal Panes & Ingestion Logs Hook
  const {
    openPanes,
    setOpenPanes,
    logsByPaneId,
    setLogsByPaneId,
    activePaneId,
    setActivePaneId,
    maximizedPaneId,
    setMaximizedPaneId,
    layoutMode,
    setLayoutMode,
    levelFilter,
    setLevelFilter,
    tagFilter,
    setTagFilter,
    searchQuery,
    setSearchQuery,
    isLogTreeOpen,
    setIsLogTreeOpen,
    appendLogToPane,
    updatePaneStatus,
    openTargetPane,
    openCombinedPane,
    handleClosePane,
    handleClearActiveLogs,
    handleToggleMaximize,
  } = usePaneLogsState();

  // Workspace Management & Zero-Flicker Cache Hook
  const {
    workspaceName,
    workspacePath,
    knownWorkspaces,
    targets,
    devices,
    activeSessions,
    isWorkspaceModalOpen,
    setIsWorkspaceModalOpen,
    setDevices,
    handleSwitchWorkspace,
    handleRenameWorkspace,
    handleRemoveWorkspace,
  } = useWorkspaceState({
    openPanes,
    activePaneId,
    logsByPaneId,
    activeSection,
    setOpenPanes,
    setActivePaneId,
    setLogsByPaneId,
  });

  // Target Lifecycle Operations & Batch Orchestration Hook
  const {
    isStartingAll,
    isStoppingAll,
    batchProgress,
    setBatchProgress,
    actionFeedback,
    dismissActionFeedback,
    executingActions,
    devOptionsModal,
    setDevOptionsModal,
    handleToggleRun,
    handleReload,
    handleRestart,
    handleLaunchWithOptions,
    handleExecuteAction,
    handleRunAll,
    handleReloadAll,
    handleRestartAll,
    handleStopAll,
    runningTargetsCount,
    allTargetsRunning,
    anyTargetRunning,
  } = useTargetOperations({
    targets,
    devices,
    openPanes,
    activeSessions,
    updatePaneStatus,
    appendLogToPane,
    openTargetPane,
    setActivePaneId,
    setActiveSection,
  });

  // Software Auto-Updater Hook
  const {
    isUpdateModalOpen,
    setIsUpdateModalOpen,
    updateInfo,
    isCheckingUpdate,
    handleCheckUpdates,
    handleInstallUpdate,
    handleRestartApp,
  } = useAutoUpdater();

  // Diagnostics / Doctor State
  const [doctorReport, setDoctorReport] = useState<DoctorReport | null>(null);
  const [isDoctorLoading, setIsDoctorLoading] = useState<boolean>(false);

  const handleRefreshDoctor = useCallback(async () => {
    setIsDoctorLoading(true);
    try {
      const report = await api.runDoctor(workspacePath);
      setDoctorReport(report);
    } catch (e) {
      console.error("Doctor error:", e);
    } finally {
      setIsDoctorLoading(false);
    }
  }, [api, workspacePath]);

  const handleBootEmulator = useCallback(
    async (name: string): Promise<{ success: boolean; message?: string; error?: string }> => {
      try {
        const res = await api.bootEmulator(name);
        if (res.success) {
          const devs = await api.fetchDevices();
          setDevices(devs);
          return res;
        } else {
          return { success: false, error: res.error || "Failed to boot emulator." };
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        return { success: false, error: `Connection failed: ${msg}` };
      }
    },
    [api, setDevices]
  );

  const handleCopyActivePaneLogs = useCallback(async () => {
    const currentLogs = logsByPaneId[activePaneId] || [];
    if (currentLogs.length === 0) return false;
    const text = currentLogs
      .map((l) => `[${l.level}] ${l.timestamp || ""} ${l.message}`)
      .join("\n");
    return await copyToClipboard(text);
  }, [logsByPaneId, activePaneId]);

  return (
    <div className="app-container">
      {/* 1. Global Top Bar */}
      <TopBar
        workspaceName={workspaceName}
        activeSection={activeSection}
        isPrimarySidebarCollapsed={isPrimarySidebarCollapsed}
        isSecondarySidebarCollapsed={isSecondarySidebarCollapsed}
        searchQuery={searchQuery}
        levelFilter={levelFilter}
        targetsCount={targets.length}
        runningTargetsCount={runningTargetsCount}
        allTargetsRunning={allTargetsRunning}
        anyTargetRunning={anyTargetRunning}
        isStartingAll={isStartingAll}
        isStoppingAll={isStoppingAll}
        actionFeedback={actionFeedback}
        onChangeSearch={setSearchQuery}
        onSelectLevel={setLevelFilter}
        onTogglePrimarySidebar={() => setIsPrimarySidebarCollapsed(!isPrimarySidebarCollapsed)}
        onToggleSecondarySidebar={() => setIsSecondarySidebarCollapsed(!isSecondarySidebarCollapsed)}
        onSelectSection={setActiveSection}
        onRunAll={handleRunAll}
        onReloadAll={handleReloadAll}
        onRestartAll={handleRestartAll}
        onStopAll={handleStopAll}
        onOpenDevOptions={() => setDevOptionsModal({ isOpen: true })}
        updateAvailable={updateInfo}
        onOpenUpdateModal={() => setIsUpdateModalOpen(true)}
      />

      <div className="main-body">
        {/* 2. Primary Sidebar (Workspaces Rail / Drawer) */}
        <PrimarySidebar
          activeWorkspacePath={workspacePath}
          knownWorkspaces={knownWorkspaces}
          activeSection={activeSection}
          devices={devices}
          isCollapsed={isPrimarySidebarCollapsed}
          onToggleCollapse={() => setIsPrimarySidebarCollapsed(!isPrimarySidebarCollapsed)}
          onSelectWorkspace={handleSwitchWorkspace}
          onAddWorkspace={() => setIsWorkspaceModalOpen(true)}
          onSelectSection={setActiveSection}
          onRenameWorkspace={handleRenameWorkspace}
          onRemoveWorkspace={handleRemoveWorkspace}
        />

        {/* 3. Secondary Sidebar (Active Workspace Sub-Navigation & Project Switcher) */}
        <SecondarySidebar
          workspaceName={workspaceName}
          workspacePath={workspacePath}
          activeSection={activeSection}
          activePaneId={activePaneId}
          targets={targets}
          openPanes={openPanes}
          activeSessions={activeSessions}
          isCollapsed={isSecondarySidebarCollapsed}
          onToggleCollapse={() => setIsSecondarySidebarCollapsed(!isSecondarySidebarCollapsed)}
          onSelectSection={setActiveSection}
          onOpenTargetPane={openTargetPane}
          onOpenCombinedPane={openCombinedPane}
        />

        {/* 4. Main Viewport with Persistent DOM Views (Zero Blinking, Instant Switching) */}
        <main className="viewport">
          {/* Terminal Mode with Secondary Top Bar */}
          <div
            className={`viewport-view-container ${activeSection === "terminal" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "terminal" ? "flex" : "none",
              flexDirection: "column",
              height: "100%",
              width: "100%",
            }}
          >
            <TerminalSecondaryBar
              panes={openPanes}
              activePaneId={activePaneId}
              layoutMode={layoutMode}
              availableTargets={targets}
              isMaximized={!!maximizedPaneId}
              isLogTreeOpen={isLogTreeOpen}
              levelFilter={levelFilter}
              searchQuery={searchQuery}
              onSelectTab={setActivePaneId}
              onClosePane={handleClosePane}
              onOpenTargetPane={openTargetPane}
              onOpenCombinedPane={openCombinedPane}
              onChangeLayout={setLayoutMode}
              onToggleMaximize={handleToggleMaximize}
              onToggleLogTree={() => setIsLogTreeOpen((prev) => !prev)}
              onCopyActiveLogs={handleCopyActivePaneLogs}
              onClearActiveLogs={handleClearActiveLogs}
              onSelectLevel={setLevelFilter}
              onChangeSearch={setSearchQuery}
            />

            <div className="terminal-workspace-area">
              <div className="terminal-workspace-split">
                {isLogTreeOpen && (
                  <TerminalLogTree
                    logs={logsByPaneId[activePaneId] || []}
                    activeLevelFilter={levelFilter}
                    activeTagFilter={tagFilter}
                    onSelectLevelFilter={setLevelFilter}
                    onSelectTagFilter={setTagFilter}
                  />
                )}
                <div className="terminal-grid-container">
                  <TerminalGrid
                    panes={openPanes}
                    activePaneId={activePaneId}
                    maximizedPaneId={maximizedPaneId}
                    logsByPaneId={logsByPaneId}
                    layoutMode={layoutMode}
                    levelFilter={levelFilter}
                    tagFilter={tagFilter}
                    searchQuery={searchQuery}
                    executingActions={executingActions}
                    onFocusPane={setActivePaneId}
                    onToggleRun={handleToggleRun}
                    onReload={handleReload}
                    onRestart={handleRestart}
                    onExecuteAction={handleExecuteAction}
                    onOpenDevOptions={(id) => setDevOptionsModal({ isOpen: true, targetId: id })}
                    onSplitRight={() => setLayoutMode("split-h")}
                    onSplitDown={() => setLayoutMode("split-v")}
                    onToggleMaximize={(id) => setMaximizedPaneId(maximizedPaneId === id ? null : id)}
                    onClearLogs={(id) => setLogsByPaneId((prev) => ({ ...prev, [id]: [] }))}
                    onClosePane={handleClosePane}
                    onOpenAllPanes={() => {}}
                    onOpenCombinedPane={openCombinedPane}
                  />
                </div>
              </div>
            </div>
          </div>

          {/* Overview Mode */}
          <div
            className={`viewport-view-container ${activeSection === "overview" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "overview" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <WorkspaceOverviewView
              workspaceName={workspaceName}
              workspacePath={workspacePath}
              targets={targets}
              openPanes={openPanes}
              activeSessions={activeSessions}
              devices={devices}
              isStartingAll={isStartingAll}
              isStoppingAll={isStoppingAll}
              allTargetsRunning={allTargetsRunning}
              anyTargetRunning={anyTargetRunning}
              onOpenTargetPane={openTargetPane}
              onRunAll={handleRunAll}
              onReloadAll={handleReloadAll}
              onRestartAll={handleRestartAll}
              onStopAll={handleStopAll}
              onToggleRunTarget={handleToggleRun}
              onReloadTarget={handleReload}
              onRestartTarget={handleRestart}
              onOpenDevOptions={(id) => setDevOptionsModal({ isOpen: true, targetId: id })}
              onSwitchToTerminal={() => setActiveSection("terminal")}
              onSwitchToTargets={() => setActiveSection("targets")}
            />
          </div>

          {/* Targets & Process Matrix Mode */}
          <div
            className={`viewport-view-container ${activeSection === "targets" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "targets" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <TargetsView
              targets={targets}
              openPanes={openPanes}
              activeSessions={activeSessions}
              workspaceName={workspaceName}
              workspacePath={workspacePath}
              isStartingAll={isStartingAll}
              isStoppingAll={isStoppingAll}
              allTargetsRunning={allTargetsRunning}
              anyTargetRunning={anyTargetRunning}
              executingActions={executingActions}
              onToggleRun={handleToggleRun}
              onReload={handleReload}
              onRestart={handleRestart}
              onExecuteAction={handleExecuteAction}
              onOpenTargetPane={openTargetPane}
              onRunAll={handleRunAll}
              onReloadAll={handleReloadAll}
              onRestartAll={handleRestartAll}
              onStopAll={handleStopAll}
              onOpenDevOptions={(id) => setDevOptionsModal({ isOpen: true, targetId: id })}
              onSwitchToTerminal={(paneId) => {
                if (paneId) setActivePaneId(paneId);
                setActiveSection("terminal");
              }}
            />
          </div>

          {/* Devices & Emulators Hub */}
          <div
            className={`viewport-view-container ${activeSection === "devices" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "devices" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <DevicesView
              devices={devices}
              onRefreshDevices={async () => {
                const devs = await api.fetchDevices();
                setDevices(devs);
              }}
              onBootEmulator={handleBootEmulator}
            />
          </div>

          {/* Doctor Diagnostics Hub */}
          <div
            className={`viewport-view-container ${activeSection === "doctor" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "doctor" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <DoctorView
              report={doctorReport}
              isLoading={isDoctorLoading}
              onRefreshDoctor={handleRefreshDoctor}
            />
          </div>

          {/* Model Context Protocol (MCP) Hub Tab */}
          <div
            className={`viewport-view-container ${activeSection === "mcp" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "mcp" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <McpView
              onFetchStatus={api.fetchMcpStatus}
              onToggleServer={api.toggleMcpServer}
              onFetchLogs={api.fetchMcpLogs}
              onFetchSessions={api.fetchMcpSessions}
              onFetchAgents={api.fetchMcpAgents}
              onDeleteLog={api.deleteMcpLog}
              onClearLogs={api.clearMcpLogs}
            />
          </div>

          {/* Settings & System Environment Tab */}
          <div
            className={`viewport-view-container ${activeSection === "settings" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "settings" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <SettingsView
              onInstallCli={async () => {
                const res = await api.installShellCli();
                if (!res.success) throw new Error(res.error);
              }}
              onUninstallCli={async () => {
                const res = await api.uninstallShellCli();
                if (!res.success) throw new Error(res.error);
              }}
              onNavigateToMcp={() => setActiveSection("mcp")}
              workspacePath={workspacePath}
              updateInfo={updateInfo}
              isCheckingUpdate={isCheckingUpdate}
              onCheckUpdates={handleCheckUpdates}
              onOpenUpdateModal={() => setIsUpdateModalOpen(true)}
            />
          </div>
        </main>
      </div>

      {/* 5. Process Lifecycle Status Toast Notification */}
      <ProcessStatusToast
        progress={batchProgress}
        onDismiss={() => setBatchProgress(null)}
      />

      {/* 6. Real-time Target Action Feedback Toast Notification */}
      <ActionStatusToast
        feedback={actionFeedback}
        onDismiss={dismissActionFeedback}
      />

      {/* 6. Dev Server & Runner Options Modal */}
      <DevServerOptionsModal
        isOpen={devOptionsModal.isOpen}
        onClose={() => setDevOptionsModal({ isOpen: false })}
        targets={targets}
        initialTargetId={devOptionsModal.targetId}
        onLaunchWithOptions={handleLaunchWithOptions}
        onStopTarget={async (id) => {
          await api.stopTarget(id);
        }}
      />

      {/* 7. Workspace Add / Switch Modal */}
      <WorkspaceModal
        isOpen={isWorkspaceModalOpen}
        onClose={() => setIsWorkspaceModalOpen(false)}
        currentPath={workspacePath}
        knownWorkspaces={knownWorkspaces}
        onRemoveWorkspace={handleRemoveWorkspace}
        onSelectWorkspace={(path, immediateData) => {
          handleSwitchWorkspace(path, immediateData);
          setIsWorkspaceModalOpen(false);
        }}
      />

      {/* 8. Integrated Software Update Modal */}
      <UpdateModal
        isOpen={isUpdateModalOpen}
        onClose={() => setIsUpdateModalOpen(false)}
        updateInfo={updateInfo}
        onInstallUpdate={handleInstallUpdate}
        onRestartApp={handleRestartApp}
      />
    </div>
  );
};

export default App;
