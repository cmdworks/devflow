/**
 * DevFlow Desktop Companion App Logic
 * Manages workspace discovery, dynamic multi-pane layouts, Termax virtualized log streams,
 * and real-time SSE / Tauri event synchronization.
 */

class DevFlowApp {
  constructor() {
    this.apiBase = window.location.origin.includes('localhost') || window.location.origin.includes('127.0.0.1')
      ? window.location.origin
      : 'http://localhost:9090';

    this.workspaceDir = '';
    this.targets = [];
    this.devices = [];
    this.activeSessions = [];
    this.openPanes = []; // Array of { id, targetId, title, platform, isCombined, engine, el, status, buildTime }
    this.eventSource = null;

    this.initElements();
    this.attachEventListeners();
    this.init();
  }

  initElements() {
    // Top bar elements
    this.wsNameDisplay = document.getElementById('wsNameDisplay');
    this.wsPathInput = document.getElementById('wsPathInput');
    this.btnSwitchWs = document.getElementById('btnSwitchWs');
    this.topDevicesBar = document.getElementById('topDevicesBar');

    this.btnGlobalRunAll = document.getElementById('btnGlobalRunAll');
    this.btnGlobalReloadAll = document.getElementById('btnGlobalReloadAll');
    this.btnGlobalRestartAll = document.getElementById('btnGlobalRestartAll');
    this.btnGlobalStopAll = document.getElementById('btnGlobalStopAll');
    this.btnTopDoctor = document.getElementById('btnTopDoctor');
    this.btnInstallCli = document.getElementById('btnInstallCli');

    // Sidebar elements
    this.targetCountBadge = document.getElementById('targetCountBadge');
    this.subProjectsList = document.getElementById('subProjectsList');
    this.btnCombineAll = document.getElementById('btnCombineAll');
    this.sidebarDevicesList = document.getElementById('sidebarDevicesList');
    this.btnRefreshDevices = document.getElementById('btnRefreshDevices');
    this.emulatorInput = document.getElementById('emulatorInput');
    this.btnBootEmu = document.getElementById('btnBootEmu');
    this.externalSessionsList = document.getElementById('externalSessionsList');

    // Main viewport elements
    this.panesGrid = document.getElementById('panesGrid');
    this.viewportEmptyState = document.getElementById('viewportEmptyState');
    this.btnEmptyOpenAll = document.getElementById('btnEmptyOpenAll');
    this.btnEmptyCombine = document.getElementById('btnEmptyCombine');

    // Modals
    this.doctorModal = document.getElementById('doctorModal');
    this.doctorResults = document.getElementById('doctorResults');
    this.btnCloseDoctor = document.getElementById('btnCloseDoctor');
  }

  attachEventListeners() {
    // Top bar workspace edit
    this.btnSwitchWs.addEventListener('click', () => {
      if (this.wsPathInput.classList.contains('hidden')) {
        this.wsPathInput.value = this.workspaceDir;
        this.wsPathInput.classList.remove('hidden');
        this.wsNameDisplay.classList.add('hidden');
        this.wsPathInput.focus();
      } else {
        this.applyWorkspacePath(this.wsPathInput.value.trim());
      }
    });

    this.wsPathInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        this.applyWorkspacePath(this.wsPathInput.value.trim());
      } else if (e.key === 'Escape') {
        this.wsPathInput.classList.add('hidden');
        this.wsNameDisplay.classList.remove('hidden');
      }
    });

    // Top bar global actions
    this.btnGlobalRunAll.addEventListener('click', () => this.runAllTargets());
    this.btnGlobalReloadAll.addEventListener('click', () => this.reloadAll());
    this.btnGlobalRestartAll.addEventListener('click', () => this.restartAll());
    this.btnGlobalStopAll.addEventListener('click', () => this.stopAll());
    this.btnTopDoctor.addEventListener('click', () => this.runDoctor());
    if (this.btnInstallCli) {
      this.btnInstallCli.addEventListener('click', () => this.installCliInPath());
    }

    // Sidebar combine all
    this.btnCombineAll.addEventListener('click', () => this.openCombinedPane());

    // Devices & Emulator
    this.btnRefreshDevices.addEventListener('click', () => this.refreshDevices());
    this.btnBootEmu.addEventListener('click', () => {
      const name = this.emulatorInput.value.trim();
      if (name) this.bootEmulator(name);
    });

    // Empty state buttons
    this.btnEmptyOpenAll.addEventListener('click', () => this.openAllSubProjects());
    this.btnEmptyCombine.addEventListener('click', () => this.openCombinedPane());

    // Doctor modal close
    this.btnCloseDoctor.addEventListener('click', () => {
      this.doctorModal.classList.add('hidden');
    });
  }

  async init() {
    // Parse URL params for ?dir=
    const params = new URLSearchParams(window.location.search);
    const dirParam = params.get('dir');
    await this.fetchWorkspace(dirParam || '');
    this.connectEvents();
  }

  async fetchWorkspace(dirPath = '') {
    try {
      const url = dirPath ? `${this.apiBase}/api/workspace?dir=${encodeURIComponent(dirPath)}` : `${this.apiBase}/api/workspace`;
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP error ${res.status}`);
      const data = await res.json();

      this.workspaceDir = data.workspace_path;
      this.targets = data.targets || [];
      this.devices = data.devices || [];
      this.activeSessions = data.active_sessions || [];

      this.renderTopBar(data.workspace_name);
      this.renderSidebar();

      // If no open panes, automatically open discovered sub-project panes
      if (this.openPanes.length === 0 && this.targets.length > 0) {
        for (const target of this.targets) {
          this.openSubProjectPane(target);
        }
      }
    } catch (e) {
      console.warn('Failed to fetch workspace from backend, using fallback:', e);
      this.renderTopBar('mac-mtp (Fallback)');
      this.renderFallbackTargets();
    }
  }

  applyWorkspacePath(newPath) {
    this.wsPathInput.classList.add('hidden');
    this.wsNameDisplay.classList.remove('hidden');
    if (newPath && newPath !== this.workspaceDir) {
      // Clear panes and reload workspace
      this.clearAllPanes();
      this.fetchWorkspace(newPath);
    }
  }

  renderTopBar(workspaceName) {
    this.wsNameDisplay.textContent = `${workspaceName} (${this.workspaceDir})`;

    // Render devices in top bar
    if (this.devices.length === 0) {
      this.topDevicesBar.innerHTML = `
        <div class="device-pill">
          <span class="pulse-dot yellow"></span>
          <span class="dev-label">No devices detected</span>
        </div>
      `;
    } else {
      this.topDevicesBar.innerHTML = this.devices.map(d => {
        const isOnline = d.state === 'Connected' || d.state === 'Booted' || d.online;
        const dotClass = isOnline ? 'green' : 'gray';
        return `
          <div class="device-pill" title="Platform: ${d.platform} | ID: ${d.id}">
            <span class="pulse-dot ${dotClass}"></span>
            <span class="dev-label">${this.escapeHtml(d.name)} [${d.platform}]</span>
          </div>
        `;
      }).join('');
    }
  }

  renderSidebar() {
    this.targetCountBadge.textContent = this.targets.length;

    // Sub-Projects list
    if (this.targets.length === 0) {
      this.subProjectsList.innerHTML = '<div class="sidebar-empty">No recognized targets found</div>';
    } else {
      this.subProjectsList.innerHTML = this.targets.map(target => {
        const isOpen = this.openPanes.some(p => p.targetId === target.id);
        const activeClass = isOpen ? 'active-in-pane' : '';
        const platformClass = (target.platform || 'generic').toLowerCase();

        return `
          <div class="subproject-card ${activeClass}" data-target-id="${target.id}">
            <div class="subproject-info">
              <div class="subproject-title">${this.escapeHtml(target.name)}</div>
              <div class="subproject-meta">
                <span class="platform-tag ${platformClass}">${target.platform}</span>
                <span class="target-status-dot idle" id="dot-${target.id}"></span>
                <span style="color:#64748b">${this.escapeHtml(target.framework)}</span>
              </div>
            </div>
            <button class="subproject-action-btn" title="Open Pane">＋</button>
          </div>
        `;
      }).join('');

      // Add click handlers on subproject cards
      this.subProjectsList.querySelectorAll('.subproject-card').forEach(card => {
        card.addEventListener('click', () => {
          const targetId = card.getAttribute('data-target-id');
          const target = this.targets.find(t => t.id === targetId);
          if (target) {
            this.openSubProjectPane(target);
          }
        });
      });
    }

    // Devices list in sidebar
    if (this.devices.length === 0) {
      this.sidebarDevicesList.innerHTML = '<div class="sidebar-empty">No devices discovered</div>';
    } else {
      this.sidebarDevicesList.innerHTML = this.devices.map(d => {
        const isOnline = d.state === 'Connected' || d.state === 'Booted' || d.online;
        const color = isOnline ? '#10b981' : '#64748b';
        return `
          <div class="device-item">
            <span style="color:${color};font-weight:600">${isOnline ? '●' : '○'} ${this.escapeHtml(d.name)}</span>
            <span class="platform-tag ${(d.platform || '').toLowerCase()}">${d.platform}</span>
          </div>
        `;
      }).join('');
    }

    // External sessions
    if (this.activeSessions.length === 0) {
      this.externalSessionsList.innerHTML = '<div class="sidebar-empty">No external sessions</div>';
    } else {
      this.externalSessionsList.innerHTML = this.activeSessions.map(s => {
        return `
          <div class="session-item">
            <span>● ${this.escapeHtml(s.project_name)} (PID ${s.pid})</span>
            <span style="color:#06b6d4;font-size:11px">[${s.platform}]</span>
          </div>
        `;
      }).join('');
    }
  }

  renderFallbackTargets() {
    this.targets = [
      { id: 'MacLink', name: 'MacLink', platform: 'Macos', framework: 'swift', path: '/Users/as/Dev/Projects/opensources/mac-mtp' },
      { id: 'MacLinkCompanion', name: 'MacLinkCompanion', platform: 'Android', framework: 'kotlin', path: '/Users/as/Dev/Projects/opensources/mac-mtp/android' }
    ];
    this.devices = [
      { id: 'desktop-host', name: 'Local Desktop Host', platform: 'Desktop', state: 'Connected', online: true },
      { id: 'SM-A356E', name: 'Samsung Galaxy A35', platform: 'Android', state: 'Connected', online: true }
    ];
    this.renderTopBar('mac-mtp');
    this.renderSidebar();
    this.openSubProjectPane(this.targets[0]);
    this.openSubProjectPane(this.targets[1]);
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // Pane Management & Termax Engine Binding
  // ═══════════════════════════════════════════════════════════════════════════

  openSubProjectPane(target) {
    // Check if pane already open
    const existing = this.openPanes.find(p => p.targetId === target.id);
    if (existing) {
      existing.el.scrollIntoView({ behavior: 'smooth' });
      existing.el.style.borderColor = '#06b6d4';
      setTimeout(() => { existing.el.style.borderColor = ''; }, 1000);
      return;
    }

    const paneId = `pane-${target.id}`;
    const paneEl = document.createElement('div');
    paneEl.className = 'terminal-pane';
    paneEl.id = paneId;

    paneEl.innerHTML = `
      <div class="pane-header">
        <div class="pane-header-left">
          <span class="pane-title">${this.escapeHtml(target.name)}</span>
          <span class="platform-tag ${(target.platform || '').toLowerCase()}">${target.platform}</span>
          <span class="pane-status-pill idle" id="status-${target.id}">Idle</span>
          <span class="pane-build-time" id="buildtime-${target.id}"></span>
        </div>
        <div class="pane-header-right">
          <div class="pane-filter-group">
            <button class="filter-btn active" data-lvl="ALL">ALL</button>
            <button class="filter-btn" data-lvl="E">ERR</button>
            <button class="filter-btn" data-lvl="W">WRN</button>
            <button class="filter-btn" data-lvl="I">INF</button>
            <button class="filter-btn" data-lvl="D">DBG</button>
          </div>
          <input type="text" class="pane-search-input" placeholder="Search..." title="Regex search message / tag">
          <button class="btn-pane-action btn-run-target" title="Run / Stop Target">▶ Run</button>
          <button class="btn-pane-action btn-reload-target" title="Hot-reload">⚡</button>
          <button class="btn-pane-action btn-restart-target" title="Restart">🔄</button>
          <button class="btn-pane-action btn-clear-target" title="Clear console">Clear</button>
          <button class="btn-pane-close" title="Close Pane">✕</button>
        </div>
      </div>
      <div class="pane-terminal-body" id="body-${target.id}"></div>
    `;

    this.panesGrid.appendChild(paneEl);
    const terminalBody = paneEl.querySelector(`#body-${target.id}`);
    const engine = new TermaxLogPane(terminalBody, {
      targetId: target.id,
      targetName: target.name,
      platform: target.platform
    });

    const paneObj = {
      id: paneId,
      targetId: target.id,
      title: target.name,
      platform: target.platform,
      isCombined: false,
      engine,
      el: paneEl,
      target
    };

    this.openPanes.push(paneObj);
    this.attachPaneEvents(paneEl, paneObj);
    this.updateGridLayout();
    this.updateSidebarCardStatus();
  }

  openCombinedPane() {
    const existing = this.openPanes.find(p => p.isCombined);
    if (existing) {
      existing.el.scrollIntoView({ behavior: 'smooth' });
      return;
    }

    const paneId = 'pane-combined';
    const paneEl = document.createElement('div');
    paneEl.className = 'terminal-pane';
    paneEl.id = paneId;

    paneEl.innerHTML = `
      <div class="pane-header">
        <div class="pane-header-left">
          <span class="pane-title">🌐 All Workspace Targets (Combined Stream)</span>
          <span class="pane-status-pill running">Live Aggregator</span>
        </div>
        <div class="pane-header-right">
          <div class="pane-filter-group">
            <button class="filter-btn active" data-lvl="ALL">ALL</button>
            <button class="filter-btn" data-lvl="E">ERR</button>
            <button class="filter-btn" data-lvl="W">WRN</button>
            <button class="filter-btn" data-lvl="I">INF</button>
            <button class="filter-btn" data-lvl="D">DBG</button>
          </div>
          <input type="text" class="pane-search-input" placeholder="Search..." title="Regex search message / tag">
          <button class="btn-pane-action btn-clear-target" title="Clear console">Clear</button>
          <button class="btn-pane-close" title="Close Pane">✕</button>
        </div>
      </div>
      <div class="pane-terminal-body" id="body-combined"></div>
    `;

    this.panesGrid.appendChild(paneEl);
    const terminalBody = paneEl.querySelector('#body-combined');
    const engine = new TermaxLogPane(terminalBody, {
      targetId: 'all',
      targetName: 'All Targets'
    });

    const paneObj = {
      id: paneId,
      targetId: 'all',
      title: 'Combined Stream',
      platform: 'universal',
      isCombined: true,
      engine,
      el: paneEl
    };

    this.openPanes.push(paneObj);
    this.attachPaneEvents(paneEl, paneObj);
    this.updateGridLayout();
  }

  attachPaneEvents(paneEl, paneObj) {
    // Close button
    paneEl.querySelector('.btn-pane-close').addEventListener('click', () => {
      this.closePane(paneObj.id);
    });

    // Filter chips
    paneEl.querySelectorAll('.filter-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        paneEl.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        const lvl = btn.getAttribute('data-lvl');
        paneObj.engine.setLevelFilter(lvl);
      });
    });

    // Search input
    const searchInput = paneEl.querySelector('.pane-search-input');
    if (searchInput) {
      searchInput.addEventListener('input', (e) => {
        paneObj.engine.setSearchQuery(e.target.value);
      });
    }

    // Clear button
    const btnClear = paneEl.querySelector('.btn-clear-target');
    if (btnClear) {
      btnClear.addEventListener('click', () => {
        paneObj.engine.clear();
      });
    }

    // Target control buttons (if not combined)
    if (!paneObj.isCombined) {
      const btnRun = paneEl.querySelector('.btn-run-target');
      const btnReload = paneEl.querySelector('.btn-reload-target');
      const btnRestart = paneEl.querySelector('.btn-restart-target');

      if (btnRun) {
        btnRun.addEventListener('click', () => {
          this.toggleTargetRun(paneObj.target);
        });
      }
      if (btnReload) {
        btnReload.addEventListener('click', () => {
          this.reloadTarget(paneObj.target.id);
        });
      }
      if (btnRestart) {
        btnRestart.addEventListener('click', () => {
          this.restartTarget(paneObj.target.id);
        });
      }
    }
  }

  closePane(paneId) {
    const idx = this.openPanes.findIndex(p => p.id === paneId);
    if (idx !== -1) {
      const [removed] = this.openPanes.splice(idx, 1);
      removed.engine.destroy();
      removed.el.remove();
      this.updateGridLayout();
      this.updateSidebarCardStatus();
    }
  }

  clearAllPanes() {
    for (const p of this.openPanes) {
      p.engine.destroy();
      p.el.remove();
    }
    this.openPanes = [];
    this.updateGridLayout();
  }

  openAllSubProjects() {
    for (const target of this.targets) {
      this.openSubProjectPane(target);
    }
  }

  updateGridLayout() {
    const count = this.openPanes.length;
    this.panesGrid.className = 'panes-grid';

    if (count === 0) {
      this.viewportEmptyState.classList.remove('hidden');
      this.panesGrid.classList.add('hidden');
    } else {
      this.viewportEmptyState.classList.add('hidden');
      this.panesGrid.classList.remove('hidden');

      if (count === 1) {
        this.panesGrid.classList.add('panes-1');
      } else if (count === 2) {
        this.panesGrid.classList.add('panes-2');
      } else {
        this.panesGrid.classList.add('panes-3');
      }
    }
  }

  updateSidebarCardStatus() {
    this.subProjectsList.querySelectorAll('.subproject-card').forEach(card => {
      const tid = card.getAttribute('data-target-id');
      const isOpen = this.openPanes.some(p => p.targetId === tid);
      if (isOpen) {
        card.classList.add('active-in-pane');
      } else {
        card.classList.remove('active-in-pane');
      }
    });
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // Target Execution Actions
  // ═══════════════════════════════════════════════════════════════════════════

  async toggleTargetRun(target) {
    const statusPill = document.getElementById(`status-${target.id}`);
    const btnRun = document.querySelector(`#pane-${target.id} .btn-run-target`);

    // Find matched device
    const matchedDev = this.devices.find(d => {
      if (target.platform === 'Android') return d.platform === 'Android';
      if (target.platform === 'Macos' || target.platform === 'Desktop') return d.platform === 'Desktop';
      return true;
    });

    try {
      if (statusPill) {
        statusPill.className = 'pane-status-pill building';
        statusPill.textContent = 'Building...';
      }

      const res = await fetch(`${this.apiBase}/api/target/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          target_id: target.id,
          target_path: target.path,
          framework: target.framework,
          device_id: matchedDev ? matchedDev.id : null
        })
      });

      const data = await res.json();
      if (data.success) {
        if (statusPill) {
          statusPill.className = 'pane-status-pill running';
          statusPill.textContent = 'Running';
        }
        if (btnRun) {
          btnRun.textContent = '■ Stop';
        }
        this.updateDot(target.id, 'running');
      } else {
        if (statusPill) {
          statusPill.className = 'pane-status-pill error';
          statusPill.textContent = 'Error';
        }
        this.updateDot(target.id, 'error');
      }
    } catch (e) {
      console.error('Failed to start target:', e);
      if (statusPill) {
        statusPill.className = 'pane-status-pill error';
        statusPill.textContent = 'Error';
      }
    }
  }

  async runAllTargets() {
    for (const target of this.targets) {
      this.openSubProjectPane(target);
      this.toggleTargetRun(target);
    }
  }

  async reloadTarget(targetId) {
    try {
      await fetch(`${this.apiBase}/api/target/reload`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ target_id: targetId })
      });
    } catch (e) {
      console.error('Failed to reload target:', e);
    }
  }

  async restartTarget(targetId) {
    try {
      await fetch(`${this.apiBase}/api/target/restart`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ target_id: targetId })
      });
    } catch (e) {
      console.error('Failed to restart target:', e);
    }
  }

  async reloadAll() {
    try {
      await fetch(`${this.apiBase}/api/workspace/reload-all`, { method: 'POST' });
    } catch (e) {
      console.error('Failed to reload all:', e);
    }
  }

  async restartAll() {
    try {
      await fetch(`${this.apiBase}/api/workspace/restart-all`, { method: 'POST' });
    } catch (e) {
      console.error('Failed to restart all:', e);
    }
  }

  async stopAll() {
    for (const p of this.openPanes) {
      if (!p.isCombined) {
        try {
          await fetch(`${this.apiBase}/api/target/stop`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ target_id: p.targetId })
          });
          const statusPill = document.getElementById(`status-${p.targetId}`);
          if (statusPill) {
            statusPill.className = 'pane-status-pill idle';
            statusPill.textContent = 'Stopped';
          }
          this.updateDot(p.targetId, 'idle');
        } catch (e) {}
      }
    }
  }

  updateDot(targetId, statusClass) {
    const dot = document.getElementById(`dot-${targetId}`);
    if (dot) {
      dot.className = `target-status-dot ${statusClass}`;
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // SSE Real-Time Event Dispatcher
  // ═══════════════════════════════════════════════════════════════════════════

  connectEvents() {
    if (this.eventSource) {
      this.eventSource.close();
    }

    this.eventSource = new EventSource(`${this.apiBase}/api/events`);

    this.eventSource.onmessage = (msg) => {
      try {
        const evt = JSON.parse(msg.data);
        this.handleServerEvent(evt);
      } catch (e) {
        console.warn('Failed to parse SSE event:', e);
      }
    };

    this.eventSource.onerror = () => {
      // Reconnect automatically
    };
  }

  handleServerEvent(evt) {
    if (!evt || !evt.type) return;

    if (evt.type === 'LogAppended') {
      const { session_id, entry } = evt.payload || {};
      if (!entry) return;

      // Dispatch to matching target pane(s)
      for (const pane of this.openPanes) {
        if (!pane.isCombined && (pane.targetId === session_id || !session_id)) {
          pane.engine.appendLog(entry);
        } else if (pane.isCombined) {
          const taggedEntry = Object.assign({}, entry);
          if (!taggedEntry.tag && session_id) {
            taggedEntry.tag = session_id;
          }
          pane.engine.appendLog(taggedEntry);
        }
      }
    } else if (evt.type === 'BuildCompleted') {
      const { session_id, result } = evt.payload || {};
      if (session_id && result) {
        const buildTimeEl = document.getElementById(`buildtime-${session_id}`);
        const statusPill = document.getElementById(`status-${session_id}`);
        if (buildTimeEl) {
          buildTimeEl.textContent = `${result.duration_ms}ms`;
        }
        if (statusPill) {
          statusPill.className = result.success ? 'pane-status-pill running' : 'pane-status-pill error';
          statusPill.textContent = result.success ? 'Running' : 'Build Failed';
        }
      }
    }
  }

  async runDoctor() {
    this.doctorModal.classList.remove('hidden');
    this.doctorResults.innerHTML = '<p style="color:#06b6d4">Running diagnostics...</p>';

    try {
      const res = await fetch(`${this.apiBase}/api/doctor?dir=${encodeURIComponent(this.workspaceDir)}`);
      const report = await res.json();

      let html = `<div style="margin-bottom:12px;font-weight:600">Project: ${this.escapeHtml(report.project_path)}</div>`;
      for (const check of report.checks) {
        const isPass = check.status === 'passed';
        const color = isPass ? '#10b981' : (check.status === 'warning' ? '#f59e0b' : '#ef4444');
        const icon = isPass ? '✓' : '!';
        html += `
          <div style="margin-bottom:8px;padding:8px;background:rgba(255,255,255,0.04);border-radius:6px">
            <span style="color:${color};font-weight:bold">${icon} [${check.status.toUpperCase()}]</span>
            <span style="font-weight:600">${this.escapeHtml(check.name)}</span> — 
            <span style="color:#94a3b8">${this.escapeHtml(check.message)}</span>
            ${check.fix_hint ? `<div style="color:#38bdf8;font-size:11px;margin-top:4px">💡 Fix: ${this.escapeHtml(check.fix_hint)}</div>` : ''}
          </div>
        `;
      }
      this.doctorResults.innerHTML = html;
    } catch (e) {
      this.doctorResults.innerHTML = `<p style="color:#ef4444">Failed to run doctor: ${e.message}</p>`;
    }
  }

  async installCliInPath() {
    try {
      const res = await fetch(`${this.apiBase}/api/shell/install`, { method: 'POST' });
      const data = await res.json();
      if (data.success) {
        alert(`✓ ${data.message}\n\nYou can now run 'devflow' directly from VS Code or any terminal!`);
      } else {
        alert(`Failed to install CLI command: ${data.error}`);
      }
    } catch (e) {
      alert(`Installation request error: ${e.message}`);
    }
  }

  async refreshDevices() {
    try {
      const res = await fetch(`${this.apiBase}/api/devices`);
      this.devices = await res.json();
      this.renderTopBar(this.wsNameDisplay.textContent.split(' ')[0]);
      this.renderSidebar();
    } catch (e) {}
  }

  async bootEmulator(name) {
    try {
      const res = await fetch(`${this.apiBase}/api/devices/boot`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name })
      });
      const data = await res.json();
      if (data.success) {
        alert(`Emulator '${name}' booting successfully!`);
        this.refreshDevices();
      } else {
        alert(`Failed to boot emulator: ${data.error}`);
      }
    } catch (e) {
      alert(`Boot request failed: ${e.message}`);
    }
  }

  escapeHtml(str) {
    return String(str || '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }
}

document.addEventListener('DOMContentLoaded', () => {
  window.devFlowApp = new DevFlowApp();
});
