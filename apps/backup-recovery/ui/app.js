(() => {
  "use strict";

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge Backup & Recovery requires the Tauri desktop runtime.";
    return;
  }

  const titles = {
    create: "Create encrypted backup",
    verify: "Verify or inspect backup",
    restore: "Restore backup",
    about: "About Backup & Recovery",
  };

  function toast(message, error = false) {
    const host = document.getElementById("toast");
    host.hidden = false;
    host.className = error ? "toast error" : "toast";
    host.textContent = message;
    window.setTimeout(() => { host.hidden = true; }, 3600);
  }

  function showView(name) {
    document.querySelectorAll(".view").forEach((view) => view.classList.remove("active"));
    document.querySelectorAll(".nav").forEach((button) => button.classList.remove("active"));
    document.getElementById(`view-${name}`).classList.add("active");
    document.querySelector(`.nav[data-view="${name}"]`)?.classList.add("active");
    document.getElementById("title").textContent = titles[name] ?? "Backup & Recovery";
  }

  function sourceLines() {
    return document.getElementById("sources").value
      .split(/\r?\n/)
      .map((value) => value.trim())
      .filter(Boolean);
  }

  function formatBytes(value) {
    const bytes = Number(value ?? 0);
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  }

  function showSummary(targetId, summary, title) {
    const target = document.getElementById(targetId);
    target.hidden = false;
    target.replaceChildren();
    const heading = document.createElement("strong");
    heading.textContent = title;
    const details = document.createElement("p");
    details.textContent = `${summary.entries} files · ${formatBytes(summary.total_bytes)} · format v${summary.format_version}`;
    const status = document.createElement("p");
    status.textContent = summary.verified ? "Integrity: verified" : "Integrity: not fully verified (metadata decrypted only)";
    target.append(heading, details, status);
  }

  async function discover() {
    try {
      const sources = await invoke("suite_sources");
      const host = document.getElementById("discovered");
      host.replaceChildren();
      const existing = [];
      for (const source of sources) {
        const row = document.createElement("div");
        row.className = "source-row";
        const label = document.createElement("span");
        label.textContent = `${source.label}: ${source.path}`;
        const status = document.createElement("strong");
        status.textContent = source.exists ? "Available" : "Not found";
        row.append(label, status);
        host.appendChild(row);
        if (source.exists) existing.push(source.path);
      }
      if (existing.length) {
        const current = new Set(sourceLines());
        for (const path of existing) current.add(path);
        document.getElementById("sources").value = [...current].join("\n");
      }
      toast("Suite data discovery completed.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function createBackup() {
    const password = document.getElementById("create-password").value;
    if (password !== document.getElementById("create-confirm").value) {
      toast("Backup passwords do not match.", true);
      return;
    }
    const button = document.getElementById("create-backup");
    button.disabled = true;
    button.textContent = "Creating…";
    try {
      const summary = await invoke("create_encrypted_backup", {
        request: {
          source_paths: sourceLines(),
          destination: document.getElementById("create-destination").value,
          password,
        },
      });
      showSummary("create-result", summary, "Encrypted backup created");
      document.getElementById("create-password").value = "";
      document.getElementById("create-confirm").value = "";
      toast("Encrypted backup created.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Create encrypted backup";
    }
  }

  async function inspectOrVerify(verify) {
    const path = document.getElementById("verify-path").value;
    const password = document.getElementById("verify-password").value;
    try {
      const summary = await invoke(verify ? "verify_encrypted_backup" : "inspect_encrypted_backup", {
        request: { backup_path: path, password },
      });
      showSummary("verify-result", summary, verify ? "Backup integrity verified" : "Backup metadata inspected");
      toast(verify ? "Backup integrity verified." : "Backup metadata decrypted.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function restore() {
    const button = document.getElementById("restore-backup");
    button.disabled = true;
    button.textContent = "Restoring…";
    try {
      const summary = await invoke("restore_encrypted_backup", {
        request: {
          backup_path: document.getElementById("restore-path").value,
          destination: document.getElementById("restore-destination").value,
          password: document.getElementById("restore-password").value,
        },
      });
      const target = document.getElementById("restore-result");
      target.hidden = false;
      target.textContent = `Restored ${summary.entries} files (${formatBytes(summary.total_bytes)}) to ${summary.destination}`;
      document.getElementById("restore-password").value = "";
      toast("Backup verified and restored.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Verify and restore";
    }
  }

  document.querySelectorAll(".nav").forEach((button) => {
    button.addEventListener("click", () => showView(button.dataset.view));
  });
  document.getElementById("discover").addEventListener("click", discover);
  document.getElementById("create-backup").addEventListener("click", createBackup);
  document.getElementById("inspect-backup").addEventListener("click", () => inspectOrVerify(false));
  document.getElementById("verify-backup").addEventListener("click", () => inspectOrVerify(true));
  document.getElementById("restore-backup").addEventListener("click", restore);
})();
