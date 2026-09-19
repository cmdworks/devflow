// DevFlow Desktop Companion App Logic

class DevflowClient {
  constructor() {
    this.serverUrl = 'http://localhost:9090';
    this.authToken = '';
    this.eventSource = null;
    this.rpcId = 1;
    this.activeSessionId = null;
    this.logs = [];
    this.currentLevelFilter = 'ALL';
    this.searchQuery = '';
    this.autoScroll = true;

    this.initElements();
    this.bindEvents();
    this.tryConnect();
  }

  initElements() {
    this.elServerUrl = document.getElementById('serverUrl');
    this.elAuthToken = document.getElementById('authToken');
    this.btnConnect = document.getElementById('btnConnect');
    this.statusBadge = document.getElementById('connectionStatus');
    this.devicesList = document.getElementById('devicesList');
    this.emulatorInput = document.getElementById('emulatorNameInput');
    this.btnBoot = document.getElementById('btnBootEmulator');
    this.btnRefreshDevices = document.getElementById('btnRefreshDevices');

    this.sessionStatusPill = document.getElementById('sessionStatusPill');
    this.sessionProject = document.getElementById('sessionProject');
    this.sessionFramework = document.getElementById('sessionFramework');
    this.sessionTarget = document.getElementById('sessionTarget');
    this.sessionBuildTime = document.getElementById('sessionBuildTime');
    this.reloadCount = document.getElementById('reloadCount');
    this.restartCount = document.getElementById('restartCount');

    this.btnReload = document.getElementById('btnReload');
    this.btnRestart = document.getElementById('btnRestart');
    this.btnBuild = document.getElementById('btnBuild');
    this.btnDoctor = document.getElementById('btnDoctor');

    this.crashBanner = document.getElementById('crashAlertBanner');
    this.crashType = document.getElementById('crashType');
    this.crashLocation = document.getElementById('crashLocation');
    this.crashPreview = document.getElementById('crashPreview');
    this.btnDismissCrash = document.getElementById('btnDismissCrash');
    this.btnCopyCrash = document.getElementById('btnCopyCrash');

    this.doctorModal = document.getElementById('doctorModal');
    this.doctorContent = document.getElementById('doctorContent');
    this.btnCloseDoctor = document.getElementById('btnCloseDoctor');

    this.logConsole = document.getElementById('logConsole');
    this.logSearchInput = document.getElementById('logSearchInput');
    this.autoScrollCheck = document.getElementById('autoScrollCheck');
    this.btnClearLogs = document.getElementById('btnClearLogs');
    this.filterBtns = document.querySelectorAll('.filter-btn');
  }

  bindEvents() {
    this.btnConnect.addEventListener('click', () => {
      this.serverUrl = this.elServerUrl.value.trim().replace(/\/$/, '');
      this.authToken = this.elAuthToken.value.trim();
      this.connect();
    });

    this.btnRefreshDevices.addEventListener('click', () => this.fetchDevices());
    this.btnBoot.addEventListener('click', () => this.bootEmulator());

    this.btnReload.addEventListener('click', () => this.triggerReload());
    this.btnRestart.addEventListener('click', () => this.triggerRestart());
    this.btnBuild.addEventListener('click', () => this.triggerBuild());
    this.btnDoctor.addEventListener('click', () => this.runDoctor());

    this.btnDismissCrash.addEventListener('click', () => this.crashBanner.classList.add('hidden'));
    this.btnCopyCrash.addEventListener('click', () => {
      navigator.clipboard.writeText(`${this.crashType.textContent}\n${this.crashLocation.textContent}\n${this.crashPreview.textContent}`);
      this.btnCopyCrash.textContent = 'Copied!';
      setTimeout(() => { this.btnCopyCrash.textContent = 'Copy Report'; }, 1500);
    });

    this.btnCloseDoctor.addEventListener('click', () => this.doctorModal.classList.add('hidden'));

    this.filterBtns.forEach(btn => {
      btn.addEventListener('click', (e) => {
        this.filterBtns.forEach(b => b.classList.remove('active'));
        e.target.classList.add('active');
        this.currentLevelFilter = e.target.dataset.level;
        this.renderLogs();
      });
    });

    this.logSearchInput.addEventListener('input', (e) => {
      this.searchQuery = e.target.value.toLowerCase();
      this.renderLogs();
    });

    this.autoScrollCheck.addEventListener('change', (e) => {
      this.autoScroll = e.target.checked;
    });

    this.btnClearLogs.addEventListener('click', () => {
      this.logs = [];
      this.logConsole.innerHTML = '';
    });
  }

  tryConnect() {
    this.serverUrl = this.elServerUrl.value.trim().replace(/\/$/, '');
    this.authToken = this.elAuthToken.value.trim();
    this.connect();
  }

  connect() {
    if (this.eventSource) {
      this.eventSource.close();
    }

    this.updateStatus(false, 'Connecting...');

    let sseUrl = `${this.serverUrl}/events`;
    if (this.authToken) {
      sseUrl += `?token=${encodeURIComponent(this.authToken)}`;
    }

    try {
      this.eventSource = new EventSource(sseUrl);

      this.eventSource.onopen = () => {
        this.updateStatus(true, 'Connected');
        this.fetchDevices();
      };

      this.eventSource.addEventListener('connected', () => {
        this.updateStatus(true, 'Connected');
        this.fetchDevices();
      });

      this.eventSource.addEventListener('devflow_event', (e) => {
        try {
          const event = JSON.parse(e.data);
          this.handleDevflowEvent(event);
        } catch (err) {
          console.error('Failed to parse SSE event:', err);
        }
      });

      this.eventSource.onerror = () => {
        this.updateStatus(false, 'Disconnected');
      };
    } catch (e) {
      this.updateStatus(false, 'Error');
    }
  }

  updateStatus(connected, label) {
    if (connected) {
      this.statusBadge.className = 'status-badge connected';
      this.statusBadge.querySelector('.status-label').textContent = label;
    } else {
      this.statusBadge.className = 'status-badge disconnected';
      this.statusBadge.querySelector('.status-label').textContent = label;
    }
  }

  async callRpc(method, params = {}) {
    const payload = {
      jsonrpc: '2.0',
      id: this.rpcId++,
      method,
      params,
    };

    const headers = { 'Content-Type': 'application/json' };
    if (this.authToken) {
      headers['Authorization'] = `Bearer ${this.authToken}`;
    }

    try {
      const res = await fetch(`${this.serverUrl}/rpc`, {
        method: 'POST',
        headers,
        body: JSON.stringify(payload),
      });

      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: ${res.statusText}`);
      }

      const data = await res.json();
      if (data.error) {
        throw new Error(data.error.message);
      }
      return data.result;
    } catch (err) {
      this.appendLog({
        timestamp: new Date().toISOString(),
        level: 'E',
        tag: 'DevFlowRPC',
        message: `RPC Error (${method}): ${err.message}`
      });
      throw err;
    }
  }

  async fetchDevices() {
    try {
      const res = await this.callRpc('tools/call', {
        name: 'devflow_list_devices',
        arguments: {}
      });

      const devices = (res && res.devices) ? res.devices : [];
      this.renderDevices(devices);
    } catch (e) {
      console.warn('Could not fetch devices:', e);
    }
  }

  renderDevices(devices) {
    this.devicesList.innerHTML = '';
    if (!devices || devices.length === 0) {
      this.devicesList.innerHTML = '<div class="empty-state">No devices or emulators found</div>';
      return;
    }

    devices.forEach(d => {
      const item = document.createElement('div');
      item.className = 'device-item';
      item.innerHTML = `
        <div class="device-info">
          <span class="device-name">${d.name} ${d.is_default ? '⭐' : ''}</span>
          <span class="device-meta">${d.platform} • ${d.state} ${d.is_emulator ? '[AVD]' : ''}</span>
        </div>
      `;
      item.addEventListener('click', () => {
        document.querySelectorAll('.device-item').forEach(el => el.classList.remove('active'));
        item.classList.add('active');
        this.sessionTarget.textContent = d.name;
      });
      this.devicesList.appendChild(item);
    });
  }

  async bootEmulator() {
    const name = this.emulatorInput.value.trim();
    if (!name) return;
    this.appendLog({
      timestamp: new Date().toISOString(),
      level: 'I',
      tag: 'Emulator',
      message: `Booting emulator '${name}'...`
    });

    try {
      await this.callRpc('tools/call', {
        name: 'devflow_boot_emulator',
        arguments: { name }
      });
      this.emulatorInput.value = '';
      setTimeout(() => this.fetchDevices(), 2000);
    } catch (e) {
      // Handled in callRpc
    }
  }

  async triggerReload() {
    this.appendLog({
      timestamp: new Date().toISOString(),
      level: 'I',
      tag: 'Action',
      message: 'Triggering framework reload...'
    });
    try {
      await this.callRpc('tools/call', {
        name: 'devflow_reload',
        arguments: { session_id: this.activeSessionId || 'default' }
      });
    } catch (e) {}
  }

  async triggerRestart() {
    this.appendLog({
      timestamp: new Date().toISOString(),
      level: 'I',
      tag: 'Action',
      message: 'Triggering full application restart...'
    });
    try {
      await this.callRpc('tools/call', {
        name: 'devflow_restart',
        arguments: { session_id: this.activeSessionId || 'default' }
      });
    } catch (e) {}
  }

  async triggerBuild() {
    this.appendLog({
      timestamp: new Date().toISOString(),
      level: 'I',
      tag: 'Action',
      message: 'Triggering one-off build...'
    });
    try {
      const res = await this.callRpc('tools/call', {
        name: 'devflow_build',
        arguments: { project_path: '.' }
      });
      if (res) {
        this.appendLog({
          timestamp: new Date().toISOString(),
          level: res.success ? 'I' : 'E',
          tag: 'Build',
          message: res.success ? `Build succeeded in ${res.duration_ms}ms` : `Build failed: ${res.error_message}`
        });
      }
    } catch (e) {}
  }

  async runDoctor() {
    this.doctorModal.classList.remove('hidden');
    this.doctorContent.innerHTML = '<p>Running environment diagnostics...</p>';
    try {
      const res = await this.callRpc('tools/call', {
        name: 'devflow_doctor',
        arguments: { project_path: '.' }
      });

      if (res && res.checks) {
        let html = `<p style="margin-bottom: 8px;"><strong>Summary:</strong> ${res.passed_count} passed, ${res.warning_count} warnings, ${res.failure_count} failed</p>`;
        html += '<ul style="list-style: none; display: flex; flex-direction: column; gap: 6px;">';
        res.checks.forEach(c => {
          const color = c.status === 'passed' ? 'var(--accent-green)' : (c.status === 'warning' ? 'var(--accent-yellow)' : 'var(--accent-red)');
          html += `
            <li style="padding: 6px; background: var(--bg-tertiary); border-radius: var(--radius-sm); border-left: 3px solid ${color};">
              <strong>${c.name}</strong>: ${c.message}
              ${c.fix_hint ? `<div style="font-size: 0.75rem; color: var(--text-muted); margin-top: 2px;">Fix: ${c.fix_hint}</div>` : ''}
            </li>
          `;
        });
        html += '</ul>';
        this.doctorContent.innerHTML = html;
      }
    } catch (e) {
      this.doctorContent.innerHTML = `<p style="color: var(--accent-red);">Doctor check failed: ${e.message}</p>`;
    }
  }

  handleDevflowEvent(evt) {
    if (!evt) return;

    if (evt.type === 'SessionStateChanged' && evt.payload) {
      const p = evt.payload;
      this.activeSessionId = p.session_id;
      this.sessionStatusPill.textContent = p.status || 'Running';
      this.sessionStatusPill.className = `pill pill-${(p.status || 'running').toLowerCase()}`;

      if (p.state) {
        this.sessionProject.textContent = p.state.project_name || '—';
        this.sessionFramework.textContent = p.state.framework || '—';
        this.reloadCount.textContent = p.state.reload_count || 0;
        this.restartCount.textContent = p.state.restart_count || 0;
        if (p.state.build_duration_ms) {
          this.sessionBuildTime.textContent = `${p.state.build_duration_ms}ms`;
        }
      }
    } else if (evt.type === 'LogAppended' && evt.payload) {
      this.appendLog(evt.payload.entry);
    } else if (evt.type === 'BuildCompleted' && evt.payload) {
      const b = evt.payload.result;
      if (b) {
        this.sessionBuildTime.textContent = `${b.duration_ms}ms`;
      }
    }
  }

  appendLog(entry) {
    if (!entry) return;
    this.logs.push(entry);

    // Check for crash
    if (entry.level === 'E' && (entry.message.includes('Exception') || entry.message.includes('FATAL') || entry.message.includes('Crash'))) {
      this.showCrashAlert(entry);
    }

    if (this.shouldDisplayLog(entry)) {
      this.renderLogLine(entry);
      if (this.autoScroll) {
        this.logConsole.scrollTop = this.logConsole.scrollHeight;
      }
    }
  }

  showCrashAlert(entry) {
    this.crashType.textContent = entry.tag ? `Crash: ${entry.tag}` : 'Fatal Exception Detected';
    this.crashLocation.textContent = entry.timestamp;
    this.crashPreview.textContent = entry.message;
    this.crashBanner.classList.remove('hidden');
  }

  shouldDisplayLog(entry) {
    if (this.currentLevelFilter !== 'ALL' && entry.level !== this.currentLevelFilter) {
      return false;
    }
    if (this.searchQuery) {
      const text = `${entry.tag || ''} ${entry.message || ''}`.toLowerCase();
      if (!text.includes(this.searchQuery)) {
        return false;
      }
    }
    return true;
  }

  renderLogs() {
    this.logConsole.innerHTML = '';
    const filtered = this.logs.filter(l => this.shouldDisplayLog(l));
    filtered.forEach(l => this.renderLogLine(l));
    if (this.autoScroll) {
      this.logConsole.scrollTop = this.logConsole.scrollHeight;
    }
  }

  renderLogLine(entry) {
    const line = document.createElement('div');
    line.className = 'log-line';

    const level = (entry.level || 'I').toUpperCase();
    const badgeClass = `badge-${level.toLowerCase()}`;
    const timeStr = entry.timestamp ? entry.timestamp.split('T')[1]?.slice(0, 8) || entry.timestamp : '00:00:00';

    line.innerHTML = `
      <span class="log-time">[${timeStr}]</span>
      <span class="log-badge ${badgeClass}">${level}</span>
      <span class="log-tag">[${entry.tag || 'App'}]</span>
      <span class="log-msg">${escapeHtml(entry.message || '')}</span>
    `;

    this.logConsole.appendChild(line);
  }
}

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

window.addEventListener('DOMContentLoaded', () => {
  new DevflowClient();
});
