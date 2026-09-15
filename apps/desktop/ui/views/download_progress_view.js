// 📄 Dosya Yolu: /turkuazvm/apps/desktop/ui/views/download_progress_view.js
// 📌 Amac: Android image ve Windows/Linux ISO indirmeleri icin tek ortak progress View bilesenini render eder
// 📌 Modul - JavaScript
// Version: 0.38.0
// Aciklama: Determinate/indeterminate progress, yuzde, byte, hiz, ETA, sure, asama ve hata durumlarini ortak DOM kontratiyla gosterir
// Bagimli Oldugu Katman: View

(function attachTurkuazVmDownloadProgressView(globalScope) {
  "use strict";

  const ACTIVE_BADGE = "AKTIF";
  const FAILED_BADGE = "HATA";
  const READY_BADGE = "100%";

  function role(root, name) {
    return root?.querySelector(`[data-download-role="${name}"]`) || null;
  }

  function setText(root, name, value) {
    const node = role(root, name);
    if (node) node.textContent = value ?? "-";
  }

  function hide(root) {
    if (!root) return;
    root.classList.add("hidden");
    root.classList.remove("is-failed", "is-complete", "is-cancelled");
    const track = role(root, "track");
    if (track) {
      track.classList.add("is-indeterminate");
      track.classList.remove("is-failed", "is-complete");
      track.removeAttribute("aria-valuenow");
    }
    const bar = role(root, "bar");
    if (bar) bar.style.width = "0%";
  }

  function render(root, model) {
    if (!root || !model) return;
    root.classList.remove("hidden");

    const state = String(model.state || "active").toLowerCase();
    const failed = state === "failed";
    const complete = state === "ready" || state === "complete";
    const cancelled = state === "cancelled";
    const percent = Number.isFinite(model.percent) ? Math.min(100, Math.max(0, Math.round(model.percent))) : null;
    const determinate = percent !== null && !model.indeterminate && !failed && !cancelled;

    root.classList.toggle("is-failed", failed);
    root.classList.toggle("is-complete", complete);
    root.classList.toggle("is-cancelled", cancelled);

    setText(root, "title", model.title || "Indirme durumu");
    setText(root, "stage", model.stageLabel || model.stage || "Asama bekleniyor");
    setText(root, "bytes", model.bytesLabel || "Boyut bekleniyor");
    setText(root, "speed", `Hiz ${model.speedLabel || "-"}`);
    setText(root, "eta", `ETA ${model.etaLabel || "-"}`);
    setText(root, "elapsed", `Sure ${model.elapsedLabel || "-"}`);
    setText(root, "detail", model.detail || "");

    const badge = role(root, "percent");
    if (badge) {
      badge.textContent = failed ? FAILED_BADGE : (complete ? READY_BADGE : (determinate ? `%${percent}` : (cancelled ? "IPTAL" : ACTIVE_BADGE)));
      badge.classList.toggle("error", failed);
      badge.classList.toggle("complete", complete);
    }

    const track = role(root, "track");
    if (track) {
      track.classList.toggle("is-indeterminate", !determinate && !failed && !complete && !cancelled);
      track.classList.toggle("is-failed", failed);
      track.classList.toggle("is-complete", complete);
      if (determinate) track.setAttribute("aria-valuenow", String(percent));
      else track.removeAttribute("aria-valuenow");
    }

    const bar = role(root, "bar");
    if (bar) {
      if (failed || complete) bar.style.width = "100%";
      else if (determinate) bar.style.width = `${percent}%`;
      else if (cancelled) bar.style.width = "0%";
      else bar.style.width = "34%";
    }
  }

  globalScope.TurkuazVmDownloadProgressView = Object.freeze({ render, hide });
})(window);
