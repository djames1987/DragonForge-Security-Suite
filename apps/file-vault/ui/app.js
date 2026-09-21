(() => {
  "use strict";

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge File Vault requires the Tauri desktop runtime.";
    return;
  }

  const titles = {
    create: "Create encrypted vault",
    open: "Open encrypted vault",
    about: "About File Vault",
  };

  function toast(message, error = false) {
    const region = document.getElementById("toast-region");
    const item = document.createElement("div");
    item.className = error ? "toast error" : "toast";
    item.textContent = message;
    region.appendChild(item);
    window.setTimeout(() => item.remove(), 3500);
  }

  function showView(name) {
    document.querySelectorAll(".view").forEach((view) => view.classList.remove("active"));
    document.querySelectorAll(".nav").forEach((button) => button.classList.remove("active"));
    document.getElementById(`view-${name}`).classList.add("active");
    document.querySelector(`.nav[data-view="${name}"]`)?.classList.add("active");
    document.getElementById("page-title").textContent = titles[name];
  }

  function sourcesFromTextarea() {
    return document.getElementById("create-sources").value
      .split(/\r?\n/)
      .map((value) => value.trim())
      .filter(Boolean);
  }

  function formatBytes(value) {
    const bytes = Number(value);
    if (!Number.isFinite(bytes)) return "—";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KiB`;
    if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MiB`;
    return `${(bytes / 1024 ** 3).toFixed(2)} GiB`;
  }

  function renderSummary(summary) {
    const box = document.getElementById("vault-summary");
    box.classList.remove("hidden");
    box.innerHTML = [
      ["Entries", summary.entries],
      ["Files", summary.files],
      ["Folders", summary.directories],
      ["Data", formatBytes(summary.total_file_bytes)],
    ].map(([label, value]) => `
      <div class="summary-item">
        <strong>${String(value)}</strong>
        <span>${label}</span>
      </div>
    `).join("");
  }

  function renderEntries(entries) {
    document.getElementById("entry-count").textContent = `${entries.length} entries`;
    const list = document.getElementById("entry-list");
    if (!entries.length) {
      list.innerHTML = '<div class="empty">The vault contains no entries.</div>';
      return;
    }

    list.replaceChildren(...entries.map((entry) => {
      const row = document.createElement("div");
      row.className = "entry";

      const icon = document.createElement("div");
      icon.className = "entry-icon";
      icon.textContent = entry.is_directory ? "DIR" : "FILE";

      const path = document.createElement("div");
      path.className = "entry-path";
      path.textContent = entry.path;

      const size = document.createElement("div");
      size.className = "entry-size";
      size.textContent = entry.is_directory ? "Folder" : formatBytes(entry.size);

      row.append(icon, path, size);
      return row;
    }));
  }

  async function createContainer(event) {
    event.preventDefault();
    const sources = sourcesFromTextarea();
    const outputPath = document.getElementById("create-output").value.trim();
    const password = document.getElementById("create-password").value;
    const confirm = document.getElementById("create-confirm").value;
    const state = document.getElementById("create-state");

    if (!sources.length) {
      toast("Add at least one source file or folder.", true);
      return;
    }
    if (password.length < 12) {
      toast("Use a password of at least 12 characters.", true);
      return;
    }
    if (password !== confirm) {
      toast("Passwords do not match.", true);
      return;
    }

    state.textContent = "Encrypting…";
    try {
      const summary = await invoke("create_container", {
        outputPath,
        sourcePaths: sources,
        password,
      });
      state.textContent = `Created ${summary.files} encrypted file(s).`;
      document.getElementById("create-password").value = "";
      document.getElementById("create-confirm").value = "";
      toast("Encrypted File Vault created.");
    } catch (error) {
      state.textContent = "Creation failed.";
      toast(String(error), true);
    }
  }

  function currentOpenCredentials() {
    return {
      vaultPath: document.getElementById("open-vault").value.trim(),
      password: document.getElementById("open-password").value,
    };
  }

  async function inspectContainer() {
    try {
      const credentials = currentOpenCredentials();
      const entries = await invoke("inspect_container", credentials);
      renderEntries(entries);
      const summary = await invoke("verify_container", credentials);
      renderSummary(summary);
      toast("Vault authenticated and unlocked.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function verifyContainer() {
    try {
      const summary = await invoke("verify_container", currentOpenCredentials());
      renderSummary(summary);
      toast("Vault integrity verified.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function extractContainer() {
    const credentials = currentOpenCredentials();
    const destinationPath = document.getElementById("extract-destination").value.trim();
    try {
      const summary = await invoke("extract_container", {
        ...credentials,
        destinationPath,
      });
      renderSummary(summary);
      toast("Vault extracted successfully.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  document.querySelectorAll(".nav").forEach((button) => {
    button.addEventListener("click", () => showView(button.dataset.view));
  });
  document.getElementById("create-form").addEventListener("submit", createContainer);
  document.getElementById("inspect-button").addEventListener("click", inspectContainer);
  document.getElementById("verify-button").addEventListener("click", verifyContainer);
  document.getElementById("extract-button").addEventListener("click", extractContainer);
})();
