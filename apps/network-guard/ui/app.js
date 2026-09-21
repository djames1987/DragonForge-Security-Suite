(() => {
  "use strict";
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge Network Guard requires the Tauri desktop runtime.";
    return;
  }

  const titles = { connections: "Network visibility", dns: "DNS cache", about: "About Network Guard" };

  function showToast(message, error = false) {
    const toast = document.getElementById("toast");
    toast.hidden = false;
    toast.className = error ? "toast error" : "toast";
    toast.textContent = message;
    window.setTimeout(() => { toast.hidden = true; }, 3200);
  }

  function showView(name) {
    document.querySelectorAll(".view").forEach((item) => item.classList.remove("active"));
    document.querySelectorAll(".nav").forEach((item) => item.classList.remove("active"));
    document.getElementById(`view-${name}`).classList.add("active");
    document.querySelector(`.nav[data-view="${name}"]`)?.classList.add("active");
    document.getElementById("title").textContent = titles[name] ?? "Network Guard";
  }

  function endpoint(address, port) {
    if (!address) return "—";
    return port == null ? address : `${address}:${port}`;
  }

  function renderWarnings(warnings) {
    const host = document.getElementById("warnings");
    host.replaceChildren();
    for (const warning of warnings ?? []) {
      const item = document.createElement("div");
      item.className = "warning";
      item.textContent = warning;
      host.appendChild(item);
    }
  }

  function cell(text) {
    const td = document.createElement("td");
    td.textContent = String(text ?? "—");
    return td;
  }

  function renderConnections(rows) {
    const body = document.getElementById("connections");
    body.replaceChildren();
    for (const row of rows) {
      const tr = document.createElement("tr");
      if (row.wildcard_listener) tr.classList.add("attention");
      tr.append(
        cell(String(row.protocol).toUpperCase()),
        cell(row.process_name),
        cell(row.process_id),
        cell(endpoint(row.local_address, row.local_port)),
        cell(endpoint(row.remote_address, row.remote_port)),
        cell(row.state ?? (row.protocol === "udp" ? "Endpoint" : "—"))
      );
      body.appendChild(tr);
    }
  }

  function renderDns(rows) {
    const body = document.getElementById("dns");
    body.replaceChildren();
    for (const row of rows) {
      const tr = document.createElement("tr");
      tr.append(cell(row.name), cell(row.record_type), cell(row.data), cell(row.ttl_seconds ?? "—"));
      body.appendChild(tr);
    }
    document.getElementById("dns-count").textContent = `${rows.length} records`;
  }

  async function refresh() {
    const button = document.getElementById("refresh");
    button.disabled = true;
    button.textContent = "Refreshing…";
    try {
      const snapshot = await invoke("refresh_network_snapshot");
      document.getElementById("tcp").textContent = snapshot.summary.tcp_connections;
      document.getElementById("udp").textContent = snapshot.summary.udp_endpoints;
      document.getElementById("processes").textContent = snapshot.summary.processes;
      document.getElementById("wildcards").textContent = snapshot.summary.wildcard_listeners;
      document.getElementById("stamp").textContent = new Date(Number(snapshot.timestamp_ms)).toLocaleString();
      renderWarnings(snapshot.warnings);
      renderConnections(snapshot.connections);
      renderDns(snapshot.dns_cache);
      showToast("Network snapshot refreshed.");
    } catch (error) {
      showToast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Refresh snapshot";
    }
  }

  document.querySelectorAll(".nav").forEach((button) => {
    button.addEventListener("click", () => showView(button.dataset.view));
  });
  document.getElementById("refresh").addEventListener("click", refresh);
  refresh();
})();
