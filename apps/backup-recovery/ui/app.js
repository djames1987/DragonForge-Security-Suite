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
    recovery: "Suite recovery & migration",
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
    if (!window.confirm("Verify and restore this backup to the selected new destination? DragonForge will not overwrite an existing destination.")) return;
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


  function showRecoverySummary(targetId, summary, title) {
    const target = document.getElementById(targetId);
    target.hidden = false;
    target.replaceChildren();
    const heading = document.createElement("strong");
    heading.textContent = title;
    const details = document.createElement("p");
    details.textContent = `${summary.entries} files · ${formatBytes(summary.total_bytes)} · recovery format v${summary.format_version} · schema v${summary.schema_version}`;
    const migration = document.createElement("p");
    migration.textContent = summary.migrated
      ? `Schema migrated in memory from v${summary.original_schema_version} to v${summary.schema_version}.`
      : `Schema v${summary.schema_version} is current.`;
    const source = document.createElement("p");
    source.textContent = `Scope: ${summary.scope} · source platform: ${summary.source_platform} · suite version: ${summary.suite_version} · integrity: ${summary.verified ? "verified" : "metadata inspected"}`;
    target.append(heading, details, migration, source);
  }

  async function createRecovery() {
    const password = document.getElementById("recovery-create-password").value;
    if (password !== document.getElementById("recovery-create-confirm").value) {
      toast("Recovery passwords do not match.", true);
      return;
    }
    const button = document.getElementById("create-recovery");
    button.disabled = true;
    button.textContent = "Creating…";
    try {
      const summary = await invoke("create_recovery_package", {
        request: {
          destination: document.getElementById("recovery-create-path").value,
          password,
          scope: document.getElementById("recovery-scope").value,
        },
      });
      showRecoverySummary("recovery-create-result", summary, "Encrypted recovery package created");
      document.getElementById("recovery-create-password").value = "";
      document.getElementById("recovery-create-confirm").value = "";
      toast("Recovery package created and verified.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Create recovery package";
    }
  }

  async function inspectOrVerifyRecovery(verify) {
    const recoveryPath = document.getElementById("recovery-verify-path").value;
    const password = document.getElementById("recovery-verify-password").value;
    try {
      const summary = await invoke(
        verify ? "verify_recovery_package" : "inspect_recovery_package",
        { request: { recovery_path: recoveryPath, password } },
      );
      showRecoverySummary(
        "recovery-verify-result",
        summary,
        verify ? "Recovery package verified" : "Recovery package inspected",
      );
      toast(verify ? "Recovery package integrity verified." : "Recovery metadata decrypted.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function discoverRecoveryRoots() {
    try {
      const roots = await invoke("suite_recovery_roots");
      document.getElementById("recovery-config-root").value = roots.config_root;
      document.getElementById("recovery-data-root").value = roots.data_root;
      toast("Current DragonForge roots loaded. Clean restore still requires them to be empty.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function restoreRecovery() {
    if (!window.confirm("Verify and restore this recovery package into the selected clean DragonForge roots? Existing non-empty roots are refused.")) return;
    const button = document.getElementById("restore-recovery");
    button.disabled = true;
    button.textContent = "Restoring…";
    try {
      const dataRoot = document.getElementById("recovery-data-root").value.trim();
      const summary = await invoke("restore_recovery_package", {
        request: {
          recovery_path: document.getElementById("recovery-restore-path").value,
          password: document.getElementById("recovery-restore-password").value,
          config_root: document.getElementById("recovery-config-root").value,
          data_root: dataRoot || null,
        },
      });
      const target = document.getElementById("recovery-restore-result");
      target.hidden = false;
      target.textContent = `Restored ${summary.entries} files (${formatBytes(summary.total_bytes)}) to clean suite roots${summary.migrated_from_schema ? `; migrated from schema v${summary.migrated_from_schema}` : ""}.`;
      document.getElementById("recovery-restore-password").value = "";
      toast("Recovery package verified and restored.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Verify and restore to clean roots";
    }
  }

  async function repairSuiteState() {
    if (!window.confirm("Recover only valid JSON .bak files for missing or corrupt DragonForge state? Corrupt primaries are quarantined before replacement.")) return;
    const button = document.getElementById("repair-suite-state");
    button.disabled = true;
    try {
      const summary = await invoke("repair_suite_state");
      const target = document.getElementById("repair-result");
      target.hidden = false;
      target.textContent = `Inspected ${summary.inspected_backups} JSON backups · recovered missing: ${summary.recovered_missing} · recovered corrupt: ${summary.recovered_corrupt} · skipped invalid backups: ${summary.skipped_invalid_backups}.`;
      toast("Recoverable suite state repair completed.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
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
  document.getElementById("create-recovery").addEventListener("click", createRecovery);
  document.getElementById("inspect-recovery").addEventListener("click", () => inspectOrVerifyRecovery(false));
  document.getElementById("verify-recovery").addEventListener("click", () => inspectOrVerifyRecovery(true));
  document.getElementById("discover-recovery-roots").addEventListener("click", discoverRecoveryRoots);
  document.getElementById("restore-recovery").addEventListener("click", restoreRecovery);
  document.getElementById("repair-suite-state").addEventListener("click", repairSuiteState);
})();
