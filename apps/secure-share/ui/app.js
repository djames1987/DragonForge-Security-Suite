(() => {
  "use strict";

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge Secure Share requires the Tauri desktop runtime.";
    return;
  }

  const titles = {
    create: "Create secure share",
    open: "Verify or reveal secure share",
    extract: "Extract secure attachments",
    about: "About Secure Share",
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
    document.getElementById("title").textContent = titles[name] ?? "Secure Share";
  }

  function pathLines(id) {
    return document.getElementById(id).value
      .split(/\r?\n/)
      .map((value) => value.trim())
      .filter(Boolean);
  }

  function expirationMs() {
    const raw = document.getElementById("expires").value;
    if (!raw) return 0;
    const value = new Date(raw);
    return Number.isNaN(value.valueOf()) ? 0 : value.getTime();
  }

  function formatBytes(value) {
    const bytes = Number(value ?? 0);
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  }

  function formatDate(ms) {
    const date = new Date(Number(ms));
    return Number.isNaN(date.valueOf()) ? "Unknown" : date.toLocaleString();
  }

  function renderSummary(targetId, summary, heading) {
    const target = document.getElementById(targetId);
    target.hidden = false;
    target.replaceChildren();
    const title = document.createElement("strong");
    title.textContent = heading;
    const metadata = document.createElement("p");
    metadata.textContent = `From: ${summary.sender_label || "Unspecified"} · To: ${summary.recipient_label}`;
    const content = document.createElement("p");
    content.textContent = `${summary.attachment_count} attachment file(s), ${formatBytes(summary.total_attachment_bytes)} · Secret: ${summary.has_secret ? "yes" : "no"}`;
    const expiration = document.createElement("p");
    expiration.textContent = `Expires: ${formatDate(summary.expires_at_ms)} · ${summary.expired ? "EXPIRED" : "Active"} · Integrity: ${summary.verified ? "verified" : "not verified"}`;
    target.append(title, metadata, content, expiration);
  }

  async function createShare() {
    const password = document.getElementById("create-password").value;
    if (password !== document.getElementById("create-confirm").value) {
      toast("Package passwords do not match.", true);
      return;
    }
    const expires = expirationMs();
    if (!expires) {
      toast("Choose a valid expiration date and time.", true);
      return;
    }

    const button = document.getElementById("create-share");
    button.disabled = true;
    button.textContent = "Creating…";
    try {
      const secret = document.getElementById("secret").value;
      const summary = await invoke("create_secure_share", {
        request: {
          sender_label: document.getElementById("sender").value,
          recipient_label: document.getElementById("recipient").value,
          expires_at_ms: expires,
          secret_text: secret ? secret : null,
          attachment_paths: pathLines("attachments"),
          destination: document.getElementById("create-destination").value,
          password,
        },
      });
      renderSummary("create-result", summary, "Encrypted share created and verified");
      document.getElementById("create-password").value = "";
      document.getElementById("create-confirm").value = "";
      toast("Secure share created.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Create encrypted share";
    }
  }

  async function verifyShare() {
    try {
      const summary = await invoke("verify_secure_share", {
        request: {
          share_path: document.getElementById("open-path").value,
          password: document.getElementById("open-password").value,
        },
      });
      renderSummary("open-result", summary, "Package integrity verified");
      document.getElementById("secret-result").hidden = true;
      toast("Secure share verified.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function revealSecret() {
    try {
      const result = await invoke("reveal_secure_share_secret", {
        request: {
          share_path: document.getElementById("open-path").value,
          password: document.getElementById("open-password").value,
        },
      });
      renderSummary("open-result", result.summary, "Package verified");
      const secret = document.getElementById("secret-result");
      secret.hidden = false;
      secret.textContent = result.secret_text ?? "This share contains no protected secret text.";
      toast("Secret reveal completed.");
    } catch (error) {
      document.getElementById("secret-result").hidden = true;
      toast(String(error), true);
    }
  }

  async function extractShare() {
    if (!window.confirm("Verify and extract these attachments to the selected new destination? Existing destinations are not overwritten.")) return;
    const button = document.getElementById("extract-share");
    button.disabled = true;
    button.textContent = "Extracting…";
    try {
      const summary = await invoke("extract_secure_share_attachments", {
        request: {
          share_path: document.getElementById("extract-path").value,
          destination: document.getElementById("extract-destination").value,
          password: document.getElementById("extract-password").value,
        },
      });
      renderSummary("extract-result", summary, "Attachments verified and extracted");
      document.getElementById("extract-password").value = "";
      toast("Secure attachments extracted.");
    } catch (error) {
      toast(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Verify and extract attachments";
    }
  }

  document.querySelectorAll(".nav").forEach((button) => {
    button.addEventListener("click", () => showView(button.dataset.view));
  });
  document.getElementById("create-share").addEventListener("click", createShare);
  document.getElementById("verify-share").addEventListener("click", verifyShare);
  document.getElementById("reveal-secret").addEventListener("click", revealSecret);
  document.getElementById("extract-share").addEventListener("click", extractShare);
})();
