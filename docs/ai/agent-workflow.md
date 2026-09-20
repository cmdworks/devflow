# AI Agent Operational Runbook 🤖

This runbook guides autonomous AI coding agents (such as Google Antigravity, Cursor, Claude Desktop, and Roo-Code) on how to effectively operate, control, and develop applications using DevFlow.

---

## 🔁 The DevFlow AI Agent Loop

When working on a user's repository with DevFlow installed, follow this systematic loop:

```
                  ┌──────────────────────┐
                  │ 1. Discover & Check  │
                  │   devflow_doctor     │
                  │ devflow_list_devices │
                  └──────────┬───────────┘
                             │
                             ▼
                  ┌──────────────────────┐
                  │ 2. Start Session     │
                  │devflow_start_session │
                  └──────────┬───────────┘
                             │
                             ▼
┌───────────────► ┌──────────────────────┐
│                 │ 3. Read Live State   │
│                 │   devflow_get_logs   │
│                 │devflow_get_sess_state│
│                 └──────────┬───────────┘
│                            │
│                            ▼
│                 ┌──────────────────────┐
│                 │ 4. Apply Code Edit   │
│                 │ (Write source files) │
│                 └──────────┬───────────┘
│                            │
│                            ▼
│                 ┌──────────────────────┐
│                 │ 5. Trigger Reload    │
│                 │    devflow_reload    │
│                 │   (or auto-watcher)  │
│                 └──────────┬───────────┘
│                            │
└────────────────────────────┘
```

---

## 🛠️ Step-by-Step Tool Instructions for Agents

### Step 1: Discover Environment & Connected Targets
Before building or editing code, verify environment health and available devices:
- Call `devflow_doctor` with `project_path: "."` to discover detected frameworks, missing SDKs, and build tools.
- Call `devflow_list_devices` to discover connected Android physical devices, AVDs, Apple iOS simulators, and desktop hosts.

### Step 2: Start or Attach to a Development Session
- Call `devflow_start_session` with `project_path: "."`.
- DevFlow will automatically detect the framework (e.g. Kotlin Gradle, SwiftPM, Flutter, React Native, or Tauri), perform initial compilation, install to the device, launch the app, attach log streams, and activate file watching.
- DevFlow returns a unique `session_id`.

### Step 3: Inspect Logs & Diagnose Issues
- Call `devflow_get_logs` with `session_id` (or omit `session_id` — DevFlow will automatically use your active session).
- Use `levels: ["E", "W"]` to filter for errors and warnings.
- Use `query: "NullPointerException"` to search for specific crash traces.
- DevFlow automatically groups stack traces into clustered crash reports with frequency counts.

### Step 4: Apply Code Edits
- Modify project source files using your code editing tools.
- DevFlow's internal file watcher (`devflow_watcher`) will detect changes within 300ms.

### Step 5: Hot Reload or Restart
- For fast incremental updates (e.g. UI tweaks, function body edits): Call `devflow_reload`.
- For structural changes (e.g. AndroidManifest modifications, new dependencies, native package changes): Call `devflow_restart`.
- Validate that the app compiles and launches cleanly by checking `devflow_get_logs`.

---

## 💡 Best Practices for AI Agents

1. **Do not run manual shell polling scripts**: Never run `tail -f` or infinite bash `sleep` loops to inspect device logs. DevFlow maintains a high-speed $O(1)$ ring buffer accessible via `devflow_get_logs`.
2. **Omit `session_id` when in doubt**: DevFlow's **Smart Context Resolution** automatically maps tool calls to your active session.
3. **Check `devflow_doctor` on compilation failure**: If builds fail unexpectedly, invoke `devflow_doctor` to check SDK paths, Java versions, or missing toolchain binaries.
4. **Use structured JSON results**: All DevFlow MCP responses return clean, structured JSON in addition to human-readable summaries.
