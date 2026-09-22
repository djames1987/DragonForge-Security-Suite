(() => {
  "use strict";

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge Security Center requires the Tauri desktop runtime.";
    return;
  }

  const state = {
    snapshot: null,
    settings: null,
  };
  
  const titles = {
    overview: "Security overview",
    components: "Suite components",
    activity: "Activity history",
    settings: "Security Center settings",
    about: "About Security Center",
  };
  
  function escapeHtml(value) {
    return String(value ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#039;");
  }
  
  function toast(message, error = false) {
    const region = document.getElementById("toast-region");
    const item = document.createElement("div");
    item.className = error ? "toast error" : "toast";
    item.textContent = message;
    region.appendChild(item);
    window.setTimeout(() => item.remove(), 3200);
  }
  
  function showView(name) {
    document.querySelectorAll(".view").forEach((view) => view.classList.remove("active"));
    document.querySelectorAll(".nav-item").forEach((item) => item.classList.remove("active"));
    document.getElementById(`view-${name}`).classList.add("active");
    document.querySelector(`.nav-item[data-view="${name}"]`)?.classList.add("active");
    document.getElementById("page-title").textContent = titles[name] ?? "Security Center";
  
    if (name === "activity") {
      loadActivity();
    }
  }
  
  function componentInitials(name) {
    return name
      .split(/\s+/)
      .map((part) => part[0] ?? "")
      .join("")
      .slice(0, 2)
      .toUpperCase();
  }
  
  function componentCard(component) {
    const stateClass = `state-${escapeHtml(component.state)}`;
    const action =
      component.id === "agent"
        ? `<button class="component-action" data-launch="agent">${component.state === "active" ? "Restart DragonForge Agent" : "Start DragonForge Agent"}</button>`
        : component.id === "password-manager"
          ? '<button class="component-action" data-launch="password-manager">Open Password Manager</button>'
        : component.id === "file-vault"
          ? '<button class="component-action" data-launch="file-vault">Open File Vault</button>'
          : component.id === "authenticator"
            ? '<button class="component-action" data-launch="authenticator">Open Authenticator</button>'
            : component.id === "security-scanner"
              ? '<button class="component-action" data-launch="security-scanner">Open Security Scanner</button>'
              : component.id === "integrity-monitor"
                ? '<button class="component-action" data-launch="integrity-monitor">Open Integrity Monitor</button>'
                : component.id === "network-guard"
                  ? '<button class="component-action" data-launch="network-guard">Open Network Guard</button>'
                  : component.id === "backup-recovery"
                    ? '<button class="component-action" data-launch="backup-recovery">Open Backup & Recovery</button>'
                    : component.id === "secure-share"
                      ? '<button class="component-action" data-launch="secure-share">Open Secure Share</button>'
                      : "";
    return `
      <article class="component-card">
        <div class="component-card-top">
          <div class="component-icon">${escapeHtml(componentInitials(component.name))}</div>
          <span class="state-chip ${stateClass}">${escapeHtml(component.state_label)}</span>
        </div>
        <h3>${escapeHtml(component.name)}</h3>
        <p>${escapeHtml(component.description)}</p>
        <div class="component-detail">${escapeHtml(component.detail)}</div>
        ${action}
      </article>
    `;
  }
  
  function renderComponents(components) {
    const all = document.getElementById("all-components");
    const overview = document.getElementById("overview-components");
    all.innerHTML = components.map(componentCard).join("");
    overview.innerHTML = components.slice(0, 6).map(componentCard).join("");
    document.querySelectorAll('[data-launch="agent"]').forEach((button) => {
      button.addEventListener("click", startAgent);
    });
    document.querySelectorAll('[data-launch="password-manager"]').forEach((button) => {
      button.addEventListener("click", launchPasswordManager);
    });
    document.querySelectorAll('[data-launch="file-vault"]').forEach((button) => {
      button.addEventListener("click", launchFileVault);
    });
    document.querySelectorAll('[data-launch="authenticator"]').forEach((button) => {
      button.addEventListener("click", launchAuthenticator);
    });
    document.querySelectorAll('[data-launch="security-scanner"]').forEach((button) => {
      button.addEventListener("click", launchSecurityScanner);
    });
    document.querySelectorAll('[data-launch="integrity-monitor"]').forEach((button) => {
      button.addEventListener("click", launchIntegrityMonitor);
    });
    document.querySelectorAll('[data-launch="network-guard"]').forEach((button) => {
      button.addEventListener("click", launchNetworkGuard);
    });
    document.querySelectorAll('[data-launch="backup-recovery"]').forEach((button) => {
      button.addEventListener("click", launchBackupRecovery);
    });
    document.querySelectorAll('[data-launch="secure-share"]').forEach((button) => {
      button.addEventListener("click", launchSecureShare);
    });
  }
  
  function formatTime(timestampMs) {
    const date = new Date(Number(timestampMs));
    if (Number.isNaN(date.valueOf())) return "—";
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  
  function eventRows(events) {
    if (!events.length) {
      return '<div class="empty-message">No local activity events are currently retained.</div>';
    }
  
    return events.map((event) => `
      <div class="event-row">
        <time>${escapeHtml(formatTime(event.timestamp_ms))}</time>
        <div>
          <strong>${escapeHtml(event.summary)}</strong>
          <p>${escapeHtml(event.code)} · ${escapeHtml(event.component)}</p>
        </div>
        <span class="severity severity-${escapeHtml(event.severity)}" title="${escapeHtml(event.severity)}"></span>
      </div>
    `).join("");
  }
  
  function renderEvents(events) {
    document.getElementById("overview-events").innerHTML = eventRows(events.slice(0, 6));
  }
  
  function renderAgent(agent) {
    document.getElementById("agent-label").textContent = agent.label;
    document.getElementById("agent-detail").textContent = agent.detail;
    document.getElementById("sidebar-agent-label").textContent = agent.label;
    document.getElementById("sidebar-agent-detail").textContent = agent.detail;
    const startButton = document.getElementById("start-agent");
    const restartButton = document.getElementById("restart-agent");
    const stopButton = document.getElementById("stop-agent");
    if (startButton) {
      startButton.disabled = agent.available;
      startButton.textContent = agent.available ? "Agent running" : "Start Agent";
    }
    if (restartButton) restartButton.disabled = !agent.available;
    if (stopButton) stopButton.disabled = !agent.available;
  }
  
  function renderLastFailure(snapshot) {
    const target = document.getElementById("last-failure");
    if (!target) return;
    target.textContent = snapshot.last_failure || "No recorded crash/failure is currently available.";
  }

  function renderHealth(snapshot) {
    const health = snapshot.health;
    document.getElementById("health-label").textContent = health.label;
    document.getElementById("metric-active").textContent = health.active;
    document.getElementById("metric-integrated").textContent = health.integrated;
    document.getElementById("metric-planned").textContent = health.planned;
    document.getElementById("platform-pill").textContent = snapshot.platform;
    document.getElementById("health-copy").textContent =
      health.attention > 0
        ? `${health.attention} component(s) currently need attention.`
        : "Current components report no suite-level attention condition.";
  }
  
  function populateSettings(settings) {
    state.settings = settings;
    document.getElementById("setting-overview").checked = settings.start_on_overview;
    document.getElementById("setting-retention").value = String(settings.retain_event_count);
    document.getElementById("setting-identifiers").checked =
      settings.include_diagnostic_identifiers;
  }
  
  async function loadSnapshot() {
    const snapshot = await invoke("dashboard_snapshot");
    state.snapshot = snapshot;
    renderHealth(snapshot);
    renderComponents(snapshot.components);
    renderEvents(snapshot.events);
    renderAgent(snapshot.agent);
    renderLastFailure(snapshot);
    populateSettings(snapshot.settings);
  }
  
  async function refreshHealth() {
    const button = document.getElementById("refresh-button");
    button.disabled = true;
    button.textContent = "Refreshing…";
    try {
      const snapshot = await invoke("refresh_health");
      state.snapshot = snapshot;
      renderHealth(snapshot);
      renderComponents(snapshot.components);
      renderEvents(snapshot.events);
      renderAgent(snapshot.agent);
      renderLastFailure(snapshot);
      toast("Suite health refreshed.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Refresh health";
    }
  }
  
  async function startAgent() {
    try {
      const active = state.snapshot?.agent?.available;
      if (active) {
        await invoke("restart_agent");
        toast("DragonForge Agent restarted and reconnected.");
      } else {
        await invoke("launch_agent");
        toast("DragonForge Agent start requested.");
      }
      await new Promise((resolve) => window.setTimeout(resolve, 350));
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function restartAgent() {
    try {
      await invoke("restart_agent");
      toast("DragonForge Agent restarted and reconnected.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function stopAgent() {
    try {
      const status = await invoke("stop_agent");
      toast(status.available ? "Agent shutdown is still in progress." : "DragonForge Agent stopped gracefully.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchAuthenticator() {
    try {
      await invoke("launch_authenticator");
      toast("Authenticator launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchSecurityScanner() {
    try {
      await invoke("launch_security_scanner");
      toast("Security Scanner launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchIntegrityMonitor() {
    try {
      await invoke("launch_integrity_monitor");
      toast("Integrity Monitor launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchNetworkGuard() {
    try {
      await invoke("launch_network_guard");
      toast("Network Guard launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchSecureShare() {
    try {
      await invoke("launch_secure_share");
      toast("Secure Share launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchBackupRecovery() {
    try {
      await invoke("launch_backup_recovery");
      toast("Backup & Recovery launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchFileVault() {
    try {
      await invoke("launch_file_vault");
      toast("File Vault launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function launchPasswordManager() {
    try {
      await invoke("launch_password_manager");
      toast("Password Manager launch requested.");
      await loadSnapshot();
    } catch (error) {
      toast(String(error), true);
    }
  }
  
  async function loadActivity() {
    try {
      const events = await invoke("recent_events", { limit: 250 });
      document.getElementById("activity-events").innerHTML = eventRows(events);
    } catch (error) {
      toast(String(error), true);
    }
  }
  
  async function clearActivity() {
    try {
      await invoke("clear_events");
      await loadActivity();
      await loadSnapshot();
      toast("In-memory activity history cleared.");
    } catch (error) {
      toast(String(error), true);
    }
  }
  
  async function saveSettings(event) {
    event.preventDefault();
    const current = state.settings ?? {
      version: 1,
      start_on_overview: true,
      retain_event_count: 250,
      include_diagnostic_identifiers: false,
    };
    const settings = {
      ...current,
      start_on_overview: document.getElementById("setting-overview").checked,
      retain_event_count: Number(document.getElementById("setting-retention").value),
      include_diagnostic_identifiers: document.getElementById("setting-identifiers").checked,
    };
  
    const status = document.getElementById("settings-state");
    status.textContent = "Saving…";
    try {
      const saved = await invoke("save_settings", { settings });
      populateSettings(saved);
      status.textContent = "Saved locally.";
      await loadSnapshot();
      toast("Security Center settings saved.");
    } catch (error) {
      status.textContent = "Save failed.";
      toast(String(error), true);
    }
  }
  

  async function copyDiagnostics() {
    try {
      const report = await invoke("diagnostic_report");
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(report);
      } else {
        const area = document.createElement("textarea");
        area.value = report;
        area.setAttribute("readonly", "");
        area.style.position = "fixed";
        area.style.opacity = "0";
        document.body.appendChild(area);
        area.select();
        document.execCommand("copy");
        area.remove();
      }
      toast("Redaction-safe diagnostics copied.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function createSupportBundle() {
    try {
      const path = await invoke("create_support_bundle");
      toast(`Support bundle created: ${path}`);
    } catch (error) {
      toast(String(error), true);
    }
  }

  function wireNavigation() {
    document.querySelectorAll(".nav-item[data-view]").forEach((button) => {
      button.addEventListener("click", () => showView(button.dataset.view));
    });
    document.querySelectorAll("[data-go]").forEach((button) => {
      button.addEventListener("click", () => showView(button.dataset.go));
    });
  }
  
  async function initialize() {
    wireNavigation();
    document.getElementById("refresh-button").addEventListener("click", refreshHealth);
    document.getElementById("start-agent").addEventListener("click", startAgent);
    document.getElementById("restart-agent").addEventListener("click", restartAgent);
    document.getElementById("stop-agent").addEventListener("click", stopAgent);
    document.getElementById("clear-events").addEventListener("click", clearActivity);
    document.getElementById("settings-form").addEventListener("submit", saveSettings);
    document.getElementById("copy-diagnostics")?.addEventListener("click", copyDiagnostics);
    document.getElementById("create-support-bundle")?.addEventListener("click", createSupportBundle);
  
    try {
      await loadSnapshot();
      if (state.settings && !state.settings.start_on_overview) {
        showView("components");
      }
    } catch (error) {
      toast(`Security Center failed to load: ${String(error)}`, true);
      document.getElementById("health-label").textContent = "Dashboard unavailable";
      document.getElementById("health-copy").textContent =
        "Security Center could not initialize its local state.";
    }
  }
  
  initialize();
  
})();
