(() => {
  "use strict";

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge Integrity Monitor requires the Tauri desktop runtime.";
    return;
  }

  const state = {
    baseline: null,
  };

  const titles = {
    monitor: "Integrity baseline",
    about: "About Integrity Monitor",
  };

  function toast(message, error = false) {
    const item = document.createElement("div");
    item.className = error ? "toast error" : "toast";
    item.textContent = message;
    document.getElementById("toast-region").appendChild(item);
    window.setTimeout(() => item.remove(), 3400);
  }

  function showView(name) {
    document.querySelectorAll(".view").forEach((view) => view.classList.remove("active"));
    document.querySelectorAll(".nav").forEach((button) => button.classList.remove("active"));
    document.getElementById(`view-${name}`).classList.add("active");
    document.querySelector(`.nav[data-view="${name}"]`)?.classList.add("active");
    document.getElementById("page-title").textContent = titles[name] ?? "Integrity Monitor";
  }

  function formatTime(timestampMs) {
    if (!timestampMs) return "—";
    const date = new Date(Number(timestampMs));
    return Number.isNaN(date.valueOf()) ? "—" : date.toLocaleString();
  }

  function label(value) {
    return String(value ?? "")
      .replaceAll("_", " ")
      .replace(/w/g, (character) => character.toUpperCase());
  }

  function renderWarnings(warnings) {
    const region = document.getElementById("warnings");
    region.replaceChildren();
    for (const warning of warnings ?? []) {
      const item = document.createElement("div");
      item.className = "warning";
      item.textContent = warning;
      region.appendChild(item);
    }
  }

  function changeCard(change) {
    const card = document.createElement("article");
    card.className = "change";

    const top = document.createElement("div");
    top.className = "change-top";

    const surface = document.createElement("span");
    surface.className = "surface";
    surface.textContent = label(change.surface);

    const badge = document.createElement("span");
    badge.className = `badge kind-${change.kind}`;
    badge.textContent = change.kind;

    top.append(surface, badge);

    const key = document.createElement("h3");
    key.textContent = change.key;

    const copy = document.createElement("p");
    copy.textContent =
      change.kind === "added"
        ? "This item was not present in the saved baseline."
        : change.kind === "removed"
          ? "This baseline item is no longer present in the current snapshot."
          : "The item's fingerprint differs from the saved baseline.";

    card.append(top, key, copy);
    return card;
  }

  function renderComparison(report) {
    document.getElementById("metric-unchanged").textContent = report.summary.unchanged;
    document.getElementById("metric-added").textContent = report.summary.added;
    document.getElementById("metric-removed").textContent = report.summary.removed;
    document.getElementById("metric-changed").textContent = report.summary.changed;
    document.getElementById("check-time").textContent =
      `Checked ${formatTime(report.checked_at_ms)}`;

    renderWarnings(report.warnings);

    const changes = document.getElementById("changes");
    if (!report.changes.length) {
      changes.innerHTML = '<div class="empty">No differences were detected from the saved baseline.</div>';
      return;
    }
    changes.replaceChildren(...report.changes.map(changeCard));
  }

  async function loadBaselineStatus() {
    const status = await invoke("get_baseline_status");
    state.baseline = status;
    const checkButton = document.getElementById("check-button");
    const baselineButton = document.getElementById("baseline-button");

    if (status.exists) {
      document.getElementById("baseline-state").textContent = "Baseline ready";
      document.getElementById("baseline-copy").textContent =
        "Compare the current system fingerprints against the saved reference baseline.";
      document.getElementById("baseline-meta").textContent =
        `${status.entries} entries · created ${formatTime(status.created_at_ms)}`;
      baselineButton.textContent = "Replace baseline";
      checkButton.disabled = false;
    } else {
      document.getElementById("baseline-state").textContent = "No baseline yet";
      document.getElementById("baseline-copy").textContent =
        "Create an initial trusted reference point before monitoring for changes.";
      document.getElementById("baseline-meta").textContent = "No baseline is stored.";
      baselineButton.textContent = "Create baseline";
      checkButton.disabled = true;
    }
  }

  async function createOrReplaceBaseline() {
    const replace = Boolean(state.baseline?.exists);
    const button = document.getElementById("baseline-button");
    button.disabled = true;
    button.textContent = replace ? "Replacing…" : "Creating…";
    try {
      const summary = await invoke("create_integrity_baseline", { replace });
      renderWarnings(summary.warnings);
      toast(replace ? "Integrity baseline replaced." : "Integrity baseline created.");
      document.getElementById("changes").innerHTML =
        '<div class="empty">Baseline updated. Run an integrity check when you want to compare the current system.</div>';
      ["metric-unchanged", "metric-added", "metric-removed", "metric-changed"].forEach((id) => {
        document.getElementById(id).textContent = "—";
      });
      await loadBaselineStatus();
    } catch (error) {
      toast(String(error), true);
      await loadBaselineStatus();
    } finally {
      button.disabled = false;
    }
  }

  async function checkIntegrity() {
    const button = document.getElementById("check-button");
    button.disabled = true;
    button.textContent = "Checking…";
    try {
      const report = await invoke("check_integrity");
      renderComparison(report);
      const total = report.summary.added + report.summary.removed + report.summary.changed;
      toast(total ? `${total} integrity change(s) detected.` : "No integrity changes detected.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = !state.baseline?.exists;
      button.textContent = "Check integrity";
    }
  }

  async function initialize() {
    document.querySelectorAll(".nav").forEach((button) => {
      button.addEventListener("click", () => showView(button.dataset.view));
    });
    document.getElementById("baseline-button").addEventListener("click", createOrReplaceBaseline);
    document.getElementById("check-button").addEventListener("click", checkIntegrity);

    try {
      const info = await invoke("monitor_info");
      document.getElementById("platform-pill").textContent = info.platform_scope.replace("_", " ");
      await loadBaselineStatus();
    } catch (error) {
      document.getElementById("baseline-state").textContent = "Integrity Monitor unavailable";
      document.getElementById("baseline-copy").textContent = String(error);
      toast(String(error), true);
    }
  }

  initialize();
})();
