(() => {
  "use strict";

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge Security Scanner requires the Tauri desktop runtime.";
    return;
  }

  const titles = {
    scan: "Security posture scan",
    about: "About Security Scanner",
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
    document.getElementById("page-title").textContent = titles[name] ?? "Security Scanner";
  }

  function badge(text, className) {
    const item = document.createElement("span");
    item.className = `badge ${className}`;
    item.textContent = text;
    return item;
  }

  function findingCard(finding) {
    const card = document.createElement("article");
    card.className = "finding";

    const top = document.createElement("div");
    top.className = "finding-top";
    const category = document.createElement("span");
    category.className = "category";
    category.textContent = finding.category;
    const badges = document.createElement("div");
    badges.append(
      badge(finding.status, `status-${finding.status}`),
      badge(finding.severity, `severity-${finding.severity}`)
    );
    top.append(category, badges);

    const title = document.createElement("h3");
    title.textContent = finding.title;
    const summary = document.createElement("p");
    summary.textContent = finding.summary;

    const evidence = document.createElement("div");
    evidence.className = "detail";
    const evidenceLabel = document.createElement("strong");
    evidenceLabel.textContent = "Evidence";
    const evidenceText = document.createElement("span");
    evidenceText.textContent = finding.evidence;
    evidence.append(evidenceLabel, evidenceText);

    const recommendation = document.createElement("div");
    recommendation.className = "detail recommendation";
    const recommendationLabel = document.createElement("strong");
    recommendationLabel.textContent = "Recommendation";
    const recommendationText = document.createElement("span");
    recommendationText.textContent = finding.recommendation;
    recommendation.append(recommendationLabel, recommendationText);

    card.append(top, title, summary, evidence, recommendation);
    return card;
  }

  function renderReport(report) {
    document.getElementById("metric-pass").textContent = report.summary.pass;
    document.getElementById("metric-attention").textContent = report.summary.attention;
    document.getElementById("metric-unknown").textContent = report.summary.unknown;
    document.getElementById("metric-info").textContent = report.summary.informational;
    document.getElementById("platform-pill").textContent = report.platform;
    document.getElementById("scan-time").textContent =
      report.timestamp_ms ? new Date(report.timestamp_ms).toLocaleString() : "Scan completed";
    document.getElementById("scan-state").textContent =
      report.summary.attention > 0 ? "Review recommended" : "Scan complete";
    document.getElementById("scan-copy").textContent =
      report.summary.attention > 0
        ? `${report.summary.attention} finding(s) need review. This scanner does not change settings automatically.`
        : "No attention findings were produced by the checks that completed.";

    const findings = document.getElementById("findings");
    findings.replaceChildren(...report.findings.map(findingCard));
  }

  async function runScan() {
    const button = document.getElementById("scan-button");
    button.disabled = true;
    button.textContent = "Scanning…";
    document.getElementById("scan-state").textContent = "Scanning local posture…";
    try {
      const report = await invoke("run_scan");
      renderReport(report);
      toast("Security posture scan completed.");
    } catch (error) {
      document.getElementById("scan-state").textContent = "Scan unavailable";
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Run scan";
    }
  }

  async function initialize() {
    document.querySelectorAll(".nav").forEach((button) => {
      button.addEventListener("click", () => showView(button.dataset.view));
    });
    document.getElementById("scan-button").addEventListener("click", runScan);

    try {
      const info = await invoke("scanner_info");
      document.getElementById("platform-pill").textContent = info.platform_scope.replace("_", " ");
    } catch (error) {
      toast(String(error), true);
    }
  }

  initialize();
})();
