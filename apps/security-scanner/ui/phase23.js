(() => {
  "use strict";

  const strings = Object.freeze({
    skip: "Skip to main content",
    firstRunTitle: "DragonForge privacy & accessibility",
    firstRunBody:
      "DragonForge is local-first. Diagnostics and support bundles stay on this computer unless you explicitly export them. Network features such as update checks and Password Manager sync run only through their dedicated controls.",
    firstRunA11y:
      "Keyboard navigation, visible focus, reduced-motion preferences, Windows high-contrast/forced-colors mode, and screen-reader status announcements are supported across the desktop suite.",
    firstRunContinue: "Continue",
    privacyLabel: "Privacy guidance",
  });

  const t = (key) => strings[key] ?? key;
  const main =
    document.querySelector("main") ||
    document.querySelector(".content") ||
    document.querySelector(".main-content") ||
    document.querySelector("[role='main']");

  document.documentElement.lang ||= "en";
  document.documentElement.dataset.locale = "en-US";
  document.documentElement.dataset.l10nReady = "true";

  if (main) {
    main.id ||= "phase23-main-content";
    main.setAttribute("role", "main");
    if (!document.querySelector(".phase23-skip-link")) {
      const skip = document.createElement("a");
      skip.className = "phase23-skip-link";
      skip.href = `#${main.id}`;
      skip.textContent = t("skip");
      document.body.prepend(skip);
    }
  }

  const enhance = (root = document) => {
    root.querySelectorAll("nav").forEach((nav) => {
      nav.setAttribute("role", "navigation");
      if (!nav.hasAttribute("aria-label")) nav.setAttribute("aria-label", "Primary");
    });

    root.querySelectorAll("button").forEach((button) => {
      if (!button.hasAttribute("type")) button.setAttribute("type", "button");
      const text = button.textContent?.trim() ?? "";
      if (!button.hasAttribute("aria-label") && !text && button.title) {
        button.setAttribute("aria-label", button.title);
      }
    });

    root
      .querySelectorAll(
        "[id*='status'], [id*='result'], [class*='status'], [class*='toast'], [class*='result']",
      )
      .forEach((node) => {
        if (!node.hasAttribute("aria-live")) node.setAttribute("aria-live", "polite");
        if (!node.hasAttribute("role")) node.setAttribute("role", "status");
        node.setAttribute("aria-atomic", "true");
      });

    root
      .querySelectorAll(".brand-mark, .about-mark, .agent-shield, [data-decorative='true']")
      .forEach((node) => node.setAttribute("aria-hidden", "true"));

    root.querySelectorAll(".nav.active, .nav-item.active").forEach((node) => {
      node.setAttribute("aria-current", "page");
    });
    root.querySelectorAll(".nav:not(.active), .nav-item:not(.active)").forEach((node) => {
      node.removeAttribute("aria-current");
    });
  };

  enhance();

  const observer = new MutationObserver((records) => {
    for (const record of records) {
      if (record.type === "attributes") {
        enhance(record.target.parentElement || document);
      }
      for (const node of record.addedNodes) {
        if (node.nodeType === Node.ELEMENT_NODE) enhance(node);
      }
    }
  });
  observer.observe(document.body, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["class"],
  });

  document.querySelectorAll("nav").forEach((nav) => {
    nav.addEventListener("keydown", (event) => {
      if (!["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) {
        return;
      }
      const controls = [...nav.querySelectorAll("button:not([disabled]), a[href]")];
      if (!controls.length) return;
      const current = controls.indexOf(document.activeElement);
      let next = current < 0 ? 0 : current;
      if (event.key === "Home") next = 0;
      else if (event.key === "End") next = controls.length - 1;
      else if (event.key === "ArrowDown" || event.key === "ArrowRight") {
        next = (Math.max(current, -1) + 1) % controls.length;
      } else {
        next = (current <= 0 ? controls.length : current) - 1;
      }
      controls[next]?.focus();
      event.preventDefault();
    });
  });

  if (
    document.title.includes("Security Center") &&
    !window.localStorage.getItem("dragonforge.phase23.firstRunSeen")
  ) {
    const dialog = document.createElement("dialog");
    dialog.className = "phase23-first-run";
    dialog.setAttribute("aria-labelledby", "phase23-first-run-title");
    dialog.setAttribute("aria-describedby", "phase23-first-run-description");
    dialog.innerHTML = `
      <div class="phase23-first-run-card">
        <div class="eyebrow">${t("privacyLabel")}</div>
        <h2 id="phase23-first-run-title">${t("firstRunTitle")}</h2>
        <p id="phase23-first-run-description">${t("firstRunBody")}</p>
        <p>${t("firstRunA11y")}</p>
        <button class="phase23-first-run-continue" type="button">${t("firstRunContinue")}</button>
      </div>`;
    document.body.appendChild(dialog);
    const close = () => {
      window.localStorage.setItem("dragonforge.phase23.firstRunSeen", "1");
      dialog.close();
      dialog.remove();
    };
    dialog.querySelector(".phase23-first-run-continue")?.addEventListener("click", close);
    dialog.addEventListener("cancel", (event) => {
      event.preventDefault();
      close();
    });
    dialog.showModal();
    dialog.querySelector("button")?.focus();
  }
})();
