// 📄 Dosya Yolu: /turkuazvm/apps/desktop/ui/views/vm_workspace_view.js
// 📌 Amac: TurkuazVM Home dashboard ve VM Workspace HTML gorunumlerini saf View fonksiyonlariyla uretir
// 📌 Modul - JavaScript
// Version: 0.41.1
// Aciklama: Ana sayfada hizli aksiyonlari ve secili VM kontrol alaninda buyuk ekran okunabilirligini koruyan workspace gorunumlerini uretir
// Bagimli Oldugu Katman: View

(function attachTurkuazVmWorkspaceView(globalScope) {
  "use strict";

  const GIB = 1024 ** 3;

  function bytesToGib(value) {
    const bytes = Number(value || 0);
    if (!Number.isFinite(bytes) || bytes <= 0) return "-";
    const gib = bytes / GIB;
    return `${gib >= 10 ? gib.toFixed(0) : gib.toFixed(1)} GiB`;
  }

  function stateClass(state) {
    const normalized = String(state || "").toLowerCase();
    if (normalized === "running") return "running";
    if (normalized === "error") return "error";
    return "stopped";
  }

  function renderHome(context) {
    const { dashboard, machines, escapeHtml, escapeAttribute, getStateLabel, getGuestProfileLabel, formatMemory } = context;
    const safeMachines = Array.isArray(machines) ? machines : [];
    const running = safeMachines.filter((machine) => String(machine.state || "").toLowerCase() === "running").length;
    const stopped = safeMachines.filter((machine) => String(machine.state || "").toLowerCase() === "stopped").length;
    const hostLabel = dashboard?.host_label || "Yerel Host";
    const hostPlatform = [dashboard?.platform, dashboard?.architecture].filter(Boolean).join(" / ") || "-";
    const acceleration = String(dashboard?.acceleration || "-").toUpperCase();
    const qemuReady = Boolean(dashboard?.qemu_ready);
    const qemuImgReady = Boolean(dashboard?.qemu_img_ready);
    const engineReady = Boolean(dashboard?.engine_ready);
    const recentMachines = [...safeMachines]
      .sort((left, right) => {
        const leftRunning = String(left.state || "").toLowerCase() === "running" ? 0 : 1;
        const rightRunning = String(right.state || "").toLowerCase() === "running" ? 0 : 1;
        return leftRunning - rightRunning || String(left.name || "").localeCompare(String(right.name || ""), "tr");
      })
      .slice(0, 5);

    return `
      <section class="home-hero">
        <div>
          <p class="home-kicker">TURKUAZVM WORKSPACE</p>
          <h1>${engineReady ? "Sistem hazir." : "Sistem kontrol bekliyor."}</h1>
          <p>VM'leri, sistem goruntulerini ve host kaynaklarini tek calisma alanindan yonetin. Teknik ayrintilar Uzman Modu acildiginda gorunur.</p>
        </div>
        <div class="home-hero-actions">
          <button class="secondary-button" type="button" data-home-action="images">Goruntu Merkezi</button>
        </div>
      </section>
      <section class="home-metric-grid" aria-label="Sistem ozeti">
        <article class="home-metric"><span>Toplam VM</span><strong>${safeMachines.length}</strong><small>${running} calisiyor / ${stopped} kapali</small></article>
        <article class="home-metric"><span>Calisan</span><strong>${running}</strong><small>${running ? "Aktif sanal makineler" : "Aktif hata yok"}</small></article>
        <article class="home-metric"><span>Host</span><strong>${escapeHtml(acceleration)}</strong><small>${escapeHtml(hostPlatform)}</small></article>
        <article class="home-metric"><span>QEMU</span><strong>${qemuReady ? "Hazir" : "Bekliyor"}</strong><small>${qemuImgReady ? "qemu-img hazir" : "qemu-img kontrol gerekli"}</small></article>
      </section>
      <section class="home-action-strip" aria-label="Hizli baslangic">
        <button class="home-action-card primary" type="button" data-home-action="create"><span class="home-action-code">+</span><span><strong>Yeni VM Olustur</strong><small>CPU, RAM, disk ve sistemi adim adim sec.</small></span></button>
        <button class="home-action-card" type="button" data-home-action="images"><span class="home-action-code">ISO</span><span><strong>Goruntu Merkezi</strong><small>Windows, Linux ve Android kaynaklarini hazirla.</small></span></button>
        <button class="home-action-card" type="button" data-home-action="machines"><span class="home-action-code">VM</span><span><strong>VM Kontrol Merkezi</strong><small>Calistir, konsolu ac ve tum kontrolleri yonet.</small></span></button>
      </section>
      <section class="home-content-grid">
        <article class="home-panel">
          <header><strong>Sanal Makineler</strong><span>${safeMachines.length} makine</span></header>
          <div class="home-machine-list">
            ${recentMachines.length ? recentMachines.map((machine) => `
              <button class="home-machine-row" type="button" data-home-vm-id="${escapeAttribute(machine.id)}">
                <span class="home-machine-avatar">${escapeHtml(String(getGuestProfileLabel(machine.guest_profile) || "VM").slice(0, 1).toUpperCase())}</span>
                <span class="home-machine-copy"><strong>${escapeHtml(machine.name)}</strong><small>${escapeHtml(getGuestProfileLabel(machine.guest_profile))} / ${machine.vcpu_count} vCPU / ${escapeHtml(formatMemory(machine.memory_mib))}</small></span>
                <span class="home-machine-state ${stateClass(machine.state)}">${escapeHtml(getStateLabel(machine.state))}</span>
              </button>
            `).join("") : `<div class="home-empty"><strong>Henuz VM yok</strong><span>Yeni VM ile ilk sanal makinenizi olusturun.</span></div>`}
          </div>
        </article>
        <article class="home-panel system-health-panel">
          <header><strong>Sistem Sagligi</strong><span>${escapeHtml(hostLabel)}</span></header>
          <div class="home-health-row"><i class="${engineReady ? "ok" : "warn"}"></i><span>Engine</span><small>${engineReady ? "Hazir" : "Kontrol gerekli"}</small></div>
          <div class="home-health-row"><i class="${qemuReady ? "ok" : "warn"}"></i><span>QEMU Runtime</span><small>${qemuReady ? "Hazir" : "Kontrol gerekli"}</small></div>
          <div class="home-health-row expert-only"><i class="${qemuImgReady ? "ok" : "warn"}"></i><span>Disk Toolchain</span><small>${qemuImgReady ? "qemu-img hazir" : "Eksik"}</small></div>
          <div class="home-health-row expert-only"><i class="${dashboard?.authentication_required ? "warn" : "ok"}"></i><span>Transport</span><small>${escapeHtml(String(dashboard?.transport_security || "local").toUpperCase())}</small></div>
        </article>
      </section>`;
  }

  function renderStorage(machine, context) {
    const { escapeHtml, escapeAttribute, stopped } = context;
    const disks = Array.isArray(machine.disks) ? machine.disks : [];
    return `
      <div class="workspace-section-heading"><div><strong>Disk ve Medya</strong><span>${disks.length} sanal disk, ${machine.installer_media ? "ISO bagli" : "ISO yok"}</span></div><button class="detail-secondary" data-action="add-disk" data-vm-id="${escapeAttribute(machine.id)}">+ Disk Ekle</button></div>
      <div class="workspace-resource-list">
        ${disks.length ? disks.map((disk) => `<article class="workspace-resource-row"><div><strong>${escapeHtml(disk.id || "disk")}</strong><span class="resource-pill">${escapeHtml(String(disk.format || "qcow2").toUpperCase())}</span><small>${bytesToGib(disk.virtual_size_bytes)} &nbsp; ${escapeHtml(String(disk.bus || "virtio").toUpperCase())} &nbsp; Boot ${Number(disk.boot_index ?? 0)} &nbsp; <span class="expert-only">${escapeHtml(disk.relative_path || "-")}</span></small></div><button class="detail-secondary" data-action="add-disk" data-vm-id="${escapeAttribute(machine.id)}">Yonet</button></article>`).join("") : `<div class="workspace-inline-empty">Bu VM icin disk yok.</div>`}
        <article class="workspace-resource-row media-row"><div><strong>CD/DVD Kurulum Medyasi</strong><span class="resource-pill">ISO</span><small>${escapeHtml(machine.installer_media || "Bagli medya yok")}</small></div><button class="detail-secondary" data-action="media" data-vm-id="${escapeAttribute(machine.id)}" ${stopped ? "" : "disabled"}>Yonet</button></article>
      </div>`;
  }

  function renderNetwork(machine, context) {
    const { escapeHtml, escapeAttribute } = context;
    const networks = Array.isArray(machine.networks) ? machine.networks : [];
    return `
      <div class="workspace-section-heading"><div><strong>Ag Baglantilari</strong><span>${networks.length} adapter / ${context.publishedServiceCount} yayin</span></div><button class="detail-secondary" data-action="network" data-vm-id="${escapeAttribute(machine.id)}">Ag Ayarlari</button></div>
      <div class="workspace-resource-list">
        ${networks.length ? networks.map((network) => `<article class="network-resource-card"><div class="network-resource-head"><div><strong>${escapeHtml(network.id || "net")}</strong><span class="resource-pill">${escapeHtml(String(network.mode || "user_nat").toUpperCase())}</span></div><button class="detail-secondary" data-action="network" data-vm-id="${escapeAttribute(machine.id)}">Yonet</button></div><div class="network-facts"><span><small>IPv4</small><strong>${escapeHtml(network.ipv4_address || "Bekleniyor")}</strong></span><span><small>Gateway</small><strong>${escapeHtml(network.gateway || "-")}</strong></span><span class="expert-only"><small>Model</small><strong>${escapeHtml(network.device_model || "-")}</strong></span><span class="expert-only"><small>MAC</small><strong>${escapeHtml(network.mac_address || "-")}</strong></span></div>${Array.isArray(network.published_services) && network.published_services.length ? `<div class="published-service-list">${network.published_services.map((service) => `<span>${escapeHtml(String(service.protocol || "tcp").toUpperCase())} ${escapeHtml(service.host_ip || "127.0.0.1")}:${Number(service.host_port || 0)} -> ${Number(service.guest_port || 0)}</span>`).join("")}</div>` : `<small class="network-empty-note">Yayinlanmis servis yok.</small>`}</article>`).join("") : `<div class="workspace-inline-empty">Bu VM icin ag adapteri yok.</div>`}
      </div>`;
  }

  function renderAccess(machine, context) {
    const { escapeHtml, escapeAttribute } = context;
    const services = (Array.isArray(machine.networks) ? machine.networks : []).flatMap((network) => Array.isArray(network.published_services) ? network.published_services : []);
    const ssh = services.find((service) => Number(service.guest_port) === 22);
    const rdp = services.find((service) => Number(service.guest_port) === 3389);
    const accessCard = (title, service, kind) => `<article class="access-card"><div><span class="access-status-dot ${service ? "ready" : "idle"}"></span><div><strong>${title}</strong><small>${service ? `${escapeHtml(service.host_ip || "127.0.0.1")}:${Number(service.host_port || 0)}` : "Yayin yapilandirmasi bekleniyor"}</small></div></div><button class="detail-secondary" data-action="connect" data-vm-id="${escapeAttribute(machine.id)}">${service ? "Baglan" : "Hazirla"}</button></article>`;
    return `<div class="workspace-section-heading"><div><strong>Uzak Erisim</strong><span>SSH, RDP ve guest servislerini tek merkezden yonetin.</span></div><button class="detail-secondary" data-action="connect" data-vm-id="${escapeAttribute(machine.id)}">Erisim Merkezi</button></div><div class="access-grid">${accessCard("SSH", ssh, "ssh")}${accessCard("RDP", rdp, "rdp")}${machine.guest_profile === "android" ? accessCard("ADB", services.find((service) => Number(service.guest_port) === 5555), "adb") : ""}</div>`;
  }

  function renderVmDetail(machine, context) {
    const {
      escapeHtml,
      escapeAttribute,
      getStateLabel,
      getGuestProfileLabel,
      formatMemory,
      selectedTab,
      canStart,
      canDelete,
      running,
      stopped,
      hasDisk,
      hasNetwork,
      hasRemote,
      androidImageMissing,
      primaryAddress,
      publishedServiceCount
    } = context;
    const state = String(machine.state || "").toLowerCase();
    const stateLabel = getStateLabel(state);
    const guestLabel = getGuestProfileLabel(machine.guest_profile);
    const accelerationLabel = String(machine.acceleration || "-").toUpperCase();
    const diskCount = Number(machine.disk_count || 0);
    const networkCount = Number(machine.network_count || 0);

    const primaryControl = running
      ? `<button class="vm-control-button primary" data-action="display" data-vm-id="${escapeAttribute(machine.id)}"><span>Konsolu Ac</span><small>Goruntu penceresini ac</small></button>`
      : `<button class="vm-control-button primary" data-action="start" data-vm-id="${escapeAttribute(machine.id)}" ${canStart ? "" : "disabled"}><span>VM Baslat</span><small>${canStart ? "Sanal makineyi calistir" : "Baslatma kosullari eksik"}</small></button>`;
    const stopControl = running
      ? `<button class="vm-control-button stop" data-action="stop" data-vm-id="${escapeAttribute(machine.id)}"><span>Durdur</span><small>Guvenli kapatma istegi</small></button>`
      : "";

    const tabs = [
      ["overview", "Genel"],
      ["console", "Konsol"],
      ["hardware", "Donanim"],
      ["storage", "Disk"],
      ["network", "Ag"],
      ["snapshots", "Snapshot"],
      ["access", "Erisim"],
      ["logs", "Gunluk"]
    ];

    const commandCard = (action, code, title, description, options = {}) => {
      const disabled = options.disabled ? "disabled" : "";
      const extra = options.extraAttributes || "";
      const tone = options.tone ? ` ${options.tone}` : "";
      return `<button class="vm-command-card${tone}" type="button" data-action="${escapeAttribute(action)}" data-vm-id="${escapeAttribute(machine.id)}" ${extra} ${disabled}><span class="vm-command-code">${escapeHtml(code)}</span><span class="vm-command-copy"><strong>${escapeHtml(title)}</strong><small>${escapeHtml(description)}</small></span><span class="vm-command-arrow" aria-hidden="true">&gt;</span></button>`;
    };

    const overviewCommands = [
      commandCard("edit", "CPU", "Donanim", `${machine.vcpu_count} vCPU / ${formatMemory(machine.memory_mib)}`, { disabled: !stopped }),
      commandCard("media", "ISO", "Kurulum Medyasi", machine.installer_media ? "ISO bagli" : "ISO bagli degil", { disabled: !stopped }),
      commandCard("add-disk", "DSK", "Diskler", `${diskCount} sanal disk`),
      commandCard("network", "NET", "Ag", `${networkCount} adapter / ${publishedServiceCount} yayin`),
      commandCard("snapshots", "SNP", "Snapshot", "Geri donus noktalarini yonet"),
      commandCard("connect", "LNK", "Baglanti", hasRemote ? "Uzak erisim hazir" : "Erisim hedeflerini yapilandir")
    ];
    if (machine.guest_profile === "android") {
      overviewCommands.push(commandCard(
        "android",
        "AND",
        "Android Profili",
        androidImageMissing ? "Android image gerekli" : "ADB ve Android ayarlari",
        { extraAttributes: `data-vm-name="${escapeAttribute(machine.name)}" data-vm-state="${escapeAttribute(machine.state)}"` }
      ));
    }

    const readinessItems = [
      [hasDisk, "Onyukleme diski", hasDisk ? "Hazir" : "Disk gerekli"],
      [hasNetwork, "Ag baglantisi", hasNetwork ? "Hazir" : "Adapter yok"],
      [hasRemote, "Uzak erisim", hasRemote ? `${publishedServiceCount} yayin` : "Istege bagli"]
    ];
    if (machine.guest_profile === "android") {
      readinessItems.push([!androidImageMissing, "Android image", androidImageMissing ? "Atama gerekli" : "Hazir"]);
    }

    const contextCards = {
      overview: `
        <div class="vm-overview-control-layout">
          <section class="vm-overview-section">
            <div class="workspace-section-heading compact"><div><strong>Hizli Islemler</strong><span>En cok kullanilan VM ayarlarina tek tikla ulasin.</span></div></div>
            <div class="vm-command-grid">${overviewCommands.join("")}</div>
          </section>
          <aside class="vm-readiness-card">
            <div class="vm-readiness-head"><div><strong>Calisma Hazirligi</strong><span>${running ? "VM su anda calisiyor" : "Baslatma oncesi kontrol"}</span></div><span class="detail-state-badge ${stateClass(state)}">${escapeHtml(stateLabel)}</span></div>
            <div class="vm-readiness-list">${readinessItems.map(([ok, label, detail]) => `<div class="vm-readiness-row"><span class="vm-readiness-icon ${ok ? "ok" : "warn"}">${ok ? "OK" : "!"}</span><span>${escapeHtml(label)}</span><small>${escapeHtml(detail)}</small></div>`).join("")}</div>
            ${androidImageMissing ? `<button class="vm-readiness-fix" type="button" data-action="assign-image" data-vm-id="${escapeAttribute(machine.id)}">Android Image Ata</button>` : ""}
          </aside>
        </div>
        <div class="vm-overview-danger"><div><strong>VM Yonetimi</strong><span>Silme islemi VM klasorundeki disk ve medya verilerini kalici olarak kaldirir.</span></div><button class="detail-danger" data-action="delete-vm" data-vm-id="${escapeAttribute(machine.id)}" data-vm-name="${escapeAttribute(machine.name)}" ${canDelete ? "" : "disabled"}>VM Sil</button></div>`,
      console: `<div class="console-workspace-card"><div class="console-placeholder"><span>TURKUAZ VM</span><small>${running ? "Konsol hazir" : "VM kapali"}</small></div><div class="console-actions"><button class="detail-primary" data-action="display" data-vm-id="${escapeAttribute(machine.id)}" ${running ? "" : "disabled"}>Konsolu Ac</button><span>${running ? "Yerel goruntu penceresini acin." : "Konsol icin once VM'i baslatin."}</span></div></div>`,
      hardware: `<div class="hardware-grid"><article><span>CPU</span><strong>${machine.vcpu_count} vCPU</strong></article><article><span>Bellek</span><strong>${escapeHtml(formatMemory(machine.memory_mib))}</strong></article><article><span>Hizlandirma</span><strong>${escapeHtml(accelerationLabel)}</strong></article><article><span>Sistem</span><strong>${escapeHtml(guestLabel)}</strong></article></div><div class="workspace-section-actions"><button class="detail-secondary" data-action="edit" data-vm-id="${escapeAttribute(machine.id)}" ${stopped ? "" : "disabled"}>Donanimi Duzenle</button>${machine.guest_profile === "android" ? `<button class="detail-secondary" data-action="android" data-vm-id="${escapeAttribute(machine.id)}" data-vm-name="${escapeAttribute(machine.name)}" data-vm-state="${escapeAttribute(machine.state)}">Android Profili</button>` : ""}</div>`,
      storage: renderStorage(machine, { escapeHtml, escapeAttribute, stopped }),
      network: renderNetwork(machine, { escapeHtml, escapeAttribute, publishedServiceCount }),
      snapshots: `<div id="inline-snapshot-content" class="workspace-lazy-panel"><div class="workspace-loading">Snapshotlar yukleniyor...</div></div>`,
      access: renderAccess(machine, { escapeHtml, escapeAttribute }),
      logs: `<div id="inline-log-content" class="workspace-lazy-panel"><div class="workspace-loading">Gunlukler yukleniyor...</div></div>`
    };

    return `
      <header class="vm-detail-header vm-command-center">
        <div class="vm-detail-identity-row">
          <div class="vm-detail-avatar">${escapeHtml(String(guestLabel || "VM").slice(0, 1).toUpperCase())}</div>
          <div class="vm-detail-title">
            <div class="vm-detail-title-top"><h2>${escapeHtml(machine.name)}</h2><span class="detail-state-badge ${stateClass(state)}">${escapeHtml(stateLabel)}</span></div>
            <div class="vm-detail-id"><span class="expert-only">${escapeHtml(machine.id)} / </span>${escapeHtml(guestLabel)}</div>
          </div>
        </div>
        <div class="vm-primary-controls">
          <div class="vm-primary-control-group">${primaryControl}${stopControl}<button class="vm-control-button" data-action="connect" data-vm-id="${escapeAttribute(machine.id)}"><span>Baglan</span><small>SSH, RDP veya ADB</small></button></div>
          <div class="vm-quick-control-group">
            <button class="vm-quick-control" data-action="edit" data-vm-id="${escapeAttribute(machine.id)}" ${stopped ? "" : "disabled"}>Donanim</button>
            <button class="vm-quick-control" data-action="add-disk" data-vm-id="${escapeAttribute(machine.id)}">Disk</button>
            <button class="vm-quick-control" data-action="network" data-vm-id="${escapeAttribute(machine.id)}">Ag</button>
            <button class="vm-quick-control" data-action="snapshots" data-vm-id="${escapeAttribute(machine.id)}">Snapshot</button>
          </div>
        </div>
        <div class="vm-fact-strip" aria-label="VM kaynak ozeti">
          <div><span>CPU</span><strong>${machine.vcpu_count} vCPU</strong></div>
          <div><span>Bellek</span><strong>${escapeHtml(formatMemory(machine.memory_mib))}</strong></div>
          <div><span>Disk</span><strong>${diskCount}</strong></div>
          <div><span>IPv4</span><strong>${escapeHtml(primaryAddress || "-")}</strong></div>
          <div class="expert-only"><span>Hizlandirma</span><strong>${escapeHtml(accelerationLabel)}</strong></div>
        </div>
      </header>
      ${machine.last_error && !context.recoverableError ? `<div class="vm-detail-error">${escapeHtml(machine.last_error)}</div>` : ""}
      <nav class="detail-tabs" aria-label="VM detay sekmeleri">${tabs.map(([key, label]) => `<button class="detail-tab ${selectedTab === key ? "active" : ""}" data-detail-tab="${key}" type="button">${label}</button>`).join("")}</nav>
      <section class="detail-context-panel">${contextCards[selectedTab] || contextCards.overview}</section>`;
  }

  globalScope.TurkuazVmWorkspaceView = Object.freeze({ renderHome, renderVmDetail });
})(window);
