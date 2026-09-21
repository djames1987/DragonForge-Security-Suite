(() => {
  "use strict";

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    document.body.textContent = "DragonForge Authenticator requires the Tauri desktop runtime.";
    return;
  }

  const state = { accounts: [], unlocked: false };
  const titles = {
    accounts: "Authenticator accounts",
    add: "Add authenticator account",
    recovery: "Recovery codes",
    about: "About Authenticator",
  };

  const storePath = () => document.getElementById("store-path").value.trim();
  const password = () => document.getElementById("master-password").value;

  function toast(message, error = false) {
    const item = document.createElement("div");
    item.className = error ? "toast error" : "toast";
    item.textContent = message;
    document.getElementById("toast-region").appendChild(item);
    window.setTimeout(() => item.remove(), 3600);
  }

  function showView(name) {
    document.querySelectorAll(".view").forEach((view) => view.classList.remove("active"));
    document.querySelectorAll(".nav").forEach((button) => button.classList.remove("active"));
    document.getElementById(`view-${name}`).classList.add("active");
    document.querySelector(`.nav[data-view="${name}"]`)?.classList.add("active");
    document.getElementById("page-title").textContent = titles[name];
  }

  function kindLabel(kind) {
    return kind.type === "totp" ? `TOTP · ${kind.period}s` : `HOTP · counter ${kind.counter}`;
  }

  function accountCard(account) {
    const card = document.createElement("article");
    card.className = "account-card";

    const title = document.createElement("h3");
    title.textContent = account.label;
    const meta = document.createElement("div");
    meta.className = "account-meta";
    meta.textContent = [account.issuer, kindLabel(account.kind), account.algorithm.toUpperCase(), `${account.digits} digits`]
      .filter(Boolean).join(" · ");
    const code = document.createElement("div");
    code.className = "code";
    code.textContent = "••••••";
    const codeMeta = document.createElement("div");
    codeMeta.className = "code-meta";
    codeMeta.textContent = `${account.recovery_code_count} recovery code(s) stored`;

    const actions = document.createElement("div");
    actions.className = "card-actions";
    const generate = document.createElement("button");
    generate.className = "primary";
    generate.textContent = account.kind.type === "hotp" ? "Next HOTP" : "Show code";
    generate.addEventListener("click", async () => {
      try {
        const command = account.kind.type === "hotp" ? "next_hotp" : "current_code";
        const result = await invoke(command, { path: storePath(), password: password(), id: account.id });
        code.textContent = result.code;
        codeMeta.textContent = result.valid_for_seconds == null
          ? `Used counter ${result.counter}; next counter saved`
          : `Valid for ${result.valid_for_seconds}s`;
        if (account.kind.type === "hotp") {
          account.kind.counter = result.counter + 1;
          meta.textContent = [account.issuer, kindLabel(account.kind), account.algorithm.toUpperCase(), `${account.digits} digits`]
            .filter(Boolean).join(" · ");
        }
      } catch (error) {
        toast(String(error), true);
      }
    });

    const remove = document.createElement("button");
    remove.className = "danger";
    remove.textContent = "Delete";
    remove.addEventListener("click", async () => {
      if (!window.confirm(`Delete ${account.label}? This also removes its encrypted recovery codes.`)) return;
      try {
        await invoke("delete_account", { path: storePath(), password: password(), id: account.id });
        await unlock();
        toast("Authenticator account deleted.");
      } catch (error) {
        toast(String(error), true);
      }
    });

    actions.append(generate, remove);
    card.append(title, meta, code, codeMeta, actions);
    return card;
  }

  function renderAccounts() {
    const container = document.getElementById("accounts");
    if (!state.accounts.length) {
      container.replaceChildren(Object.assign(document.createElement("div"), {
        className: "empty",
        textContent: state.unlocked ? "No authenticator accounts yet." : "Unlock the encrypted store to view accounts.",
      }));
    } else {
      container.replaceChildren(...state.accounts.map(accountCard));
    }

    const selector = document.getElementById("recovery-account");
    selector.replaceChildren(...state.accounts.map((account) => {
      const option = document.createElement("option");
      option.value = account.id;
      option.textContent = account.issuer ? `${account.issuer} — ${account.label}` : account.label;
      return option;
    }));
  }

  async function unlock() {
    try {
      state.accounts = await invoke("unlock_accounts", { path: storePath(), password: password() });
      state.unlocked = true;
      const badge = document.getElementById("lock-state");
      badge.textContent = "Unlocked";
      badge.classList.add("unlocked");
      renderAccounts();
      return true;
    } catch (error) {
      state.unlocked = false;
      document.getElementById("lock-state").textContent = "Locked";
      document.getElementById("lock-state").classList.remove("unlocked");
      renderAccounts();
      toast(String(error), true);
      return false;
    }
  }

  function lockStore() {
    state.accounts = [];
    state.unlocked = false;
    document.getElementById("master-password").value = "";
    document.getElementById("recovery-codes").value = "";
    document.getElementById("otp-uri").value = "";
    document.getElementById("manual-secret").value = "";
    document.getElementById("new-password").value = "";
    document.getElementById("confirm-new-password").value = "";
    const badge = document.getElementById("lock-state");
    badge.textContent = "Locked";
    badge.classList.remove("unlocked");
    renderAccounts();
    toast("Authenticator locked.");
  }

  async function createStore() {
    const value = password();
    if (value.length < 12) {
      toast("Use a master password of at least 12 characters.", true);
      return;
    }
    try {
      await invoke("create_authenticator_store", { path: storePath(), password: value });
      toast("Encrypted Authenticator store created.");
      await unlock();
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function importUri(event) {
    event.preventDefault();
    if (!state.unlocked && !(await unlock())) return;
    try {
      await invoke("import_uri", {
        path: storePath(),
        password: password(),
        uri: document.getElementById("otp-uri").value,
      });
      document.getElementById("otp-uri").value = "";
      await unlock();
      showView("accounts");
      toast("Authenticator account imported.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function addManual(event) {
    event.preventDefault();
    if (!state.unlocked && !(await unlock())) return;
    try {
      await invoke("add_manual_account", {
        path: storePath(),
        password: password(),
        label: document.getElementById("manual-label").value,
        issuer: document.getElementById("manual-issuer").value,
        secret: document.getElementById("manual-secret").value,
        algorithm: document.getElementById("manual-algorithm").value,
        digits: Number(document.getElementById("manual-digits").value),
        kind: document.getElementById("manual-kind").value,
        periodOrCounter: Number(document.getElementById("period-counter").value),
      });
      document.getElementById("manual-secret").value = "";
      await unlock();
      showView("accounts");
      toast("Authenticator account added.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function revealRecovery() {
    const id = document.getElementById("recovery-account").value;
    if (!id) return toast("Select an account first.", true);
    try {
      const codes = await invoke("show_recovery_codes", { path: storePath(), password: password(), id });
      document.getElementById("recovery-codes").value = codes.join("\n");
      toast("Recovery codes revealed locally.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function saveRecovery() {
    const id = document.getElementById("recovery-account").value;
    if (!id) return toast("Select an account first.", true);
    const codes = document.getElementById("recovery-codes").value
      .split(/\r?\n/).map((value) => value.trim()).filter(Boolean);
    try {
      const count = await invoke("save_recovery_codes", {
        path: storePath(), password: password(), id, codes,
      });
      await unlock();
      toast(`${count} recovery code(s) saved encrypted.`);
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function changeMasterPassword(event) {
    event.preventDefault();
    if (!state.unlocked && !(await unlock())) return;
    const next = document.getElementById("new-password").value;
    const confirm = document.getElementById("confirm-new-password").value;
    if (next.length < 12) return toast("Use a new password of at least 12 characters.", true);
    if (next !== confirm) return toast("New passwords do not match.", true);

    try {
      await invoke("change_store_password", {
        path: storePath(),
        currentPassword: password(),
        newPassword: next,
      });
      document.getElementById("master-password").value = next;
      document.getElementById("new-password").value = "";
      document.getElementById("confirm-new-password").value = "";
      await unlock();
      toast("Authenticator store re-encrypted with the new password.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  async function initialize() {
    document.querySelectorAll(".nav").forEach((button) => {
      button.addEventListener("click", () => showView(button.dataset.view));
    });
    document.getElementById("unlock-button").addEventListener("click", unlock);
    document.getElementById("lock-button").addEventListener("click", lockStore);
    document.getElementById("create-button").addEventListener("click", createStore);
    document.getElementById("refresh-button").addEventListener("click", unlock);
    document.getElementById("uri-form").addEventListener("submit", importUri);
    document.getElementById("manual-form").addEventListener("submit", addManual);
    document.getElementById("reveal-recovery").addEventListener("click", revealRecovery);
    document.getElementById("save-recovery").addEventListener("click", saveRecovery);
    document.getElementById("password-change-form").addEventListener("submit", changeMasterPassword);
    document.getElementById("manual-kind").addEventListener("change", (event) => {
      const hotp = event.target.value === "hotp";
      document.getElementById("period-label").textContent = hotp ? "Initial counter" : "Period (seconds)";
      document.getElementById("period-counter").value = hotp ? "0" : "30";
    });

    try {
      const info = await invoke("default_store_path");
      document.getElementById("store-path").value = info.path;
      if (info.exists) toast("Existing encrypted Authenticator store detected.");
    } catch (error) {
      toast(String(error), true);
    }
  }

  initialize();
})();
