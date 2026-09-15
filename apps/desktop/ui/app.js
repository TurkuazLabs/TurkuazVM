// 📄 Dosya Yolu: /turkuazvm/apps/desktop/ui/app.js
// 📌 Amac: TurkuazVM Desktop UI ile Tauri Controller commandlarini baglar
// 📌 Modul - JavaScript
// Version: 0.41.2
// Aciklama: VM kontrol merkezi ve Baglanti Merkezi akislarini yonetir; calisan Turkuaz NAT VM icin SSH/RDP host yayinini onayli otomatik stop/restart ile hazirlar
// Bagimli Oldugu Katman: Controller | View

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const downloadProgressView = window.TurkuazVmDownloadProgressView;

const EVENT_RUNTIME_UPDATE = "turkuazvm://runtime-update";
const STATE_RUNNING = "running";
const STATE_STOPPED = "stopped";
const JOYSTICK_POINTER_ID = 0;
const MOUSE_LOOK_POINTER_ID = 1;
const VM_FILTER_ALL = "all";
const VM_VIEW_CARD = "card";
const VM_VIEW_COMPACT = "compact";
const VM_SORT_NAME_ASC = "name-asc";
const UI_STORAGE_VIEW_MODE = "turkuazvm.ui.vm_view_mode.v2";
const UI_STORAGE_SORT_MODE = "turkuazvm.ui.vm_sort_mode";
const UI_STORAGE_SIDEBAR = "turkuazvm.ui.sidebar_collapsed";
const UI_STORAGE_EXPERT_MODE = "turkuazvm.ui.expert_mode";
const ACTIVITY_LIMIT = 40;
const TOAST_DURATION_MS = 3600;
const RESOURCE_INDEX_WIDTH = 2;
const RESOURCE_KIND_DISK = "disk";
const RESOURCE_KIND_NETWORK = "net";
const NETWORK_MODE_MANAGED_NAT = "managed_nat";
const NETWORK_MODE_USER_NAT = "user_nat";
const NETWORK_PROTOCOL_TCP = "tcp";
const SSH_GUEST_PORT = 22;
const RDP_GUEST_PORT = 3389;
const CONNECTION_USER_STORAGE_PREFIX = "turkuazvm.connection.ssh_user.";
const CONNECTION_STATUS_READY = "ready";
const CONNECTION_STATUS_WARNING = "warning";
const CONNECTION_STATUS_ERROR = "error";
const CONNECTION_MODE_QEMU_NAT = "qemu_nat";
const CONNECTION_MODE_DIRECT = "direct";
const INSTALLER_MEDIA_MODE_OFFICIAL_PAGE = "official_page";
const HOST_MODE_REMOTE = "remote";
const CREATE_STEP_ORDER = Object.freeze(["catalog", "resource", "summary"]);
const VM_FILTER_LABELS = Object.freeze({
  all: "Tumu",
  running: "Calisiyor",
  stopped: "Durdu",
  error: "Hata",
  android: "Android",
  linux: "Linux",
  windows: "Windows"
});
const VM_STATE_SORT_RANK = Object.freeze({ running: 0, starting: 1, paused: 2, stopped: 3, error: 4 });
const ANDROID_RELEASE_ORDER = Object.freeze(["17", "16", "15", "14", "13", "12L", "12", "11", "10"]);
const ANDROID_CI_OFFICIAL_BASE_URL = "https://ci.android.com";
const ANDROID_SDK_IMAGE_SUFFIX = "-sdk-x86_64";
const ANDROID_INSTALL_STAGE_LABELS = Object.freeze({
  discovering: "Resmi paket katalogu okunuyor",
  checking_disk: "Disk kontrolu",
  downloading_device: "Android System Image indiriliyor",
  downloading_host: "Android Emulator indiriliyor",
  validating: "Dogrulaniyor",
  extracting: "Paketler aciliyor",
  assembling: "Emulator bundle hazirlaniyor",
  finalizing: "Kurulum tamamlaniyor",
  cancelling: "Iptal ediliyor",
  completed: "Hazir",
  failed: "Kurulum hatasi"
});
const ANDROID_INSTALL_ACTIVE_STAGE_ORDER = Object.freeze([
  "checking_disk",
  "discovering",
  "downloading_device",
  "downloading_host",
  "validating",
  "extracting",
  "assembling",
  "finalizing"
]);

const uiState = {
  snapshotVmId: null,
  snapshotVmName: null,
  cloneSourceVmId: null,
  cloneSourceVmName: null,
  androidVmId: null,
  androidVmName: null,
  androidVmState: null,
  gamingInputVmId: null,
  gamingInputVmName: null,
  gamingBindings: [],
  gameCatalog: [],
  guestCatalog: [],
  selectedGuestFamily: "linux",
  createStep: "catalog",
  createDiskTouched: false,
  createMediaTemplateId: null,
  createMediaSources: [],
  createMediaSource: null,
  createMediaSelectedMediaId: null,
  createMediaLocalPath: null,
  createMediaReady: false,
  createMediaPollTimer: null,
  createMediaProgressSample: null,
  createAndroidImageId: null,
  createAndroidImageDeferred: false,
  createAndroidImagePollTimer: null,
  createdVmId: null,
  createdVmRecommendedDiskGib: null,
  createdVmSourceKind: null,
  storageFlowVmId: null,
  storageFlowNextStep: null,
  storageFlowDiskReady: false,
  networkFlowVmId: null,
  networkFlowReady: false,
  networkFlowFromAndroid: false,
  androidImageFlowVmId: null,
  androidImageFlowReady: false,
  installerMediaFlowVmId: null,
  installerMediaSelectedPath: null,
  installerMediaTemplateId: null,
  installerMediaSource: null,
  installerMediaSources: [],
  installerMediaSelectedMediaId: null,
  installerMediaPollTimer: null,
  installerMediaProgressSample: null,
  installerMediaReady: false,
  configurationCompleteVmId: null,
  vmIdentityTouched: false,
  vmIdTouched: false,
  androidImages: [],
  androidAssignments: {},
  androidImagePollTimer: null,
  androidAutoAssignVmId: null,
  androidAutoAssignImageId: null,
  androidStandardReleaseId: null,
  imageCenterTab: "iso",
  imageCenterIsoSourceId: null,
  imageCenterIsoDownloads: new Map(),
  imageCenterIsoPollTimer: null,
  taskDockOpen: false,
  artifactCacheEntries: [],
  machines: [],
  hosts: [],
  qemuReady: false,
  qemuImgReady: false,
  hostArchitecture: null,
  hostPlatform: null,
  hostAcceleration: null,
  vmSearchQuery: "",
  selectedVmId: null,
  selectedVmTab: "overview",
  activeNavigation: "home",
  vmQuickFilter: VM_FILTER_ALL,
  vmSortMode: VM_SORT_NAME_ASC,
  vmViewMode: VM_VIEW_COMPACT,
  sidebarCollapsed: false,
  expertMode: false,
  activities: [],
  dashboard: null
};

const elements = {
  engineStatus: document.querySelector("#engine-status"),
  engineMiniStatus: document.querySelector("#engine-mini-status"),
  resourceEngineStatus: document.querySelector("#resource-engine-status"),
  qemuStatus: document.querySelector("#qemu-status"),
  hostStatus: document.querySelector("#host-status"),
  accelerationStatus: document.querySelector("#acceleration-status"),
  transportStatus: document.querySelector("#transport-status"),
  adbStatus: document.querySelector("#adb-status"),
  gpuStatus: document.querySelector("#gpu-status"),
  appShell: document.querySelector(".app-shell"),
  sidebarCollapseButton: document.querySelector("#sidebar-collapse-button"),
  topbarSidebarToggle: document.querySelector("#topbar-sidebar-toggle"),
  shortcutsButton: document.querySelector("#shortcuts-button"),
  expertModeButton: document.querySelector("#expert-mode-button"),
  shortcutsModal: document.querySelector("#shortcuts-modal"),
  closeShortcutsModal: document.querySelector("#close-shortcuts-modal"),
  toastStack: document.querySelector("#toast-stack"),
  homeButton: document.querySelector("#home-button"),
  homePage: document.querySelector("#home-page"),
  homeOverview: document.querySelector("#home-overview"),
  machinesPage: document.querySelector("#machines-page"),
  vmWorkspaceShell: document.querySelector("#vm-workspace-shell"),
  fleetEmptyWorkspace: document.querySelector("#fleet-empty-workspace"),
  workspaceSectionKicker: document.querySelector("#workspace-section-kicker"),
  workspaceSectionTitle: document.querySelector("#workspace-section-title"),
  machinesButton: document.querySelector("#machines-button"),
  vmSearchInput: document.querySelector("#vm-search-input"),
  clearVmFilterButton: document.querySelector("#clear-vm-filter-button"),
  bulkStartButton: document.querySelector("#bulk-start-button"),
  bulkStopButton: document.querySelector("#bulk-stop-button"),
  selectionSummary: document.querySelector("#selection-summary"),
  treeMachineList: document.querySelector("#tree-machine-list"),
  vmFilterGroup: document.querySelector("#vm-filter-group"),
  vmSortSelect: document.querySelector("#vm-sort-select"),
  cardViewButton: document.querySelector("#card-view-button"),
  compactViewButton: document.querySelector("#compact-view-button"),
  summaryTotalVms: document.querySelector("#summary-total-vms"),
  summaryRunningVms: document.querySelector("#summary-running-vms"),
  summaryStoppedVms: document.querySelector("#summary-stopped-vms"),
  summaryErrorVms: document.querySelector("#summary-error-vms"),
  summarySnapshots: document.querySelector("#summary-snapshots"),
  summaryDisks: document.querySelector("#summary-disks"),
  activityList: document.querySelector("#activity-list"),
  activityEmpty: document.querySelector("#activity-empty"),
  clearActivityButton: document.querySelector("#clear-activity-button"),
  storageButton: document.querySelector("#storage-button"),
  artifactCacheButton: document.querySelector("#artifact-cache-button"),
  artifactCacheModal: document.querySelector("#artifact-cache-modal"),
  closeArtifactCacheModal: document.querySelector("#close-artifact-cache-modal"),
  artifactCacheEnabled: document.querySelector("#artifact-cache-enabled"),
  artifactCacheUsed: document.querySelector("#artifact-cache-used"),
  artifactCacheQuota: document.querySelector("#artifact-cache-quota"),
  artifactCacheCount: document.querySelector("#artifact-cache-count"),
  artifactCachePinned: document.querySelector("#artifact-cache-pinned"),
  artifactCacheMutable: document.querySelector("#artifact-cache-mutable"),
  artifactCacheIntegrity: document.querySelector("#artifact-cache-integrity"),
  artifactCacheOffline: document.querySelector("#artifact-cache-offline"),
  artifactCacheResult: document.querySelector("#artifact-cache-result"),
  artifactCacheRefresh: document.querySelector("#artifact-cache-refresh"),
  artifactCacheVerify: document.querySelector("#artifact-cache-verify"),
  artifactCacheRevalidateAll: document.querySelector("#artifact-cache-revalidate-all"),
  artifactCacheCleanup: document.querySelector("#artifact-cache-cleanup"),
  artifactCacheList: document.querySelector("#artifact-cache-list"),
  artifactCacheEmpty: document.querySelector("#artifact-cache-empty"),
  artifactCacheFetchForm: document.querySelector("#artifact-cache-fetch-form"),
  artifactCacheFetchKey: document.querySelector("#artifact-cache-fetch-key"),
  artifactCacheFetchUrl: document.querySelector("#artifact-cache-fetch-url"),
  artifactCacheFetchPinned: document.querySelector("#artifact-cache-fetch-pinned"),
  artifactCacheFetch: document.querySelector("#artifact-cache-fetch"),
  networkButton: document.querySelector("#network-button"),
  gamingButton: document.querySelector("#gaming-button"),
  androidImagesButton: document.querySelector("#android-images-button"),
  logsButton: document.querySelector("#logs-button"),
  storageModal: document.querySelector("#storage-modal"),
  closeStorageModal: document.querySelector("#close-storage-modal"),
  storageDataRoot: document.querySelector("#storage-data-root"),
  storageFree: document.querySelector("#storage-free"),
  storageTotal: document.querySelector("#storage-total"),
  storageRuntimeFormat: document.querySelector("#storage-runtime-format"),
  storagePortableFormat: document.querySelector("#storage-portable-format"),
  storageIsolation: document.querySelector("#storage-isolation"),
  storageDiskForm: document.querySelector("#storage-disk-form"),
  storageVm: document.querySelector("#storage-vm"),
  storageDiskId: document.querySelector("#storage-disk-id"),
  storageDiskSize: document.querySelector("#storage-disk-size"),
  storageNote: document.querySelector("#storage-note"),
  storageDiskList: document.querySelector("#storage-disk-list"),
  storageDiskEmpty: document.querySelector("#storage-disk-empty"),
  storageEyebrow: document.querySelector("#storage-eyebrow"),
  storageTitle: document.querySelector("#storage-title"),
  storageSubtitle: document.querySelector("#storage-subtitle"),
  storageFlowProgress: document.querySelector("#storage-flow-progress"),
  storageFlowMediaStep: document.querySelector("#storage-flow-media-step"),
  storageFlowImageStep: document.querySelector("#storage-flow-image-step"),
  storageFlowNetworkStep: document.querySelector("#storage-flow-network-step"),
  storageFlowSummaryStep: document.querySelector("#storage-flow-summary-step"),
  storageFlowFooter: document.querySelector("#storage-flow-footer"),
  storageFlowStatus: document.querySelector("#storage-flow-status"),
  storageFlowHint: document.querySelector("#storage-flow-hint"),
  storageNextButton: document.querySelector("#storage-next-button"),
  networkModal: document.querySelector("#network-modal"),
  closeNetworkModal: document.querySelector("#close-network-modal"),
  networkManagedNat: document.querySelector("#network-managed-nat"),
  networkPrivate: document.querySelector("#network-private"),
  networkUserNat: document.querySelector("#network-user-nat"),
  networkBridge: document.querySelector("#network-bridge"),
  networkManagedTap: document.querySelector("#network-managed-tap"),
  networkModel: document.querySelector("#network-model"),
  networkSubnet: document.querySelector("#network-subnet"),
  networkGateway: document.querySelector("#network-gateway"),
  networkPool: document.querySelector("#network-pool"),
  networkAttachForm: document.querySelector("#network-attach-form"),
  networkVm: document.querySelector("#network-vm"),
  networkProfile: document.querySelector("#network-profile"),
  networkId: document.querySelector("#network-id"),
  networkBridgeNameWrap: document.querySelector("#network-bridge-name-wrap"),
  networkBridgeName: document.querySelector("#network-bridge-name"),
  networkTapNameWrap: document.querySelector("#network-tap-name-wrap"),
  networkTapName: document.querySelector("#network-tap-name"),
  networkAttachButton: document.querySelector("#network-attach-button"),
  networkAttachmentList: document.querySelector("#network-attachment-list"),
  networkServiceForm: document.querySelector("#network-service-form"),
  networkServiceNetwork: document.querySelector("#network-service-network"),
  networkServicePreset: document.querySelector("#network-service-preset"),
  networkServiceProtocol: document.querySelector("#network-service-protocol"),
  networkServiceHostPort: document.querySelector("#network-service-host-port"),
  networkServiceGuestPort: document.querySelector("#network-service-guest-port"),
  networkServicePublish: document.querySelector("#network-service-publish"),
  networkAttachmentEmpty: document.querySelector("#network-attachment-empty"),
  networkEyebrow: document.querySelector("#network-eyebrow"),
  networkTitle: document.querySelector("#network-title"),
  networkSubtitle: document.querySelector("#network-subtitle"),
  networkFlowProgress: document.querySelector("#network-flow-progress"),
  networkFlowMediaStep: document.querySelector("#network-flow-media-step"),
  networkFlowImageStep: document.querySelector("#network-flow-image-step"),
  networkFlowFooter: document.querySelector("#network-flow-footer"),
  networkFlowStatus: document.querySelector("#network-flow-status"),
  networkFlowHint: document.querySelector("#network-flow-hint"),
  networkNextButton: document.querySelector("#network-next-button"),
  androidImagesModal: document.querySelector("#android-images-modal"),
  closeAndroidImagesModal: document.querySelector("#close-android-images-modal"),
  imageCenterTabs: document.querySelector("#image-center-tabs"),
  imageCenterIsoTab: document.querySelector("#image-center-iso-tab"),
  imageCenterAndroidTab: document.querySelector("#image-center-android-tab"),
  imageCenterIsoPanel: document.querySelector("#image-center-iso-panel"),
  imageCenterAndroidPanel: document.querySelector("#image-center-android-panel"),
  imageCenterIsoRefresh: document.querySelector("#image-center-iso-refresh"),
  imageCenterIsoList: document.querySelector("#image-center-iso-list"),
  imageCenterIsoEmpty: document.querySelector("#image-center-iso-empty"),
  imageCenterStandardIso: document.querySelector("#image-center-standard-iso"),
  imageCenterStandardIsoSelect: document.querySelector("#image-center-standard-iso-select"),
  imageCenterStandardIsoBadge: document.querySelector("#image-center-standard-iso-badge"),
  imageCenterStandardIsoLabel: document.querySelector("#image-center-standard-iso-label"),
  imageCenterStandardIsoState: document.querySelector("#image-center-standard-iso-state"),
  imageCenterStandardIsoAction: document.querySelector("#image-center-standard-iso-action"),
  imageCenterStandardIsoNote: document.querySelector("#image-center-standard-iso-note"),
  imageCenterIsoDownloadProgress: document.querySelector("#image-center-iso-download-progress"),
  androidImageDefineForm: document.querySelector("#android-image-define-form"),
  androidImagesRefresh: document.querySelector("#android-images-refresh"),
  androidImagesList: document.querySelector("#android-images-list"),
  androidImagesEmpty: document.querySelector("#android-images-empty"),
  androidImagePlan: document.querySelector("#android-image-plan"),
  androidImageQuickVm: document.querySelector("#android-image-quick-vm"),
  androidImageQuickVmNote: document.querySelector("#android-image-quick-vm-note"),
  androidImagesEyebrow: document.querySelector("#android-images-eyebrow"),
  androidImagesTitle: document.querySelector("#android-images-title"),
  androidImagesSubtitle: document.querySelector("#android-images-subtitle"),
  androidImageFlowProgress: document.querySelector("#android-image-flow-progress"),
  androidImageFlowFooter: document.querySelector("#android-image-flow-footer"),
  androidImageFlowStatus: document.querySelector("#android-image-flow-status"),
  androidImageFlowHint: document.querySelector("#android-image-flow-hint"),
  androidImageNextButton: document.querySelector("#android-image-next-button"),
  androidReleaseCatalog: document.querySelector("#android-release-catalog"),
  androidReleaseContext: document.querySelector("#android-release-context"),
  androidReleaseRefresh: document.querySelector("#android-release-refresh"),
  androidStandardReleaseSelect: document.querySelector("#android-standard-release-select"),
  androidStandardReleaseBadge: document.querySelector("#android-standard-release-badge"),
  androidStandardReleaseLabel: document.querySelector("#android-standard-release-label"),
  androidStandardReleaseState: document.querySelector("#android-standard-release-state"),
  androidStandardReleaseAction: document.querySelector("#android-standard-release-action"),
  androidStandardReleaseNote: document.querySelector("#android-standard-release-note"),
  androidStandardDownloadProgress: document.querySelector("#android-standard-download-progress"),
  installerMediaModal: document.querySelector("#installer-media-modal"),
  closeInstallerMediaModal: document.querySelector("#close-installer-media-modal"),
  installerMediaVm: document.querySelector("#installer-media-vm"),
  installerMediaPath: document.querySelector("#installer-media-path"),
  installerMediaPickButton: document.querySelector("#installer-media-pick-button"),
  installerMediaAttachButton: document.querySelector("#installer-media-attach-button"),
  installerMediaEjectButton: document.querySelector("#installer-media-eject-button"),
  installerMediaNextButton: document.querySelector("#installer-media-next-button"),
  installerMediaCurrent: document.querySelector("#installer-media-current"),
  installerMediaProvider: document.querySelector("#installer-media-provider"),
  installerMediaHostArchitecture: document.querySelector("#installer-media-host-architecture"),
  installerMediaVmArchitecture: document.querySelector("#installer-media-vm-architecture"),
  installerMediaFirmware: document.querySelector("#installer-media-firmware"),
  installerMediaRecommendedLabel: document.querySelector("#installer-media-recommended-label"),
  installerMediaOptionsToggle: document.querySelector("#installer-media-options-toggle"),
  installerMediaOptionsPanel: document.querySelector("#installer-media-options-panel"),
  installerMediaSourceSelect: document.querySelector("#installer-media-source-select"),
  installerMediaOptionNote: document.querySelector("#installer-media-option-note"),
  installerMediaSourceLabel: document.querySelector("#installer-media-source-label"),
  installerMediaSourceNote: document.querySelector("#installer-media-source-note"),
  installerMediaDownloadProgress: document.querySelector("#installer-media-download-progress"),
  installerMediaDownloadButton: document.querySelector("#installer-media-download-button"),
  installerMediaCancelDownloadButton: document.querySelector("#installer-media-cancel-download-button"),
  installerMediaAttachDownloadedButton: document.querySelector("#installer-media-attach-downloaded-button"),
  installerMediaOfficialPageButton: document.querySelector("#installer-media-official-page-button"),
  configurationCompleteModal: document.querySelector("#configuration-complete-modal"),
  closeConfigurationCompleteModal: document.querySelector("#close-configuration-complete-modal"),
  configurationCompleteDone: document.querySelector("#configuration-complete-done"),
  configurationCompleteSubtitle: document.querySelector("#configuration-complete-subtitle"),
  configurationCompleteMediaStep: document.querySelector("#configuration-complete-media-step"),
  configurationCompleteImageStep: document.querySelector("#configuration-complete-image-step"),
  configurationCompleteVm: document.querySelector("#configuration-complete-vm"),
  configurationCompleteDisk: document.querySelector("#configuration-complete-disk"),
  configurationCompleteNetwork: document.querySelector("#configuration-complete-network"),
  configurationCompleteMedia: document.querySelector("#configuration-complete-media"),
  configurationCompleteImage: document.querySelector("#configuration-complete-image"),
  logsModal: document.querySelector("#logs-modal"),
  closeLogsModal: document.querySelector("#close-logs-modal"),
  logsRefreshButton: document.querySelector("#logs-refresh-button"),
  logsSummary: document.querySelector("#logs-summary"),
  logsList: document.querySelector("#logs-list"),
  logsEmpty: document.querySelector("#logs-empty"),
  gpuModal: document.querySelector("#gpu-modal"),
  closeGpuModal: document.querySelector("#close-gpu-modal"),
  downloadSettingsForm: document.querySelector("#download-settings-form"),
  downloadInstallerMediaPath: document.querySelector("#download-installer-media-path"),
  downloadAndroidImagesPath: document.querySelector("#download-android-images-path"),
  downloadArtifactCachePath: document.querySelector("#download-artifact-cache-path"),
  downloadAndroidCiBaseUrl: document.querySelector("#download-android-ci-base-url"),
  downloadOfficialFallback: document.querySelector("#download-official-fallback"),
  downloadSettingsNote: document.querySelector("#download-settings-note"),
  downloadSettingsSave: document.querySelector("#download-settings-save"),
  downloadSourceSummary: document.querySelector("#download-source-summary"),
  gpuRequested: document.querySelector("#gpu-requested"),
  gpuEffective: document.querySelector("#gpu-effective"),
  gpuHostmem: document.querySelector("#gpu-hostmem"),
  gpuVulkan: document.querySelector("#gpu-vulkan"),
  gpuVirtio: document.querySelector("#gpu-virtio"),
  gpuVirgl: document.querySelector("#gpu-virgl"),
  gpuVenus: document.querySelector("#gpu-venus"),
  gpuRutabaga: document.querySelector("#gpu-rutabaga"),
  gpuGfxstream: document.querySelector("#gpu-gfxstream"),
  gpuAndroidGfxstream: document.querySelector("#gpu-android-gfxstream"),
  gpuWarning: document.querySelector("#gpu-warning"),
  gpuVulkanSummary: document.querySelector("#gpu-vulkan-summary"),
  hostSelector: document.querySelector("#host-selector"),
  hostEyebrow: document.querySelector("#host-eyebrow"),
  machineCount: document.querySelector("#machine-count"),
  machineGrid: document.querySelector("#machine-grid"),
  vmDetailEmpty: document.querySelector("#vm-detail-empty"),
  vmDetailPanel: document.querySelector("#vm-detail-panel"),
  taskDock: document.querySelector(".task-dock"),
  taskDockToggle: document.querySelector("#task-dock-toggle"),
  taskDockPanel: document.querySelector("#task-dock-panel"),
  taskDockCount: document.querySelector("#task-dock-count"),
  taskActiveCount: document.querySelector("#task-active-count"),
  taskErrorCount: document.querySelector("#task-error-count"),
  taskLatestLabel: document.querySelector("#task-latest-label"),
  taskSystemStatus: document.querySelector("#task-system-status"),
  emptyState: document.querySelector("#empty-state"),
  errorBanner: document.querySelector("#error-banner"),
  refreshButton: document.querySelector("#refresh-button"),
  newVmButton: document.querySelector("#new-vm-button"),
  connectionModal: document.querySelector("#connection-modal"),
  closeConnectionModal: document.querySelector("#close-connection-modal"),
  connectionVm: document.querySelector("#connection-vm"),
  connectionVmState: document.querySelector("#connection-vm-state"),
  connectionSubtitle: document.querySelector("#connection-subtitle"),
  connectionIp: document.querySelector("#connection-ip"),
  connectionNetwork: document.querySelector("#connection-network"),
  connectionUser: document.querySelector("#connection-user"),
  connectionStartVm: document.querySelector("#connection-start-vm"),
  connectionSshCommand: document.querySelector("#connection-ssh-command"),
  connectionSshTarget: document.querySelector("#connection-ssh-target"),
  connectionSshStatus: document.querySelector("#connection-ssh-status"),
  connectionSshHint: document.querySelector("#connection-ssh-hint"),
  connectionRdpCommand: document.querySelector("#connection-rdp-command"),
  connectionRdpTarget: document.querySelector("#connection-rdp-target"),
  connectionRdpStatus: document.querySelector("#connection-rdp-status"),
  connectionRdpHint: document.querySelector("#connection-rdp-hint"),
  prepareSshAccess: document.querySelector("#prepare-ssh-access"),
  prepareRdpAccess: document.querySelector("#prepare-rdp-access"),
  testSshConnection: document.querySelector("#test-ssh-connection"),
  openSshConnection: document.querySelector("#open-ssh-connection"),
  testRdpConnection: document.querySelector("#test-rdp-connection"),
  openRdpConnection: document.querySelector("#open-rdp-connection"),
  copySshCommand: document.querySelector("#copy-ssh-command"),
  copyRdpCommand: document.querySelector("#copy-rdp-command"),
  createModal: document.querySelector("#create-modal"),
  closeModal: document.querySelector("#close-modal"),
  cancelCreate: document.querySelector("#cancel-create"),
  createForm: document.querySelector("#create-form"),
  createConfigPanel: document.querySelector("#create-config-panel"),
  createCompletePanel: document.querySelector("#create-complete-panel"),
  createCompleteText: document.querySelector("#create-complete-text"),
  createSubmit: document.querySelector("#create-submit"),
  createBackButton: document.querySelector("#create-back-button"),
  createNextButton: document.querySelector("#create-next-button"),
  createAddDisk: document.querySelector("#create-add-disk"),
  createFinish: document.querySelector("#create-finish"),
  guestFamilySelect: document.querySelector("#guest-family-select"),
  guestProductSelect: document.querySelector("#guest-product-select"),
  guestReleaseSelect: document.querySelector("#guest-release-select"),
  guestProfileSelect: document.querySelector("#guest-profile-select"),
  guestArchitecture: document.querySelector("#guest-architecture"),
  guestTemplateInfo: document.querySelector("#guest-template-info"),
  guestRecommendation: document.querySelector("#guest-recommendation"),
  vmGuestProfile: document.querySelector("#vm-guest-profile"),
  vmGuestTemplateId: document.querySelector("#vm-guest-template-id"),
  vmName: document.querySelector("#vm-name"),
  vmId: document.querySelector("#vm-id"),
  vmCpu: document.querySelector("#vm-cpu"),
  vmMemory: document.querySelector("#vm-memory"),
  createSummary: document.querySelector("#create-summary"),
  createSelectedFamily: document.querySelector("#create-selected-family"),
  createSelectedProduct: document.querySelector("#create-selected-product"),
  createSelectedRelease: document.querySelector("#create-selected-release"),
  createSelectedProfile: document.querySelector("#create-selected-profile"),
  createSelectedArchitecture: document.querySelector("#create-selected-architecture"),
  createSelectedFirmware: document.querySelector("#create-selected-firmware"),
  createRecommendedDisk: document.querySelector("#create-recommended-disk"),
  createNextDisk: document.querySelector("#create-next-disk"),
  createNextMedia: document.querySelector("#create-next-media"),
  createNextNetwork: document.querySelector("#create-next-network"),
  createDiskSize: document.querySelector("#create-disk-size"),
  createDiskIdPreview: document.querySelector("#create-disk-id-preview"),
  createMediaIsoPanel: document.querySelector("#create-media-iso-panel"),
  createMediaAndroidPanel: document.querySelector("#create-media-android-panel"),
  createMediaMode: document.querySelector("#create-media-mode"),
  createMediaOfficialPanel: document.querySelector("#create-media-official-panel"),
  createMediaLocalPanel: document.querySelector("#create-media-local-panel"),
  createMediaLaterPanel: document.querySelector("#create-media-later-panel"),
  createMediaSourceSelect: document.querySelector("#create-media-source-select"),
  createMediaDownloadButton: document.querySelector("#create-media-download-button"),
  createMediaCancelButton: document.querySelector("#create-media-cancel-button"),
  createMediaDownloadProgress: document.querySelector("#create-media-download-progress"),
  createMediaSourceNote: document.querySelector("#create-media-source-note"),
  createMediaLocalPath: document.querySelector("#create-media-local-path"),
  createMediaPickButton: document.querySelector("#create-media-pick-button"),
  createAndroidImageSelect: document.querySelector("#create-android-image-select"),
  createAndroidImagesRefresh: document.querySelector("#create-android-images-refresh"),
  createAndroidDownloadProgress: document.querySelector("#create-android-download-progress"),
  createAndroidProgressCancel: document.querySelector("#create-android-progress-cancel"),
  createAndroidProgressLog: document.querySelector("#create-android-progress-log"),
  createAndroidImageNote: document.querySelector("#create-android-image-note"),
  createNetworkProfile: document.querySelector("#create-network-profile"),
  createNetworkId: document.querySelector("#create-network-id"),
  createNetworkBridgeWrap: document.querySelector("#create-network-bridge-wrap"),
  createNetworkTapWrap: document.querySelector("#create-network-tap-wrap"),
  createNetworkBridgeName: document.querySelector("#create-network-bridge-name"),
  createNetworkTapName: document.querySelector("#create-network-tap-name"),
  createReviewName: document.querySelector("#create-review-name"),
  createReviewId: document.querySelector("#create-review-id"),
  createReviewCpu: document.querySelector("#create-review-cpu"),
  createReviewMemory: document.querySelector("#create-review-memory"),
  createReviewDisk: document.querySelector("#create-review-disk"),
  createReviewMedia: document.querySelector("#create-review-media"),
  createReviewNetwork: document.querySelector("#create-review-network"),
  vmEditModal: document.querySelector("#vm-edit-modal"),
  vmEditForm: document.querySelector("#vm-edit-form"),
  closeVmEditModal: document.querySelector("#close-vm-edit-modal"),
  vmEditId: document.querySelector("#vm-edit-id"),
  vmEditName: document.querySelector("#vm-edit-name"),
  vmEditCpu: document.querySelector("#vm-edit-cpu"),
  vmEditMemory: document.querySelector("#vm-edit-memory"),
  snapshotModal: document.querySelector("#snapshot-modal"),
  closeSnapshotModal: document.querySelector("#close-snapshot-modal"),
  snapshotForm: document.querySelector("#snapshot-form"),
  snapshotVmLabel: document.querySelector("#snapshot-vm-label"),
  snapshotList: document.querySelector("#snapshot-list"),
  snapshotEmpty: document.querySelector("#snapshot-empty"),
  cloneModal: document.querySelector("#clone-modal"),
  closeCloneModal: document.querySelector("#close-clone-modal"),
  cancelClone: document.querySelector("#cancel-clone"),
  cloneForm: document.querySelector("#clone-form"),
  cloneSourceLabel: document.querySelector("#clone-source-label"),
  androidModal: document.querySelector("#android-modal"),
  closeAndroidModal: document.querySelector("#close-android-modal"),
  androidVmLabel: document.querySelector("#android-vm-label"),
  androidProfileStatus: document.querySelector("#android-profile-status"),
  androidDeviceStatus: document.querySelector("#android-device-status"),
  androidAbiStatus: document.querySelector("#android-abi-status"),
  androidSdkStatus: document.querySelector("#android-sdk-status"),
  androidGuestAgentStatus: document.querySelector("#android-guest-agent-status"),
  androidImageAssignmentStatus: document.querySelector("#android-image-assignment-status"),
  androidImageAssignmentRefresh: document.querySelector("#android-image-assignment-refresh"),
  androidImageSelect: document.querySelector("#android-image-select"),
  androidImageAssign: document.querySelector("#android-image-assign"),
  gameCatalogRefresh: document.querySelector("#game-catalog-refresh"),
  gameDetectButton: document.querySelector("#game-detect-button"),
  gameCatalogResult: document.querySelector("#game-catalog-result"),
  gameCatalogList: document.querySelector("#game-catalog-list"),
  gameCatalogEmpty: document.querySelector("#game-catalog-empty"),
  androidProfileForm: document.querySelector("#android-profile-form"),
  androidStatusButton: document.querySelector("#android-status-button"),
  androidReadyButton: document.querySelector("#android-ready-button"),
  androidDisplayButton: document.querySelector("#android-display-button"),
  androidApkForm: document.querySelector("#android-apk-form"),
  androidPackagesRefresh: document.querySelector("#android-packages-refresh"),
  androidPackageList: document.querySelector("#android-package-list"),
  androidPackageEmpty: document.querySelector("#android-package-empty"),
  androidTapForm: document.querySelector("#android-tap-form"),
  androidGamingInputButton: document.querySelector("#android-gaming-input-button"),
  gamingInputModal: document.querySelector("#gaming-input-modal"),
  closeGamingInputModal: document.querySelector("#close-gaming-input-modal"),
  gamingInputVmLabel: document.querySelector("#gaming-input-vm-label"),
  gamingCapKeyboard: document.querySelector("#gaming-cap-keyboard"),
  gamingCapMouse: document.querySelector("#gaming-cap-mouse"),
  gamingCapMultitouch: document.querySelector("#gaming-cap-multitouch"),
  gamingCapGamepad: document.querySelector("#gaming-cap-gamepad"),
  gamingInputProfileForm: document.querySelector("#gaming-input-profile-form"),
  gamingBindingAdd: document.querySelector("#gaming-binding-add"),
  gamingBindingList: document.querySelector("#gaming-binding-list"),
  gamingBindingEmpty: document.querySelector("#gaming-binding-empty"),
  gamingInputReset: document.querySelector("#gaming-input-reset")
};


function formatBytes(value) {
  const bytes = Number(value || 0);
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let current = bytes;
  let index = 0;
  while (current >= 1024 && index < units.length - 1) {
    current /= 1024;
    index += 1;
  }
  return `${current.toFixed(index >= 3 ? 1 : 0)} ${units[index]}`;
}

function safeStorageGet(key) {
  try {
    return window.localStorage.getItem(key);
  } catch (_) {
    return null;
  }
}

function safeStorageSet(key, value) {
  try {
    window.localStorage.setItem(key, value);
  } catch (_) {
    // UI preference persistence is optional.
  }
}

function restoreUiPreferences() {
  const storedView = safeStorageGet(UI_STORAGE_VIEW_MODE);
  const storedSort = safeStorageGet(UI_STORAGE_SORT_MODE);
  const storedSidebar = safeStorageGet(UI_STORAGE_SIDEBAR);
  const storedExpertMode = safeStorageGet(UI_STORAGE_EXPERT_MODE);
  uiState.vmViewMode = storedView === VM_VIEW_CARD ? VM_VIEW_CARD : VM_VIEW_COMPACT;
  uiState.vmSortMode = ["name-asc", "name-desc", "state", "profile"].includes(storedSort) ? storedSort : VM_SORT_NAME_ASC;
  uiState.sidebarCollapsed = storedSidebar === "true";
  uiState.expertMode = storedExpertMode === "true";
  elements.vmSortSelect.value = uiState.vmSortMode;
  setVmViewMode(uiState.vmViewMode, false);
  setSidebarCollapsed(uiState.sidebarCollapsed, false);
  setExpertMode(uiState.expertMode, false);
}

function setSidebarCollapsed(collapsed, persist = true) {
  uiState.sidebarCollapsed = Boolean(collapsed);
  elements.appShell.classList.toggle("sidebar-collapsed", uiState.sidebarCollapsed);
  if (persist) safeStorageSet(UI_STORAGE_SIDEBAR, String(uiState.sidebarCollapsed));
}

function toggleSidebar() {
  setSidebarCollapsed(!uiState.sidebarCollapsed);
}

function setExpertMode(enabled, persist = true) {
  uiState.expertMode = Boolean(enabled);
  elements.appShell.classList.toggle("expert-mode", uiState.expertMode);
  document.body.classList.toggle("expert-mode", uiState.expertMode);
  if (elements.expertModeButton) {
    elements.expertModeButton.textContent = uiState.expertMode ? "Standart Mod" : "Uzman Modu";
    elements.expertModeButton.title = uiState.expertMode ? "Standart gorunume don" : "Uzman gorunumunu ac";
    elements.expertModeButton.classList.toggle("active", uiState.expertMode);
  }
  if (persist) safeStorageSet(UI_STORAGE_EXPERT_MODE, String(uiState.expertMode));
  if (elements.androidImagesModal && !elements.androidImagesModal.classList.contains("hidden")) renderAndroidReleaseCatalog(uiState.androidImages);
}

function toggleExpertMode() {
  setExpertMode(!uiState.expertMode);
}

function setVmViewMode(mode, persist = true) {
  uiState.vmViewMode = mode === VM_VIEW_COMPACT ? VM_VIEW_COMPACT : VM_VIEW_CARD;
  const usesWorkspaceLibrary = elements.machineGrid.classList.contains("vm-library-list");
  elements.machineGrid.classList.toggle("compact-view", !usesWorkspaceLibrary && uiState.vmViewMode === VM_VIEW_COMPACT);
  elements.cardViewButton.classList.toggle("active", uiState.vmViewMode === VM_VIEW_CARD);
  elements.compactViewButton.classList.toggle("active", uiState.vmViewMode === VM_VIEW_COMPACT);
  if (persist) safeStorageSet(UI_STORAGE_VIEW_MODE, uiState.vmViewMode);
}

function setVmSortMode(mode, persist = true) {
  uiState.vmSortMode = mode || VM_SORT_NAME_ASC;
  if (persist) safeStorageSet(UI_STORAGE_SORT_MODE, uiState.vmSortMode);
  renderMachineViews();
}

function setVmQuickFilter(filter) {
  uiState.vmQuickFilter = filter || VM_FILTER_ALL;
  for (const button of elements.vmFilterGroup.querySelectorAll("[data-vm-filter]")) {
    button.classList.toggle("active", button.dataset.vmFilter === uiState.vmQuickFilter);
  }
  renderMachineViews();
}


function sortMachines(machines) {
  const sorted = [...machines];
  const byName = (left, right) => String(left.name || "").localeCompare(String(right.name || ""), "tr-TR", { sensitivity: "base" });
  if (uiState.vmSortMode === "name-desc") return sorted.sort((left, right) => byName(right, left));
  if (uiState.vmSortMode === "state") {
    return sorted.sort((left, right) => {
      const leftRank = VM_STATE_SORT_RANK[String(left.state || "").toLowerCase()] ?? 9;
      const rightRank = VM_STATE_SORT_RANK[String(right.state || "").toLowerCase()] ?? 9;
      return leftRank - rightRank || byName(left, right);
    });
  }
  if (uiState.vmSortMode === "profile") {
    return sorted.sort((left, right) => String(left.guest_profile || "").localeCompare(String(right.guest_profile || ""), "tr-TR") || byName(left, right));
  }
  return sorted.sort(byName);
}

function renderFleetSummary(machines) {
  const summary = machines.reduce((acc, machine) => {
    const state = String(machine.state || "").toLowerCase();
    if (state === STATE_RUNNING) acc.running += 1;
    if (state === STATE_STOPPED) acc.stopped += 1;
    if (state === "error") acc.error += 1;
    acc.snapshots += Number(machine.snapshot_count || 0);
    acc.disks += Number(machine.disk_count || 0);
    return acc;
  }, { running: 0, stopped: 0, error: 0, snapshots: 0, disks: 0 });
  elements.summaryTotalVms.textContent = String(machines.length);
  elements.summaryRunningVms.textContent = String(summary.running);
  elements.summaryStoppedVms.textContent = String(summary.stopped);
  elements.summaryErrorVms.textContent = String(summary.error);
  elements.summarySnapshots.textContent = String(summary.snapshots);
  elements.summaryDisks.textContent = String(summary.disks);
}

function showToast(title, detail, type = "info") {
  const toast = document.createElement("div");
  toast.className = `toast ${type}`;
  toast.innerHTML = `<div><strong>${escapeHtml(title)}</strong><span>${escapeHtml(detail || "")}</span></div>`;
  elements.toastStack.appendChild(toast);
  window.setTimeout(() => toast.remove(), TOAST_DURATION_MS);
}

function recordActivity(operation, target, status = "success", detail = "") {
  uiState.activities.unshift({
    id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
    timestamp: Date.now(),
    operation,
    target,
    status,
    detail
  });
  if (uiState.activities.length > ACTIVITY_LIMIT) uiState.activities.length = ACTIVITY_LIMIT;
  renderActivity();
}

function activeBackgroundTaskCount() {
  const androidCount = uiState.androidImages.filter((image) => image.state === "installing").length;
  return androidCount + uiState.imageCenterIsoDownloads.size;
}

function renderTaskDockStatus() {
  const activeCount = activeBackgroundTaskCount();
  const errorCount = uiState.activities.filter((activity) => activity.status === "error").length;
  const latest = uiState.activities[0] || null;
  if (elements.taskActiveCount) elements.taskActiveCount.textContent = String(activeCount);
  if (elements.taskErrorCount) elements.taskErrorCount.textContent = String(errorCount);
  if (elements.taskDockCount) elements.taskDockCount.textContent = String(uiState.activities.length);
  if (elements.taskLatestLabel) {
    elements.taskLatestLabel.textContent = latest ? `${latest.operation} - ${latest.target || "-"}` : "Islem yok";
  }
  if (elements.taskSystemStatus) {
    elements.taskSystemStatus.textContent = errorCount > 0 ? `${errorCount} sorun var` : (activeCount > 0 ? `${activeCount} gorev calisiyor` : "Sistem Hazir");
  }
  elements.taskDock?.classList.toggle("has-errors", errorCount > 0);
  elements.taskDock?.classList.toggle("has-active", activeCount > 0);
  elements.taskDock?.classList.toggle("has-activity", uiState.activities.length > 0);
}

function renderActivity() {
  elements.activityList.innerHTML = "";
  elements.activityEmpty.classList.toggle("hidden", uiState.activities.length !== 0);
  elements.clearActivityButton.disabled = uiState.activities.length === 0;
  renderTaskDockStatus();
  for (const activity of uiState.activities) {
    const row = document.createElement("div");
    row.className = "activity-item";
    const statusLabel = activity.status === "error" ? "Hata" : (activity.status === "info" ? "Bilgi" : "Basarili");
    row.innerHTML = `
      <div class="activity-item-main"><strong>${escapeHtml(activity.operation)}</strong><span>${escapeHtml(activity.target || "-")}</span></div>
      <div class="activity-item-meta"><span class="activity-status ${escapeAttribute(activity.status)}">${escapeHtml(statusLabel)}</span><time>${escapeHtml(new Date(activity.timestamp).toLocaleTimeString("tr-TR"))}</time></div>
      ${activity.detail ? `<small>${escapeHtml(activity.detail)}</small>` : ""}
    `;
    elements.activityList.appendChild(row);
  }
}

function toggleTaskDock(forceOpen = null) {
  uiState.taskDockOpen = forceOpen === null ? !uiState.taskDockOpen : Boolean(forceOpen);
  elements.taskDockPanel?.classList.toggle("hidden", !uiState.taskDockOpen);
  elements.taskDockToggle?.setAttribute("aria-expanded", uiState.taskDockOpen ? "true" : "false");
  elements.taskDock?.classList.toggle("open", uiState.taskDockOpen);
}
function clearActivity() {
  uiState.activities = [];
  renderActivity();
}

function setButtonBusy(button, busy) {
  if (!button) return;
  button.classList.toggle("is-busy", Boolean(busy));
  button.disabled = Boolean(busy);
}

function openShortcutsModal() {
  elements.shortcutsModal.classList.remove("hidden");
}

function closeShortcutsModal() {
  elements.shortcutsModal.classList.add("hidden");
}

function closeTopModal() {
  const visible = [...document.querySelectorAll(".modal-backdrop")].reverse().find((modal) => !modal.classList.contains("hidden"));
  if (!visible) return false;
  const closeButton = visible.querySelector('[aria-label="Kapat"], [id^="close-"]');
  if (closeButton) closeButton.click();
  else visible.classList.add("hidden");
  return true;
}

function syncModalOpenState() {
  const anyOpen = [...document.querySelectorAll(".modal-backdrop")].some((modal) => !modal.classList.contains("hidden"));
  document.body.classList.toggle("modal-open", anyOpen);
}

function populateStoppedVmSelect(select) {
  const stopped = uiState.machines.filter((machine) => {
    const state = String(machine.state || "").toLowerCase();
    const recoverableStartError = isRecoverableStartPreflightError(machine);
    return state === STATE_STOPPED || recoverableStartError;
  });
  select.innerHTML = "";
  for (const machine of stopped) {
    const option = document.createElement("option");
    option.value = machine.id;
    option.textContent = `${machine.name} (${machine.id})`;
    select.appendChild(option);
  }
  select.disabled = stopped.length === 0;
}

function populateNetworkVmSelect(select) {
  select.innerHTML = "";
  for (const machine of uiState.machines) {
    const option = document.createElement("option");
    option.value = machine.id;
    option.textContent = `${machine.name} (${machine.id}) - ${getStateLabel(machine.state)}`;
    select.appendChild(option);
  }
  select.disabled = uiState.machines.length === 0;
}

function selectedStorageMachine() {
  return uiState.machines.find((machine) => machine.id === elements.storageVm.value) || null;
}

function renderStorageDisks() {
  const machine = selectedStorageMachine();
  const disks = Array.isArray(machine?.disks) ? machine.disks : [];
  elements.storageDiskList.innerHTML = "";
  elements.storageDiskEmpty.classList.toggle("hidden", disks.length !== 0);
  for (const disk of disks) {
    const row = document.createElement("article");
    row.className = "snapshot-row";
    row.innerHTML = `<div><strong>${escapeHtml(disk.id)}</strong><span>${escapeHtml(String(disk.format || "-").toUpperCase())} / ${escapeHtml(disk.bus || "-")} / ${escapeHtml(formatBytes(disk.virtual_size_bytes))}${disk.boot_index === null || disk.boot_index === undefined ? "" : ` / boot ${disk.boot_index}`}</span><code>${escapeHtml(disk.relative_path || "")}</code></div><div class="snapshot-actions"><button class="ghost-button" type="button" data-storage-disk-action="resize" data-disk-id="${escapeAttribute(disk.id)}">Buyut</button><button class="vm-action stop" type="button" data-storage-disk-action="delete" data-disk-id="${escapeAttribute(disk.id)}">Sil</button></div>`;
    elements.storageDiskList.appendChild(row);
  }
}

async function handleStorageDiskAction(event) {
  const button = event.target.closest("[data-storage-disk-action]");
  if (!button) return;
  const vmId = elements.storageVm.value;
  const diskId = button.dataset.diskId;
  const machine = selectedStorageMachine();
  const disk = machine?.disks?.find((item) => item.id === diskId);
  if (!vmId || !diskId || !disk) return;
  if (uiState.storageFlowVmId) {
    showToast("Wizard aktif", "Disk yonetim islemlerini temel yapilandirma tamamlandiktan sonra kullanin.", "info");
    return;
  }
  try {
    if (button.dataset.storageDiskAction === "resize") {
      const currentGib = Math.ceil(Number(disk.virtual_size_bytes || 0) / (1024 ** 3));
      const raw = window.prompt(`Yeni disk boyutu (GiB). Mevcut: ${currentGib}`, String(Math.max(currentGib + 1, currentGib * 2)));
      if (raw === null) return;
      const sizeGib = Number(raw);
      if (!Number.isFinite(sizeGib) || sizeGib <= currentGib) {
        showError(`Yeni boyut ${currentGib} GiB degerinden buyuk olmalidir.`);
        return;
      }
      await invoke("resize_vm_disk", { request: { vm_id: vmId, disk_id: diskId, size_gib: sizeGib } });
      recordActivity("Disk Buyut", vmId, "success", `${diskId} -> ${sizeGib} GiB`);
      showToast("Disk buyutuldu", `${diskId} / ${sizeGib} GiB`, "success");
    } else {
      if (!window.confirm(`${vmId} / ${diskId} diski ve dosyasi kalici olarak silinsin mi?`)) return;
      await invoke("delete_vm_disk", { request: { vm_id: vmId, disk_id: diskId } });
      recordActivity("Disk Sil", vmId, "success", diskId);
      showToast("Disk silindi", diskId, "success");
    }
    await refreshDashboard();
    if ([...elements.storageVm.options].some((option) => option.value === vmId)) elements.storageVm.value = vmId;
    renderStorageDisks();
  } catch (error) {
    recordActivity("Disk Yonet", vmId, "error", String(error));
    showError(String(error));
  }
}

function setStorageFlowDiskReady(ready) {
  uiState.storageFlowDiskReady = Boolean(ready);
  elements.storageNextButton.disabled = !uiState.storageFlowDiskReady;
  if (!uiState.storageFlowVmId || !uiState.storageFlowNextStep) return;

  elements.storageFlowStatus.textContent = uiState.storageFlowDiskReady ? "Disk hazir" : "Disk bekleniyor";
  const targetLabel = uiState.storageFlowNextStep === "android-image"
    ? "Android Image atama"
    : (uiState.storageFlowNextStep === "installer-media" ? "Kurulum Medyasi" : "Ag yapilandirma");
  elements.storageFlowHint.textContent = uiState.storageFlowDiskReady
    ? `Disk hazir. Ileri ile ${targetLabel} adimina gecin.`
    : `Disk olustuktan sonra Ileri ile ${targetLabel} adimina gececeksiniz.`;
}

function configureStorageFlow(flowContext = null) {
  const active = Boolean(flowContext?.vmId && flowContext?.nextStep);
  uiState.storageFlowVmId = active ? flowContext.vmId : null;
  uiState.storageFlowNextStep = active ? flowContext.nextStep : null;
  uiState.storageFlowDiskReady = false;

  elements.storageFlowProgress.classList.toggle("hidden", !active);
  elements.storageFlowFooter.classList.toggle("hidden", !active);
  elements.storageNextButton.disabled = true;
  elements.storageNextButton.textContent = "Ileri";

  if (!active) {
    elements.storageEyebrow.textContent = "DEPOLAMA YONETIMI";
    elements.storageTitle.textContent = "Depolama";
    elements.storageSubtitle.textContent = "Sunucu kapasitesi, VM disk olusturma ve TurkuazVM tasinabilir imaj politikasini yonetin.";
    elements.storageFlowStatus.textContent = "Once diski olusturun.";
    elements.storageFlowHint.textContent = "Disk basariyla olusturuldugunda Ileri aktif olur.";
    elements.storageFlowMediaStep.classList.add("hidden");
    elements.storageFlowImageStep.classList.add("hidden");
    elements.storageFlowNetworkStep.textContent = "3 Ag";
    elements.storageFlowSummaryStep.textContent = "4 Ozet";
    return;
  }

  const androidNext = flowContext.nextStep === "android-image";
  const mediaNext = flowContext.nextStep === "installer-media";
  elements.storageEyebrow.textContent = "VM OLUSTURMA / ADIM 2";
  elements.storageTitle.textContent = "Disk Yapilandirmasi";
  elements.storageSubtitle.textContent = "VM olusturuldu. Onyukleme diskinizi olusturun ve siradaki yapilandirma adimina gecin.";
  elements.storageFlowMediaStep.classList.toggle("hidden", !mediaNext);
  elements.storageFlowImageStep.classList.toggle("hidden", !androidNext);
  elements.storageFlowNetworkStep.textContent = androidNext ? "4 Ag" : (mediaNext ? "4 Ag" : "3 Ag");
  elements.storageFlowSummaryStep.textContent = androidNext ? "5 Ozet" : (mediaNext ? "5 Ozet" : "4 Ozet");
  elements.storageNextButton.textContent = androidNext ? "Ileri: Android Image" : (mediaNext ? "Ileri: Kurulum Medyasi" : "Ileri: Ag");
  setStorageFlowDiskReady(false);
}

async function openStorageModal(preselectedVmId = null, preferredDiskSizeGib = null, flowContext = null) {
  activateNavigation("storage");
  configureStorageFlow(flowContext);
  try {
    const overview = await invoke("get_storage_overview");
    elements.storageDataRoot.textContent = overview.data_root;
    elements.storageFree.textContent = formatBytes(overview.available_bytes);
    elements.storageTotal.textContent = formatBytes(overview.total_bytes);
    elements.storageRuntimeFormat.textContent = overview.default_runtime_format.toUpperCase();
    elements.storageDiskSize.value = String(preferredDiskSizeGib || overview.default_disk_size_gib);
    elements.storagePortableFormat.textContent = `${overview.portable_image_extension} / ${overview.portable_container.toUpperCase()}`;
    elements.storageIsolation.textContent = overview.private_copy_default ? "OZEL KOPYA" : "PAYLASILAN";
    elements.storageNote.textContent = overview.qemu_img_available
      ? "qemu-img hazir. Yeni diskler VM'e ozel QCOW2 olarak olusturulur; .tvmimg tasinabilir paket politikasinda ozel kopya varsayilandir."
      : "qemu-img bulunamadi; disk olusturma kullanilamaz.";
    populateStoppedVmSelect(elements.storageVm);
    if (preselectedVmId && [...elements.storageVm.options].some((option) => option.value === preselectedVmId)) {
      elements.storageVm.value = preselectedVmId;
    }
    if (flowContext?.vmId) {
      elements.storageVm.disabled = true;
      const existingVm = uiState.machines.find((machine) => machine.id === flowContext.vmId);
      if (existingVm && Number(existingVm.disk_count || 0) > 0) setStorageFlowDiskReady(true);
    }
    renderStorageDisks();
    refreshStorageDiskSuggestion();
    elements.storageModal.classList.remove("hidden");
    clearError();
  } catch (error) {
    configureStorageFlow(null);
    activateNavigation("machines");
    showError(String(error));
  }
}

function closeStorageModal() {
  elements.storageModal.classList.add("hidden");
  elements.storageVm.disabled = false;
  configureStorageFlow(null);
  activateNavigation("machines");
}

async function openArtifactCacheModal() {
  activateNavigation("artifact-cache");
  elements.artifactCacheModal.classList.remove("hidden");
  await refreshArtifactCache();
}

function closeArtifactCacheModal() {
  elements.artifactCacheModal.classList.add("hidden");
  activateNavigation("machines");
}

async function refreshArtifactCache(showFailure = true) {
  try {
    const [overview, entries] = await Promise.all([
      invoke("get_artifact_cache_overview"),
      invoke("list_artifact_cache_entries")
    ]);
    uiState.artifactCacheEntries = Array.isArray(entries) ? entries : [];
    renderArtifactCacheOverview(overview);
    renderArtifactCacheEntries(uiState.artifactCacheEntries);
    elements.artifactCacheResult.textContent = `${uiState.artifactCacheEntries.length} kayit / ${formatBytes(overview.used_bytes)} kullaniliyor.`;
    clearError();
    return uiState.artifactCacheEntries;
  } catch (error) {
    uiState.artifactCacheEntries = [];
    renderArtifactCacheEntries([]);
    if (showFailure) showError(String(error));
    return [];
  }
}

function renderArtifactCacheOverview(overview) {
  elements.artifactCacheEnabled.textContent = overview.enabled ? "ACTIVE" : "DISABLED";
  elements.artifactCacheUsed.textContent = formatBytes(overview.used_bytes);
  elements.artifactCacheQuota.textContent = formatBytes(overview.quota_bytes);
  elements.artifactCacheCount.textContent = String(overview.artifact_count);
  elements.artifactCachePinned.textContent = String(overview.pinned_count);
  elements.artifactCacheMutable.textContent = String(overview.mutable_count);
  elements.artifactCacheIntegrity.textContent = overview.verify_on_hit ? "SHA-256 ON" : "SIZE ONLY";
  elements.artifactCacheOffline.textContent = overview.allow_stale_on_transient_error ? "TRANSIENT STALE" : "STRICT";
  elements.artifactCacheVerify.disabled = !overview.enabled;
  elements.artifactCacheRevalidateAll.disabled = !overview.enabled;
  elements.artifactCacheCleanup.disabled = !overview.enabled;
}

function renderArtifactCacheEntries(entries) {
  elements.artifactCacheList.innerHTML = "";
  elements.artifactCacheEmpty.classList.toggle("hidden", entries.length !== 0);
  for (const entry of entries) {
    const row = document.createElement("article");
    row.className = "snapshot-row cache-entry-row";
    const sourceMode = entry.immutable ? "IMMUTABLE" : "MUTABLE";
    const pinState = entry.pinned ? "PINNED" : "LRU";
    const validator = entry.etag || entry.last_modified || "validator yok";
    const revalidated = entry.last_revalidated_unix_ms ? formatTimestamp(entry.last_revalidated_unix_ms) : "-";
    const revalidateButton = entry.immutable
      ? ""
      : `<button class="ghost-button" type="button" data-cache-action="revalidate" data-source-key="${escapeAttribute(entry.source_key)}">Revalidate</button>`;
    row.innerHTML = `
      <div class="cache-entry-detail">
        <strong>${escapeHtml(entry.source_key)}</strong>
        <span>${escapeHtml(entry.source_url)}</span>
        <code>${escapeHtml(entry.sha256)} / ${escapeHtml(formatBytes(entry.size_bytes))}</code>
        <small>${sourceMode} / ${pinState} / ${escapeHtml(validator)} / Son revalidate ${escapeHtml(revalidated)}</small>
      </div>
      <div class="section-actions">
        <button class="ghost-button" type="button" data-cache-action="pin" data-source-key="${escapeAttribute(entry.source_key)}" data-pinned="${entry.pinned ? "false" : "true"}">${entry.pinned ? "Unpin" : "Pin"}</button>
        ${revalidateButton}
        <button class="vm-action stop" type="button" data-cache-action="remove" data-source-key="${escapeAttribute(entry.source_key)}">Sil</button>
      </div>`;
    elements.artifactCacheList.appendChild(row);
  }
}

async function handleArtifactCacheEntryAction(event) {
  const button = event.target.closest("[data-cache-action]");
  if (!button) return;
  const sourceKey = button.dataset.sourceKey;
  const action = button.dataset.cacheAction;
  if (!sourceKey) return;
  if (action === "remove" && !window.confirm(`Cache kaydi silinsin mi? ${sourceKey}`)) return;
  button.disabled = true;
  try {
    if (action === "pin") {
      await invoke("set_artifact_cache_pinned", {
        request: { source_key: sourceKey, pinned: button.dataset.pinned === "true" }
      });
      elements.artifactCacheResult.textContent = "Pin policy guncellendi.";
    } else if (action === "remove") {
      await invoke("remove_artifact_cache_entry", { request: { source_key: sourceKey } });
      elements.artifactCacheResult.textContent = "Cache kaydi silindi.";
    } else if (action === "revalidate") {
      const report = await invoke("revalidate_artifact_cache", { request: { source_key: sourceKey } });
      elements.artifactCacheResult.textContent = `${sourceKey}: ${report.state.toUpperCase()}${report.detail ? ` / ${report.detail}` : ""}`;
    }
    await refreshArtifactCache(false);
    clearError();
  } catch (error) {
    showError(String(error));
    button.disabled = false;
  }
}

async function fetchMutableArtifactCache(event) {
  event.preventDefault();
  const sourceKey = elements.artifactCacheFetchKey.value.trim();
  const sourceUrl = elements.artifactCacheFetchUrl.value.trim();
  if (!sourceKey || !sourceUrl) return;
  elements.artifactCacheFetch.disabled = true;
  try {
    const entry = await invoke("fetch_mutable_artifact_cache", {
      request: { source_key: sourceKey, source_url: sourceUrl, pinned: elements.artifactCacheFetchPinned.checked }
    });
    elements.artifactCacheResult.textContent = `Fetched: ${entry.source_key} / ${formatBytes(entry.size_bytes)} / ${entry.sha256}`;
    await refreshArtifactCache(false);
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    elements.artifactCacheFetch.disabled = false;
  }
}

async function verifyArtifactCache() {
  elements.artifactCacheVerify.disabled = true;
  try {
    const report = await invoke("verify_artifact_cache");
    elements.artifactCacheResult.textContent = `SHA verify: ${report.checked} kontrol / ${report.valid} valid / ${report.invalid} invalid.`;
    await refreshArtifactCache(false);
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    elements.artifactCacheVerify.disabled = false;
  }
}

async function cleanupArtifactCache() {
  elements.artifactCacheCleanup.disabled = true;
  try {
    const overview = await invoke("cleanup_artifact_cache");
    renderArtifactCacheOverview(overview);
    elements.artifactCacheResult.textContent = `Kota temizligi tamamlandi. ${formatBytes(overview.used_bytes)} kullaniliyor.`;
    await refreshArtifactCache(false);
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    elements.artifactCacheCleanup.disabled = false;
  }
}

async function revalidateAllArtifactCache() {
  elements.artifactCacheRevalidateAll.disabled = true;
  try {
    const report = await invoke("revalidate_all_artifact_cache");
    elements.artifactCacheResult.textContent = `Revalidate: ${report.checked} kayit / ${report.not_modified} current / ${report.remote_modified} remote-modified (LKG retained) / ${report.failed} failed / ${report.immutable} immutable.`;
    await refreshArtifactCache(false);
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    elements.artifactCacheRevalidateAll.disabled = false;
  }
}

async function handleStorageDiskCreate(event) {
  event.preventDefault();
  if (!elements.storageVm.value) return;
  try {
    await invoke("create_vm_disk", {
      request: {
        vm_id: elements.storageVm.value,
        disk_id: elements.storageDiskId.value.trim(),
        size_gib: Number(elements.storageDiskSize.value)
      }
    });
    recordActivity("Disk Olustur", elements.storageVm.value, "success", `${elements.storageDiskId.value.trim()} / ${elements.storageDiskSize.value} GiB`);
    const createdVmId = elements.storageVm.value;
    showToast("Disk olusturuldu", elements.storageDiskId.value.trim(), "success");
    await refreshDashboard();
    populateStoppedVmSelect(elements.storageVm);
    if ([...elements.storageVm.options].some((option) => option.value === createdVmId)) {
      elements.storageVm.value = createdVmId;
    }
    renderStorageDisks();
    refreshStorageDiskSuggestion();
    clearError();

    if (uiState.storageFlowVmId === createdVmId && uiState.storageFlowNextStep) {
      elements.storageVm.disabled = true;
      setStorageFlowDiskReady(true);
    }
  } catch (error) {
    recordActivity("Disk Olustur", elements.storageVm.value || "-", "error", String(error));
    showError(String(error));
  }
}

async function handleStorageFlowNext() {
  if (!uiState.storageFlowDiskReady || !uiState.storageFlowVmId || !uiState.storageFlowNextStep) return;
  const vmId = uiState.storageFlowVmId;
  const nextStep = uiState.storageFlowNextStep;
  closeStorageModal();

  if (nextStep === "android-image") {
    showToast("Siradaki adim", "Android image atayin.", "info");
    await openAndroidImagesForVm(vmId, { vmId, nextStep: "network" });
    return;
  }
  if (nextStep === "installer-media") {
    showToast("Siradaki adim", "Kurulum ISO dosyasini baglayin.", "info");
    openInstallerMediaModal(vmId);
    return;
  }

  showToast("Siradaki adim", "Ag yapilandirmasini tamamlayin.", "info");
  await openNetworkModal(vmId, { vmId, fromAndroid: false });
}

function selectedNetworkMachine() {
  return uiState.machines.find((machine) => machine.id === elements.networkVm.value) || null;
}

function renderNetworkAttachments() {
  const machine = selectedNetworkMachine();
  const networks = Array.isArray(machine?.networks) ? machine.networks : [];
  const machineState = String(machine?.state || "").toLowerCase();
  const canMutateNetwork = machineState === STATE_STOPPED;
  elements.networkAttachmentList.innerHTML = "";
  elements.networkAttachmentEmpty.classList.toggle("hidden", networks.length !== 0);
  elements.networkServiceNetwork.innerHTML = networks
    .filter((network) => [NETWORK_MODE_MANAGED_NAT, NETWORK_MODE_USER_NAT].includes(network.mode))
    .map((network) => `<option value="${escapeAttribute(network.id)}">${escapeHtml(network.id)}${network.ipv4_address ? ` - ${escapeHtml(network.ipv4_address)}` : ""}</option>`)
    .join("");
  elements.networkServiceForm.classList.toggle("hidden", elements.networkServiceNetwork.options.length === 0);
  elements.networkAttachButton.disabled = !canMutateNetwork;
  elements.networkServicePublish.disabled = !canMutateNetwork || elements.networkServiceNetwork.options.length === 0;

  for (const network of networks) {
    const row = document.createElement("article");
    row.className = "snapshot-row network-attachment-row";
    const mac = network.mac_address || "otomatik";
    const ip = network.ipv4_address ? `${network.ipv4_address}/${network.prefix_length || 24}` : "IP guest/DHCP";
    const fabric = network.fabric_id ? ` / Fabric ${network.fabric_id}` : "";
    const services = Array.isArray(network.published_services) ? network.published_services : [];
    const serviceHtml = services.length
      ? services.map((service) => `<button class="network-service-chip" type="button" data-network-unpublish="${escapeAttribute(network.id)}" data-protocol="${escapeAttribute(service.protocol)}" data-host-port="${Number(service.host_port)}" title="${canMutateNetwork ? "Yayini kaldir" : "Degisiklik icin VM durmalidir"}" ${canMutateNetwork ? "" : "disabled"}>${escapeHtml(service.protocol.toUpperCase())} ${Number(service.host_port)} -> ${Number(service.guest_port)} x</button>`).join("")
      : '<span class="network-service-empty">Yayin yok</span>';
    const canConnect = [NETWORK_MODE_MANAGED_NAT, NETWORK_MODE_USER_NAT].includes(String(network.mode || ""))
      || Boolean(network.ipv4_address)
      || services.length > 0;
    row.innerHTML = `<div><strong>${escapeHtml(network.id)}</strong><span>${escapeHtml(String(network.mode || "-").toUpperCase())}${escapeHtml(fabric)} / ${escapeHtml(ip)} / MAC ${escapeHtml(mac)}</span><div class="network-service-chips">${serviceHtml}</div></div><div class="snapshot-actions">${canConnect ? `<button class="vm-action secondary" type="button" data-network-connect="${escapeAttribute(network.id)}">Baglan</button>` : ""}<button class="vm-action stop" type="button" data-network-detach="${escapeAttribute(network.id)}" ${canMutateNetwork ? "" : "disabled"}>Kaldir</button></div>`;
    elements.networkAttachmentList.appendChild(row);
  }
}

async function handleNetworkDetach(event) {
  const button = event.target.closest("[data-network-detach]");
  if (!button) return;
  const vmId = elements.networkVm.value;
  const networkId = button.dataset.networkDetach;
  if (!vmId || !networkId) return;
  if (uiState.networkFlowVmId) {
    showToast("Wizard aktif", "Ag kaldirma islemini temel yapilandirma tamamlandiktan sonra kullanin.", "info");
    return;
  }
  if (!window.confirm(`${vmId} uzerinden ${networkId} ag baglantisi kaldirilsin mi?`)) return;
  try {
    await invoke("detach_vm_network", { request: { vm_id: vmId, network_id: networkId } });
    recordActivity("Ag Kaldir", vmId, "success", networkId);
    showToast("Ag kaldirildi", networkId, "success");
    await refreshDashboard();
    if ([...elements.networkVm.options].some((option) => option.value === vmId)) elements.networkVm.value = vmId;
    renderNetworkAttachments();
  } catch (error) {
    recordActivity("Ag Kaldir", vmId, "error", String(error));
    showError(String(error));
  }
}

function setNetworkFlowReady(ready) {
  uiState.networkFlowReady = Boolean(ready);
  elements.networkNextButton.disabled = !uiState.networkFlowReady;
  if (!uiState.networkFlowVmId) return;
  elements.networkFlowStatus.textContent = uiState.networkFlowReady ? "Ag hazir" : "Ag bekleniyor";
  elements.networkFlowHint.textContent = uiState.networkFlowReady
    ? "Ag hazir. Ileri ile yapilandirma ozetine gecin."
    : "Turkuaz NAT veya secilen ag profili eklendiginde Ileri aktif olur.";
}

function configureNetworkFlow(flowContext = null) {
  const active = Boolean(flowContext?.vmId);
  uiState.networkFlowVmId = active ? flowContext.vmId : null;
  uiState.networkFlowReady = false;
  uiState.networkFlowFromAndroid = active && Boolean(flowContext.fromAndroid);

  elements.networkFlowProgress.classList.toggle("hidden", !active);
  elements.networkFlowFooter.classList.toggle("hidden", !active);
  elements.networkFlowImageStep.classList.toggle("hidden", !uiState.networkFlowFromAndroid);
  const networkFlowMachine = active ? uiState.machines.find((machine) => machine.id === flowContext.vmId) : null;
  elements.networkFlowMediaStep.classList.toggle("hidden", !active || uiState.networkFlowFromAndroid || !networkFlowMachine?.installer_media);
  elements.networkNextButton.disabled = true;

  if (!active) {
    elements.networkEyebrow.textContent = "AG YONETIMI";
    elements.networkTitle.textContent = "Aglar";
    elements.networkSubtitle.textContent = "Turkuaz NAT, Private, Bridge ve Legacy NAT profillerini Engine network katmanindan yonetir.";
    elements.networkFlowStatus.textContent = "Once ag baglantisini ekleyin.";
    elements.networkFlowHint.textContent = "Ag basariyla eklendiginde Ileri aktif olur.";
    return;
  }

  const flowMachine = uiState.machines.find((machine) => machine.id === uiState.networkFlowVmId);
  const hasInstallerMedia = Boolean(flowMachine?.installer_media);
  elements.networkEyebrow.textContent = uiState.networkFlowFromAndroid || hasInstallerMedia
    ? "VM OLUSTURMA / ADIM 4"
    : "VM OLUSTURMA / ADIM 3";
  elements.networkTitle.textContent = "Ag Yapilandirmasi";
  elements.networkSubtitle.textContent = "VM icin varsayilan Turkuaz NAT baglantisini ekleyin ve son ozete gecin.";
  setNetworkFlowReady(false);
}

async function openNetworkModal(preselectedVmId = null, flowContext = null) {
  activateNavigation("network");
  configureNetworkFlow(flowContext);
  try {
    const overview = await invoke("get_network_overview");
    elements.networkManagedNat.textContent = overview.managed_nat ? "YES" : "NO";
    elements.networkPrivate.textContent = overview.private_network ? "YES" : "NO";
    elements.networkUserNat.textContent = overview.user_nat ? "YES" : "NO";
    elements.networkBridge.textContent = overview.bridge ? "YES" : "NO";
    elements.networkManagedTap.textContent = overview.managed_tap ? "YES" : "NO";
    elements.networkModel.textContent = overview.default_device_model;
    elements.networkSubnet.textContent = overview.managed_nat_subnet || "-";
    elements.networkGateway.textContent = overview.managed_nat_gateway || "-";
    elements.networkPool.textContent = overview.managed_nat_pool || "-";
    elements.networkProfile.value = overview.default_profile || "managed_nat";
    updateNetworkProfileFields();
    populateNetworkVmSelect(elements.networkVm);
    if (preselectedVmId && [...elements.networkVm.options].some((option) => option.value === preselectedVmId)) {
      elements.networkVm.value = preselectedVmId;
    }
    if (flowContext?.vmId) {
      elements.networkVm.disabled = true;
      const existingVm = uiState.machines.find((machine) => machine.id === flowContext.vmId);
      if (existingVm && Number(existingVm.network_count || 0) > 0) setNetworkFlowReady(true);
    }
    renderNetworkAttachments();
    refreshNetworkIdSuggestion();
    elements.networkModal.classList.remove("hidden");
    clearError();
  } catch (error) {
    configureNetworkFlow(null);
    activateNavigation("machines");
    showError(String(error));
  }
}

function closeNetworkModal() {
  elements.networkModal.classList.add("hidden");
  elements.networkVm.disabled = false;
  configureNetworkFlow(null);
  activateNavigation("machines");
}

async function handleNetworkAttach(event) {
  event.preventDefault();
  const vmId = elements.networkVm.value;
  const networkId = elements.networkId.value.trim();
  if (!vmId || !networkId) return;
  setButtonBusy(elements.networkAttachButton, true);
  try {
    const profile = elements.networkProfile.value;
    await invoke("attach_network_profile", { request: {
      vm_id: vmId, network_id: networkId, profile,
      bridge_name: profile === "bridge" ? (elements.networkBridgeName.value.trim() || null) : null,
      tap_name: profile === "bridge" ? (elements.networkTapName.value.trim() || null) : null,
    } });
    recordActivity("Ag Ekle", vmId, "success", `${networkId} / ${profile}`);
    showToast("Ag eklendi", networkId, "success");
    await refreshDashboard();
    populateNetworkVmSelect(elements.networkVm);
    if (uiState.networkFlowVmId === vmId) {
      if ([...elements.networkVm.options].some((option) => option.value === vmId)) elements.networkVm.value = vmId;
      elements.networkVm.disabled = true;
      setNetworkFlowReady(true);
    }
    renderNetworkAttachments();
    refreshNetworkIdSuggestion();
    clearError();
  } catch (error) {
    recordActivity("Ag Ekle", vmId || "-", "error", String(error));
    showError(String(error));
  } finally {
    setButtonBusy(elements.networkAttachButton, false);
    if (uiState.networkFlowVmId) elements.networkVm.disabled = true;
  }
}

function updateNetworkProfileFields() {
  const bridge = elements.networkProfile.value === "bridge";
  elements.networkBridgeNameWrap.classList.toggle("hidden", !bridge);
  elements.networkTapNameWrap.classList.toggle("hidden", !bridge);
}

function usedPublishedHostPorts(protocol = "tcp") {
  const used = new Set();
  for (const machine of uiState.machines || []) {
    for (const network of machine.networks || []) {
      for (const service of network.published_services || []) {
        if (String(service.protocol || "").toLowerCase() === protocol) used.add(Number(service.host_port));
      }
    }
  }
  return used;
}

function nextAvailableHostPort(preferred, protocol = "tcp") {
  const used = usedPublishedHostPorts(protocol);
  for (let port = Number(preferred); port <= 65535; port += 1) {
    if (!used.has(port)) return port;
  }
  return Number(preferred);
}

function applyNetworkServicePreset() {
  const preset = elements.networkServicePreset.value;
  const values = { ssh: [2222, 22], http: [8080, 80], https: [8443, 443], rdp: [33890, 3389] };
  if (!values[preset]) return;
  elements.networkServiceProtocol.value = "tcp";
  elements.networkServiceHostPort.value = String(nextAvailableHostPort(values[preset][0], "tcp"));
  elements.networkServiceGuestPort.value = String(values[preset][1]);
}

async function handleNetworkServicePublish(event) {
  event.preventDefault();
  const vmId = elements.networkVm.value;
  const networkId = elements.networkServiceNetwork.value;
  if (!vmId || !networkId) return;
  const request = { vm_id: vmId, network_id: networkId, protocol: elements.networkServiceProtocol.value, host_port: Number(elements.networkServiceHostPort.value), guest_port: Number(elements.networkServiceGuestPort.value) };
  try {
    await invoke("publish_vm_service", { request });
    showToast("Servis yayinlandi", `${request.host_port} -> ${request.guest_port}`, "success");
    await refreshDashboard();
    if ([...elements.networkVm.options].some((option) => option.value === vmId)) elements.networkVm.value = vmId;
    renderNetworkAttachments();
  } catch (error) { showError(String(error)); }
}

async function handleNetworkAttachmentAction(event) {
  const connect = event.target.closest("[data-network-connect]");
  if (connect) {
    const machine = selectedNetworkMachine();
    const network = (machine?.networks || []).find((item) => item.id === connect.dataset.networkConnect);
    if (machine && network) openConnectionModal(machine, network);
    return;
  }
  const unpublish = event.target.closest("[data-network-unpublish]");
  if (unpublish) {
    const vmId = elements.networkVm.value;
    try {
      await invoke("unpublish_vm_service", { request: { vm_id: vmId, network_id: unpublish.dataset.networkUnpublish, protocol: unpublish.dataset.protocol, host_port: Number(unpublish.dataset.hostPort) } });
      await refreshDashboard();
      if ([...elements.networkVm.options].some((option) => option.value === vmId)) elements.networkVm.value = vmId;
      renderNetworkAttachments();
    } catch (error) { showError(String(error)); }
    return;
  }
  await handleNetworkDetach(event);
}

function openConnectionModal(machine, selectedNetwork = null) {
  const networks = machine?.networks || [];
  const network = selectedNetwork
    || networks.find((item) => (item.published_services || []).length > 0)
    || networks.find((item) => [NETWORK_MODE_MANAGED_NAT, NETWORK_MODE_USER_NAT].includes(String(item.mode || "")))
    || networks.find((item) => item.ipv4_address);
  if (!machine || !network) {
    showToast("Ag bekleniyor", "Baglanti icin uygun ag bulunamadi.", "info");
    return;
  }

  const services = Array.isArray(network.published_services) ? network.published_services : [];
  const qemuNat = [NETWORK_MODE_MANAGED_NAT, NETWORK_MODE_USER_NAT].includes(String(network.mode || ""));
  const ssh = services.find((service) => service.protocol === NETWORK_PROTOCOL_TCP && Number(service.guest_port) === SSH_GUEST_PORT);
  const rdp = services.find((service) => service.protocol === NETWORK_PROTOCOL_TCP && Number(service.guest_port) === RDP_GUEST_PORT);
  const vmState = String(machine.state || "").toLowerCase();
  const activeHost = uiState.hosts.find((host) => host.active) || null;
  const remoteNat = qemuNat && activeHost?.mode === HOST_MODE_REMOTE;
  const storedUser = safeStorageGet(`${CONNECTION_USER_STORAGE_PREFIX}${machine.id}`) || "";

  elements.connectionVm.textContent = machine.name || machine.id;
  elements.connectionVmState.textContent = getStateLabel(vmState);
  elements.connectionIp.textContent = network.ipv4_address || (qemuNat ? "QEMU DHCP" : "-");
  elements.connectionNetwork.textContent = network.id;
  elements.connectionUser.value = storedUser;

  elements.connectionModal.dataset.vmId = machine.id;
  elements.connectionModal.dataset.vmState = vmState;
  elements.connectionModal.dataset.networkId = network.id;
  elements.connectionModal.dataset.mode = qemuNat ? CONNECTION_MODE_QEMU_NAT : CONNECTION_MODE_DIRECT;
  elements.connectionModal.dataset.remoteNat = remoteNat ? "true" : "false";
  elements.connectionModal.dataset.ip = network.ipv4_address || "";
  elements.connectionModal.dataset.sshHost = !remoteNat && ssh ? normalizePublishedHost(ssh.host_ip) : "";
  elements.connectionModal.dataset.sshPort = !remoteNat && ssh ? String(ssh.host_port) : "";
  elements.connectionModal.dataset.rdpHost = !remoteNat && rdp ? normalizePublishedHost(rdp.host_ip) : "";
  elements.connectionModal.dataset.rdpPort = !remoteNat && rdp ? String(rdp.host_port) : "";

  elements.connectionSubtitle.textContent = remoteNat
    ? "Remote Engine Turkuaz NAT loopback portlari bu masaustunden dogrudan kullanilamaz; Bridge/Private veya remote tunnel gerekir."
    : qemuNat
      ? "QEMU NAT guest IP adresine hosttan dogrudan baglanmaz; Baglanti Merkezi gerekli localhost yayinini kullanir."
      : "Bu ag profili guest IP adresine dogrudan baglanti saglar.";

  updateConnectionCommands();
  elements.connectionModal.classList.remove("hidden");
}

function normalizePublishedHost(hostIp) {
  const value = String(hostIp || "").trim();
  return !value || value === "0.0.0.0" ? "127.0.0.1" : value;
}

function connectionTarget(kind) {
  const mode = elements.connectionModal.dataset.mode || CONNECTION_MODE_DIRECT;
  const ip = elements.connectionModal.dataset.ip || "";
  if (mode === CONNECTION_MODE_QEMU_NAT) {
    const host = elements.connectionModal.dataset[`${kind}Host`] || "";
    const port = Number(elements.connectionModal.dataset[`${kind}Port`] || 0);
    return host && port > 0 ? { host, port } : null;
  }
  const guestPort = kind === "ssh" ? SSH_GUEST_PORT : RDP_GUEST_PORT;
  return ip ? { host: ip, port: guestPort } : null;
}

function setConnectionStatus(element, label, state = "") {
  element.textContent = label;
  element.classList.remove(CONNECTION_STATUS_READY, CONNECTION_STATUS_WARNING, CONNECTION_STATUS_ERROR);
  if (state) element.classList.add(state);
}

function closeConnectionModal() {
  elements.connectionModal.classList.add("hidden");
}

function updateConnectionCommands() {
  const mode = elements.connectionModal.dataset.mode || CONNECTION_MODE_DIRECT;
  const vmState = elements.connectionModal.dataset.vmState || "";
  const vmStopped = vmState === STATE_STOPPED;
  const vmRunning = vmState === STATE_RUNNING;
  const vmCanReconfigure = vmStopped || vmRunning;
  const user = elements.connectionUser.value.trim();
  const remoteNat = elements.connectionModal.dataset.remoteNat === "true";
  const sshTarget = connectionTarget("ssh");
  const rdpTarget = connectionTarget("rdp");
  const qemuNat = mode === CONNECTION_MODE_QEMU_NAT;

  elements.prepareSshAccess.classList.toggle("hidden", remoteNat || !qemuNat || Boolean(sshTarget));
  elements.prepareRdpAccess.classList.toggle("hidden", remoteNat || !qemuNat || Boolean(rdpTarget));
  elements.prepareSshAccess.disabled = !vmCanReconfigure;
  elements.prepareRdpAccess.disabled = !vmCanReconfigure;
  elements.connectionStartVm.classList.toggle("hidden", !vmStopped || (!sshTarget && !rdpTarget));

  if (remoteNat) {
    elements.connectionSshTarget.textContent = "Remote Engine loopback";
    elements.connectionSshCommand.textContent = "Remote Turkuaz NAT icin dogrudan localhost baglantisi kullanilamaz";
    setConnectionStatus(elements.connectionSshStatus, "Remote tunnel gerekli", CONNECTION_STATUS_WARNING);
    elements.testSshConnection.disabled = true;
    elements.openSshConnection.disabled = true;
    elements.copySshCommand.disabled = true;
    elements.connectionSshHint.textContent = "Remote Engine icin Bridge/Private ag veya guvenli bir SSH tunnel tasarimi kullanilmalidir.";
    elements.connectionRdpTarget.textContent = "Remote Engine loopback";
    elements.connectionRdpCommand.textContent = "Remote Turkuaz NAT icin dogrudan localhost baglantisi kullanilamaz";
    setConnectionStatus(elements.connectionRdpStatus, "Remote tunnel gerekli", CONNECTION_STATUS_WARNING);
    elements.testRdpConnection.disabled = true;
    elements.openRdpConnection.disabled = true;
    elements.copyRdpCommand.disabled = true;
    elements.connectionRdpHint.textContent = "Remote Engine icin Bridge/Private ag veya guvenli bir remote tunnel gerekir.";
    return;
  }

  if (sshTarget) {
    elements.connectionSshTarget.textContent = `${sshTarget.host}:${sshTarget.port}`;
    elements.connectionSshCommand.textContent = user
      ? `ssh -p ${sshTarget.port} ${user}@${sshTarget.host}`
      : "SSH kullanici adi girin";
    setConnectionStatus(elements.connectionSshStatus, "Hedef hazir", CONNECTION_STATUS_READY);
    elements.testSshConnection.disabled = false;
    elements.openSshConnection.disabled = !user;
    elements.copySshCommand.disabled = !user;
    elements.connectionSshHint.textContent = "Port testi ag erisimini kontrol eder; SSH servisi guest icinde ayrica kurulu ve calisiyor olmalidir.";
  } else {
    elements.connectionSshTarget.textContent = qemuNat ? "Host yayini yok" : "Guest IP bulunamadi";
    elements.connectionSshCommand.textContent = qemuNat ? "SSH erisimi icin host port yayini gerekli" : "Guest IP bulunamadi";
    setConnectionStatus(elements.connectionSshStatus, qemuNat ? "Yayin gerekli" : "Hedef yok", CONNECTION_STATUS_WARNING);
    elements.testSshConnection.disabled = true;
    elements.openSshConnection.disabled = true;
    elements.copySshCommand.disabled = true;
    elements.connectionSshHint.textContent = qemuNat
      ? (vmRunning
          ? "SSH Erisimini Hazirla VM'yi kisa sure durdurur, localhost yayinini ekler ve VM'yi yeniden baslatir."
          : vmStopped
            ? "SSH Erisimini Hazirla ile uygun localhost portu otomatik secilir."
            : "SSH yayini icin VM calisiyor veya kapali durumda olmali.")
      : "Dogrudan baglanti icin guest IPv4 adresi gereklidir.";
  }

  if (rdpTarget) {
    elements.connectionRdpTarget.textContent = `${rdpTarget.host}:${rdpTarget.port}`;
    elements.connectionRdpCommand.textContent = `mstsc /v:${rdpTarget.host}:${rdpTarget.port}`;
    setConnectionStatus(elements.connectionRdpStatus, "Hedef hazir", CONNECTION_STATUS_READY);
    elements.testRdpConnection.disabled = false;
    elements.openRdpConnection.disabled = false;
    elements.copyRdpCommand.disabled = false;
    elements.connectionRdpHint.textContent = "Port testi ag erisimini kontrol eder; Windows guest icinde Uzak Masaustu ayrica etkin olmalidir.";
  } else {
    elements.connectionRdpTarget.textContent = qemuNat ? "Host yayini yok" : "Guest IP bulunamadi";
    elements.connectionRdpCommand.textContent = qemuNat ? "RDP erisimi icin host port yayini gerekli" : "Guest IP bulunamadi";
    setConnectionStatus(elements.connectionRdpStatus, qemuNat ? "Yayin gerekli" : "Hedef yok", CONNECTION_STATUS_WARNING);
    elements.testRdpConnection.disabled = true;
    elements.openRdpConnection.disabled = true;
    elements.copyRdpCommand.disabled = true;
    elements.connectionRdpHint.textContent = qemuNat
      ? (vmRunning
          ? "RDP Erisimini Hazirla VM'yi kisa sure durdurur, localhost yayinini ekler ve VM'yi yeniden baslatir."
          : vmStopped
            ? "RDP Erisimini Hazirla ile uygun localhost portu otomatik secilir."
            : "RDP yayini icin VM calisiyor veya kapali durumda olmali.")
      : "Dogrudan baglanti icin guest IPv4 adresi gereklidir.";
  }
}

async function prepareConnectionAccess(kind) {
  const vmId = elements.connectionModal.dataset.vmId || "";
  const networkId = elements.connectionModal.dataset.networkId || "";
  if (!vmId || !networkId) return;

  const button = kind === "ssh" ? elements.prepareSshAccess : elements.prepareRdpAccess;
  const command = kind === "ssh" ? "prepare_vm_ssh_access" : "prepare_vm_rdp_access";
  const label = kind === "ssh" ? "SSH" : "RDP";
  const vmState = elements.connectionModal.dataset.vmState || "";
  const restartsRunningVm = vmState === STATE_RUNNING;
  if (restartsRunningVm && !window.confirm(
    `${label} erisimi icin VM kisa sure durdurulacak, localhost port yayini eklenecek ve VM yeniden baslatilacak. Devam edilsin mi?`
  )) return;

  setButtonBusy(button, true);
  try {
    const machine = await invoke(command, { request: { vm_id: vmId, network_id: networkId } });
    const network = (machine.networks || []).find((item) => item.id === networkId);
    if (!network) throw new Error("Guncellenen ag bilgisi bulunamadi");
    recordActivity(`${label} Erisimi Hazirla`, machine.name || vmId, "success", networkId);
    showToast(
      `${label} erisimi hazirlandi`,
      restartsRunningVm ? "Host portu eklendi ve VM yeniden baslatildi." : "Host portu otomatik secildi.",
      "success"
    );
    await refreshDashboard();
    const refreshed = uiState.machines.find((item) => item.id === vmId) || machine;
    const refreshedNetwork = (refreshed.networks || []).find((item) => item.id === networkId) || network;
    openConnectionModal(refreshed, refreshedNetwork);
    clearError();
  } catch (error) {
    recordActivity(`${label} Erisimi Hazirla`, vmId, "error", String(error));
    showError(String(error));
  } finally {
    setButtonBusy(button, false);
  }
}

async function testConnection(kind) {
  const target = connectionTarget(kind);
  if (!target) return;
  const status = kind === "ssh" ? elements.connectionSshStatus : elements.connectionRdpStatus;
  const label = kind === "ssh" ? "SSH" : "RDP";
  const button = kind === "ssh" ? elements.testSshConnection : elements.testRdpConnection;
  setButtonBusy(button, true);
  setConnectionStatus(status, "Test ediliyor", CONNECTION_STATUS_WARNING);
  try {
    const result = await invoke("test_tcp_connection", { request: target });
    if (result.reachable) {
      setConnectionStatus(status, "Erisilebilir", CONNECTION_STATUS_READY);
      showToast(`${label} portu erisilebilir`, `${result.host}:${result.port}`, "success");
    } else {
      setConnectionStatus(status, "Erisilemiyor", CONNECTION_STATUS_ERROR);
      showToast(`${label} portu kapali`, "Guest servisi kurulu veya aktif olmayabilir.", "error");
    }
  } catch (error) {
    setConnectionStatus(status, "Test hatasi", CONNECTION_STATUS_ERROR);
    showError(String(error));
  } finally {
    setButtonBusy(button, false);
  }
}

async function startConnectionVm() {
  const vmId = elements.connectionModal.dataset.vmId || "";
  const networkId = elements.connectionModal.dataset.networkId || "";
  if (!vmId) return;
  setButtonBusy(elements.connectionStartVm, true);
  try {
    await invoke("start_vm", { vmId });
    showToast("VM baslatildi", vmId, "success");
    await refreshDashboard();
    const machine = uiState.machines.find((item) => item.id === vmId);
    const network = (machine?.networks || []).find((item) => item.id === networkId) || null;
    if (machine) openConnectionModal(machine, network);
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(elements.connectionStartVm, false);
  }
}

async function openSshConnection() {
  const target = connectionTarget("ssh");
  const username = elements.connectionUser.value.trim();
  if (!target || !username) return;
  safeStorageSet(`${CONNECTION_USER_STORAGE_PREFIX}${elements.connectionModal.dataset.vmId || "default"}`, username);
  setButtonBusy(elements.openSshConnection, true);
  try {
    await invoke("open_ssh_connection", { request: { username, host: target.host, port: target.port } });
    showToast("SSH istemcisi acildi", `${username}@${target.host}:${target.port}`, "success");
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(elements.openSshConnection, false);
  }
}

async function openRdpConnection() {
  const target = connectionTarget("rdp");
  if (!target) return;
  setButtonBusy(elements.openRdpConnection, true);
  try {
    await invoke("open_rdp_connection", { request: target });
    showToast("RDP istemcisi acildi", `${target.host}:${target.port}`, "success");
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(elements.openRdpConnection, false);
  }
}

async function copyConnectionCommand(element) {
  try {
    await navigator.clipboard.writeText(element.textContent);
    showToast("Kopyalandi", element.textContent, "success");
  } catch (error) {
    showError(String(error));
  }
}

async function handleNetworkFlowNext() {
  if (!uiState.networkFlowReady || !uiState.networkFlowVmId) return;
  const vmId = uiState.networkFlowVmId;
  const fromAndroid = uiState.networkFlowFromAndroid;
  closeNetworkModal();
  await openConfigurationCompleteModal(vmId, { fromAndroid });
}

function setInstallerMediaReady(ready) {
  uiState.installerMediaReady = Boolean(ready);
  elements.installerMediaNextButton.disabled = !uiState.installerMediaReady;
}

function stopInstallerMediaPolling() {
  if (uiState.installerMediaPollTimer !== null) {
    window.clearInterval(uiState.installerMediaPollTimer);
    uiState.installerMediaPollTimer = null;
  }
}

function installerMediaStateLabel(state) {
  const labels = {
    not_started: "Indirilmeyi bekliyor",
    downloading: "ISO indiriliyor",
    cancelling: "Indirme durduruluyor",
    cancelled: "Indirme durduruldu",
    verifying: "SHA-256 dogrulaniyor",
    ready: "ISO hazir",
    failed: "Indirme hatasi"
  };
  return labels[state] || String(state || "-").toUpperCase();
}

function downloadProgressTelemetry(status, previousSample = null) {
  const state = String(status?.state || "not_started");
  const downloadedBytes = Number(status?.downloaded_bytes || 0);
  const totalBytes = Number(status?.total_bytes || 0);
  const nowMs = Date.now();
  const previousWasActive = previousSample && ["downloading", "verifying", "cancelling"].includes(previousSample.state);
  const sameFlow = previousWasActive && downloadedBytes >= previousSample.downloadedBytes;
  const startedAtMs = sameFlow ? previousSample.startedAtMs : nowMs;
  const deltaSeconds = previousSample ? Math.max(0, (nowMs - previousSample.timestampMs) / 1000) : 0;
  const deltaBytes = previousSample && downloadedBytes >= previousSample.downloadedBytes ? downloadedBytes - previousSample.downloadedBytes : 0;
  const sampledRate = deltaSeconds > 0 && deltaBytes > 0 ? Math.round(deltaBytes / deltaSeconds) : 0;
  const providedRate = Number(status?.bytes_per_second || 0);
  const bytesPerSecond = providedRate > 0 ? providedRate : sampledRate;
  const providedElapsed = Number(status?.elapsed_seconds);
  const elapsedSeconds = Number.isFinite(providedElapsed) && providedElapsed >= 0
    ? Math.floor(providedElapsed)
    : Math.max(0, Math.floor((nowMs - startedAtMs) / 1000));
  const providedEta = Number(status?.eta_seconds);
  const etaSeconds = Number.isFinite(providedEta) && providedEta >= 0
    ? Math.floor(providedEta)
    : (totalBytes > downloadedBytes && bytesPerSecond > 0 ? Math.floor((totalBytes - downloadedBytes) / bytesPerSecond) : null);
  const percent = totalBytes > 0 ? Math.min(100, Math.max(0, Math.round((downloadedBytes / totalBytes) * 100))) : null;
  return {
    sample: { downloadedBytes, timestampMs: nowMs, startedAtMs, bytesPerSecond, state },
    metrics: {
      downloadedBytes,
      totalBytes,
      percent,
      bytesPerSecond,
      etaSeconds,
      elapsedSeconds,
      downloadedLabel: formatByteCount(downloadedBytes),
      totalLabel: totalBytes > 0 ? formatByteCount(totalBytes) : null,
      speedLabel: bytesPerSecond > 0 ? `${formatByteCount(bytesPerSecond)}/s` : "-",
      etaLabel: etaSeconds === null ? "-" : formatDuration(etaSeconds),
      elapsedLabel: formatDuration(elapsedSeconds),
    },
  };
}

function installerMediaProgressModel(status, previousSample, title) {
  const { sample, metrics } = downloadProgressTelemetry(status, previousSample);
  const state = String(status?.state || "not_started");
  const downloading = state === "downloading";
  const ready = state === "ready";
  const model = {
    state,
    title: ready ? `${title} hazir` : (state === "failed" ? `${title} indirilemedi` : `${title} indiriliyor`),
    stageLabel: installerMediaStateLabel(state),
    percent: ready ? 100 : (downloading ? metrics.percent : null),
    indeterminate: !ready && (!downloading || metrics.percent === null),
    bytesLabel: metrics.totalLabel ? `${metrics.downloadedLabel} / ${metrics.totalLabel}` : (metrics.downloadedBytes > 0 ? `${metrics.downloadedLabel} indirildi` : "Boyut bekleniyor"),
    speedLabel: downloading ? metrics.speedLabel : "-",
    etaLabel: downloading ? metrics.etaLabel : "-",
    elapsedLabel: metrics.elapsedLabel,
    detail: status?.detail || "",
  };
  return { sample, model };
}

function renderInstallerMediaDownload(status) {
  if (!status) return;
  const state = status.state || "not_started";
  const title = uiState.installerMediaSource?.label || "Kurulum ISO'su";
  if (state === "not_started") {
    uiState.installerMediaProgressSample = null;
    downloadProgressView?.hide(elements.installerMediaDownloadProgress);
  } else {
    const progress = installerMediaProgressModel(status, uiState.installerMediaProgressSample, title);
    uiState.installerMediaProgressSample = progress.sample;
    downloadProgressView?.render(elements.installerMediaDownloadProgress, progress.model);
  }
  elements.installerMediaSourceNote.textContent = status.detail || uiState.installerMediaSource?.note || "";

  const ready = state === "ready";
  const active = state === "downloading" || state === "verifying" || state === "cancelling";
  const cancellable = state === "downloading" || state === "verifying";
  elements.installerMediaDownloadButton.disabled = active;
  elements.installerMediaDownloadButton.textContent = ["failed", "cancelled"].includes(state)
    ? "Tekrar Indir"
    : (uiState.installerMediaSource?.recommended ? "Onerilen ISO'yu Indir" : "Secilen ISO'yu Indir");
  elements.installerMediaCancelDownloadButton.classList.toggle("hidden", !active);
  elements.installerMediaCancelDownloadButton.disabled = !cancellable;
  elements.installerMediaCancelDownloadButton.textContent = state === "cancelling" ? "Durduruluyor..." : "Indirmeyi Durdur";
  elements.installerMediaSourceSelect.disabled = active;
  elements.installerMediaOptionsToggle.disabled = active;
  elements.installerMediaAttachDownloadedButton.classList.toggle("hidden", !ready);
  elements.installerMediaAttachDownloadedButton.disabled = !ready;
  if (ready || state === "cancelled" || state === "failed") stopInstallerMediaPolling();
}

function normalizeInstallerArchitecture(value) {
  const normalized = String(value || "").trim().toLowerCase();
  if (["x86_64", "amd64", "x64"].includes(normalized)) return "x86_64";
  if (["aarch64", "arm64"].includes(normalized)) return "arm64";
  return normalized;
}

function installerMediaSourceManagedDownload(source) {
  return Boolean(source?.managed_download);
}

function installerMediaSourceCompatible(source, template) {
  if (!source || !template) return false;
  const sourceArch = normalizeInstallerArchitecture(source.architecture);
  const vmArch = normalizeInstallerArchitecture(template.architecture);
  const hostArch = normalizeInstallerArchitecture(uiState.hostArchitecture);
  const vmMatch = !sourceArch || !vmArch || sourceArch === vmArch;
  const hostMatch = !sourceArch || !hostArch || sourceArch === hostArch;
  return vmMatch && hostMatch;
}

function installerMediaSourcesForTemplate(template) {
  return [template?.installer_media, ...(Array.isArray(template?.installer_media_options) ? template.installer_media_options : [])]
    .filter(Boolean);
}

function installerMediaOptionLabel(source, template) {
  const flags = [];
  if (source.recommended) flags.push("Onerilen");
  if (!installerMediaSourceCompatible(source, template)) flags.push("Uyumsuz");
  return flags.length > 0 ? `${source.label} - ${flags.join(" / ")}` : source.label;
}

async function refreshInstallerMediaDownloadStatus(showFailure = false) {
  const guestTemplateId = uiState.installerMediaTemplateId;
  const mediaId = uiState.installerMediaSelectedMediaId;
  if (!guestTemplateId || !mediaId || !installerMediaSourceManagedDownload(uiState.installerMediaSource)) return null;
  try {
    const status = await invoke("get_installer_media_download", { request: { guest_template_id: guestTemplateId, media_id: mediaId } });
    if (status.media_id !== uiState.installerMediaSelectedMediaId) return status;
    renderInstallerMediaDownload(status);
    return status;
  } catch (error) {
    if (showFailure) showError(String(error));
    return null;
  }
}

function startInstallerMediaPolling() {
  stopInstallerMediaPolling();
  uiState.installerMediaPollTimer = window.setInterval(() => {
    refreshInstallerMediaDownloadStatus(false);
  }, 1200);
}

async function applyInstallerMediaSource(source, template) {
  stopInstallerMediaPolling();
  uiState.installerMediaSource = source || null;
  uiState.installerMediaSelectedMediaId = source?.id || null;

  elements.installerMediaDownloadButton.classList.add("hidden");
  elements.installerMediaCancelDownloadButton.classList.add("hidden");
  elements.installerMediaSourceSelect.disabled = false;
  elements.installerMediaOptionsToggle.disabled = false;
  elements.installerMediaAttachDownloadedButton.classList.add("hidden");
  elements.installerMediaOfficialPageButton.classList.add("hidden");
  downloadProgressView?.hide(elements.installerMediaDownloadProgress);
  uiState.installerMediaProgressSample = null;
  elements.installerMediaProvider.textContent = source?.provider || "Manuel";
  elements.installerMediaSourceLabel.textContent = source?.label || "Katalogda otomatik ISO kaynagi yok";
  elements.installerMediaSourceNote.textContent = source?.note || "Yerel ISO secerek devam edebilirsiniz.";
  elements.installerMediaOptionNote.textContent = source
    ? `${source.architecture || "-"} / ${installerMediaSourceManagedDownload(source) ? "TurkuazVM icinden indirilebilir" : "Resmi sayfa"}. VM template kaydi degismez.`
    : "Secilen medya VM template kaydini degistirmez; yalniz kurulum ISO'sunu degistirir.";

  if (!source) return;
  if (!installerMediaSourceCompatible(source, template)) {
    elements.installerMediaSourceNote.textContent = `${source.note || ""} Bu medya host/VM mimarisi ile uyumlu degil.`.trim();
    return;
  }
  if (installerMediaSourceManagedDownload(source)) {
    elements.installerMediaDownloadButton.classList.remove("hidden");
    elements.installerMediaDownloadButton.textContent = source.recommended ? "Onerilen ISO'yu Indir" : "Secilen ISO'yu Indir";
    if (source.mode === INSTALLER_MEDIA_MODE_OFFICIAL_PAGE) {
      elements.installerMediaOfficialPageButton.classList.remove("hidden");
    }
    const status = await refreshInstallerMediaDownloadStatus(false);
    if (status && ["downloading", "verifying", "cancelling"].includes(status.state)) startInstallerMediaPolling();
    return;
  }
  if (source.mode === INSTALLER_MEDIA_MODE_OFFICIAL_PAGE) {
    elements.installerMediaOfficialPageButton.classList.remove("hidden");
  }
}

async function prepareInstallerMediaSource(machine) {
  try {
    if (uiState.guestCatalog.length === 0) await loadGuestCatalog();
    const template = uiState.guestCatalog.find((item) => item.id === machine.guest_template_id);
    const sources = installerMediaSourcesForTemplate(template);
    const compatibleSources = sources.filter((source) => installerMediaSourceCompatible(source, template));
    const recommended = compatibleSources.find((source) => source.recommended)
      || compatibleSources[0]
      || sources.find((source) => source.recommended)
      || sources[0]
      || null;

    uiState.installerMediaTemplateId = template?.id || null;
    uiState.installerMediaSources = sources;
    uiState.installerMediaSelectedMediaId = recommended?.id || null;

    elements.installerMediaHostArchitecture.textContent = [uiState.hostPlatform, uiState.hostArchitecture, uiState.hostAcceleration]
      .filter(Boolean)
      .join(" / ") || "Bilinmiyor";
    elements.installerMediaVmArchitecture.textContent = template?.architecture || "-";
    elements.installerMediaFirmware.textContent = template?.firmware?.toUpperCase() || "-";
    elements.installerMediaRecommendedLabel.textContent = recommended?.label || "Manuel ISO";

    elements.installerMediaSourceSelect.innerHTML = sources.map((source) => {
      const compatible = installerMediaSourceCompatible(source, template);
      return `<option value="${escapeAttribute(source.id)}" ${compatible ? "" : "disabled"}>${escapeHtml(installerMediaOptionLabel(source, template))}</option>`;
    }).join("");
    if (recommended && sources.some((source) => source.id === recommended.id)) {
      elements.installerMediaSourceSelect.value = recommended.id;
    }
    elements.installerMediaOptionsToggle.classList.toggle("hidden", sources.length <= 1);
    elements.installerMediaOptionsPanel.classList.add("hidden");
    elements.installerMediaOptionsToggle.textContent = "Farkli Surum / Medya Sec";

    await applyInstallerMediaSource(recommended, template);
  } catch (error) {
    uiState.installerMediaSources = [];
    uiState.installerMediaSelectedMediaId = null;
    elements.installerMediaSourceLabel.textContent = "ISO katalogu yuklenemedi";
    elements.installerMediaSourceNote.textContent = String(error);
  }
}

function toggleInstallerMediaOptions() {
  const hidden = elements.installerMediaOptionsPanel.classList.toggle("hidden");
  elements.installerMediaOptionsToggle.textContent = hidden ? "Farkli Surum / Medya Sec" : "Secenekleri Gizle";
}

async function handleInstallerMediaSourceChange() {
  const mediaId = elements.installerMediaSourceSelect.value;
  const source = uiState.installerMediaSources.find((item) => item.id === mediaId) || null;
  if (!source) return;
  const template = uiState.guestCatalog.find((item) => item.id === uiState.installerMediaTemplateId) || null;
  await applyInstallerMediaSource(source, template);
}

function openInstallerMediaModal(vmId) {
  const machine = uiState.machines.find((item) => item.id === vmId);
  if (!machine) {
    showError(`VM bulunamadi: ${vmId}`);
    return;
  }
  if (machine.guest_profile === "android") {
    showToast("Android sistem goruntusu", "Android VM'lerde ISO yerine Android Sistem Goruntuleri kullanilir.", "info");
    openAndroidImagesForVm(vmId, { vmId, fromAndroid: true });
    return;
  }
  stopInstallerMediaPolling();
  uiState.installerMediaFlowVmId = vmId;
  uiState.installerMediaSelectedPath = null;
  uiState.installerMediaTemplateId = machine.guest_template_id || null;
  uiState.installerMediaSource = null;
  uiState.installerMediaSources = [];
  uiState.installerMediaSelectedMediaId = null;
  elements.installerMediaVm.value = `${machine.name} (${machine.id})`;
  elements.installerMediaPath.value = "";
  const hasInstallerMedia = Boolean(machine.installer_media);
  const canEjectInstallerMedia = String(machine.state || "").toLowerCase() === STATE_STOPPED;
  elements.installerMediaCurrent.textContent = hasInstallerMedia
    ? `Bagli medya: ${machine.installer_media}. ISO ilk boot boyunca takili kalir; guest yeniden baslatildiginda veya VM durdugunda otomatik cikarilir ve sonraki boot diskten yapilir.`
    : "Kurulum medyasi bagli degil.";
  elements.installerMediaEjectButton.classList.toggle("hidden", !hasInstallerMedia);
  elements.installerMediaEjectButton.disabled = !canEjectInstallerMedia;
  elements.installerMediaEjectButton.title = canEjectInstallerMedia ? "" : "ISO cikarmak icin once VM'i durdurun.";
  elements.installerMediaAttachButton.textContent = hasInstallerMedia ? "Yerel ISO ile Degistir" : "Yerel ISO'yu Bagla";
  elements.installerMediaAttachDownloadedButton.textContent = hasInstallerMedia ? "Indirilen ISO ile Degistir" : "Indirilen ISO'yu Bagla";
  elements.installerMediaAttachButton.disabled = true;
  setInstallerMediaReady(hasInstallerMedia);
  elements.installerMediaModal.classList.remove("hidden");
  prepareInstallerMediaSource(machine);
}

function closeInstallerMediaModal() {
  stopInstallerMediaPolling();
  elements.installerMediaModal.classList.add("hidden");
  uiState.installerMediaFlowVmId = null;
  uiState.installerMediaSelectedPath = null;
  uiState.installerMediaTemplateId = null;
  uiState.installerMediaSource = null;
  uiState.installerMediaSources = [];
  uiState.installerMediaSelectedMediaId = null;
  uiState.installerMediaReady = false;
}

async function startInstallerMediaDownload() {
  const guestTemplateId = uiState.installerMediaTemplateId;
  const mediaId = uiState.installerMediaSelectedMediaId;
  if (!guestTemplateId || !mediaId || !installerMediaSourceManagedDownload(uiState.installerMediaSource)) return;
  setButtonBusy(elements.installerMediaDownloadButton, true);
  try {
    const status = await invoke("start_installer_media_download", { request: { guest_template_id: guestTemplateId, media_id: mediaId } });
    recordActivity("ISO Indir", guestTemplateId, "success", status.detail || "Indirme baslatildi");
    renderInstallerMediaDownload(status);
    if (["downloading", "verifying", "cancelling"].includes(status.state)) startInstallerMediaPolling();
    if (status.state === "ready") showToast("ISO hazir", uiState.installerMediaSource?.label || guestTemplateId, "success");
    clearError();
  } catch (error) {
    recordActivity("ISO Indir", guestTemplateId, "error", String(error));
    showError(String(error));
  } finally {
    setButtonBusy(elements.installerMediaDownloadButton, false);
  }
}

async function cancelInstallerMediaDownload() {
  const guestTemplateId = uiState.installerMediaTemplateId;
  const mediaId = uiState.installerMediaSelectedMediaId;
  if (!guestTemplateId || !mediaId || !installerMediaSourceManagedDownload(uiState.installerMediaSource)) return;
  setButtonBusy(elements.installerMediaCancelDownloadButton, true);
  try {
    const status = await invoke("cancel_installer_media_download", { request: { guest_template_id: guestTemplateId, media_id: mediaId } });
    recordActivity("ISO Durdur", guestTemplateId, "success", status.detail || "Indirme durduruluyor");
    renderInstallerMediaDownload(status);
    if (["downloading", "verifying", "cancelling"].includes(status.state)) startInstallerMediaPolling();
    else stopInstallerMediaPolling();
    showToast("ISO indirme", status.detail || "Indirme durduruldu", "info");
    clearError();
  } catch (error) {
    recordActivity("ISO Durdur", guestTemplateId, "error", String(error));
    showError(String(error));
  } finally {
    setButtonBusy(elements.installerMediaCancelDownloadButton, false);
  }
}

async function attachDownloadedInstallerMedia() {
  const vmId = uiState.installerMediaFlowVmId;
  const guestTemplateId = uiState.installerMediaTemplateId;
  const mediaId = uiState.installerMediaSelectedMediaId;
  if (!vmId || !guestTemplateId || !mediaId) return;
  setButtonBusy(elements.installerMediaAttachDownloadedButton, true);
  try {
    const machine = await invoke("attach_downloaded_installer_media", { request: { vm_id: vmId, guest_template_id: guestTemplateId, media_id: mediaId } });
    recordActivity("ISO Bagla", vmId, "success", uiState.installerMediaSource?.label || guestTemplateId);
    showToast("ISO baglandi", machine.installer_media || vmId, "success");
    await refreshDashboard();
    const updated = uiState.machines.find((item) => item.id === vmId);
    elements.installerMediaCurrent.textContent = updated?.installer_media ? `Bagli medya: ${updated.installer_media}` : "ISO baglandi.";
    elements.installerMediaEjectButton.classList.remove("hidden");
    elements.installerMediaEjectButton.disabled = false;
    setInstallerMediaReady(true);
    clearError();
  } catch (error) {
    recordActivity("ISO Bagla", vmId, "error", String(error));
    showError(String(error));
  } finally {
    setButtonBusy(elements.installerMediaAttachDownloadedButton, false);
  }
}

async function openInstallerMediaOfficialPage() {
  const url = uiState.installerMediaSource?.url;
  if (!url) return;
  try {
    await invoke("open_external_url", { url });
    showToast("Resmi sayfa acildi", uiState.installerMediaSource?.provider || "Resmi kaynak", "info");
  } catch (error) {
    showError(String(error));
  }
}

async function pickInstallerMedia() {
  try {
    const selected = await invoke("pick_installer_iso");
    if (!selected) return;
    uiState.installerMediaSelectedPath = selected;
    elements.installerMediaPath.value = selected;
    elements.installerMediaAttachButton.disabled = false;
  } catch (error) {
    showError(String(error));
  }
}

async function attachInstallerMedia() {
  const vmId = uiState.installerMediaFlowVmId;
  const sourcePath = uiState.installerMediaSelectedPath;
  if (!vmId || !sourcePath) return;
  setButtonBusy(elements.installerMediaAttachButton, true);
  try {
    const machine = await invoke("configure_installer_media", { request: { vm_id: vmId, source_path: sourcePath } });
    recordActivity("Kurulum Medyasi", vmId, "success", sourcePath);
    showToast("ISO baglandi", machine.installer_media || sourcePath, "success");
    await refreshDashboard();
    const updated = uiState.machines.find((item) => item.id === vmId);
    elements.installerMediaCurrent.textContent = updated?.installer_media ? `Bagli medya: ${updated.installer_media}` : "ISO baglandi.";
    elements.installerMediaEjectButton.classList.remove("hidden");
    elements.installerMediaEjectButton.disabled = false;
    setInstallerMediaReady(true);
  } catch (error) {
    recordActivity("Kurulum Medyasi", vmId, "error", String(error));
    showError(String(error));
  } finally {
    setButtonBusy(elements.installerMediaAttachButton, false);
  }
}

async function ejectInstallerMedia() {
  const vmId = uiState.installerMediaFlowVmId;
  if (!vmId) return;
  const machine = uiState.machines.find((item) => item.id === vmId);
  if (!machine || String(machine.state || "").toLowerCase() !== STATE_STOPPED) {
    showError("ISO cikarmak icin once VM'i durdurun.");
    return;
  }
  if (!window.confirm("Bagli ISO cikarilsin ve VM kalici olarak diskten baslatilsin mi? ISO dosyasi ve sanal disk silinmez.")) return;

  setButtonBusy(elements.installerMediaEjectButton, true);
  try {
    await invoke("eject_installer_media", { vmId });
    recordActivity("ISO Cikar", vmId, "success", "Kalici disk boot etkinlestirildi");
    await refreshDashboard();
    closeInstallerMediaModal();
    showToast("ISO cikarildi", "VM artik kalici diskten baslatilacak. Baslat dugmesini kullanin.", "success");
    clearError();
  } catch (error) {
    recordActivity("ISO Cikar", vmId, "error", String(error));
    showError(String(error));
  } finally {
    setButtonBusy(elements.installerMediaEjectButton, false);
  }
}

async function handleInstallerMediaNext() {
  if (!uiState.installerMediaReady || !uiState.installerMediaFlowVmId) return;
  const vmId = uiState.installerMediaFlowVmId;
  closeInstallerMediaModal();
  showToast("Siradaki adim", "Ag yapilandirmasini tamamlayin.", "info");
  await openNetworkModal(vmId, { vmId, fromAndroid: false });
}

async function openConfigurationCompleteModal(vmId, options = {}) {
  uiState.configurationCompleteVmId = vmId;
  await refreshDashboard();
  const machine = uiState.machines.find((item) => item.id === vmId);
  const fromAndroid = Boolean(options.fromAndroid || machine?.guest_profile === "android");
  let assignment = fromAndroid ? uiState.androidAssignments[vmId] : null;
  if (fromAndroid && assignment === undefined) {
    try {
      assignment = await invoke("get_android_image_assignment", { vmId });
      uiState.androidAssignments[vmId] = assignment || null;
    } catch (_) {
      assignment = null;
    }
  }

  elements.configurationCompleteMediaStep.classList.toggle("hidden", fromAndroid || !machine?.installer_media);
  elements.configurationCompleteImageStep.classList.toggle("hidden", !fromAndroid);
  elements.configurationCompleteVm.textContent = machine ? `${machine.name} / ${machine.id}` : vmId;
  elements.configurationCompleteDisk.textContent = machine ? `${machine.disk_count} disk` : "-";
  elements.configurationCompleteNetwork.textContent = machine ? `${machine.network_count} NIC` : "-";
  elements.configurationCompleteMedia.textContent = fromAndroid ? "Uygulanmaz" : (machine?.installer_media || "Bagli degil");
  elements.configurationCompleteImage.textContent = fromAndroid
    ? (assignment ? `${assignment.image_id} / ${String(assignment.provisioning_state || "pending_first_boot").toUpperCase()}` : "Atanmadi")
    : "Uygulanmaz";
  elements.configurationCompleteSubtitle.textContent = fromAndroid
    ? "VM, disk, Android image ve ag adimlari tamamlandi. Ilk Android boot provisioning durumunu ilerletecektir."
    : "VM, disk, kurulum medyasi ve ag adimlari tamamlandi.";
  elements.configurationCompleteModal.classList.remove("hidden");
  activateNavigation("machines");
}

function closeConfigurationCompleteModal() {
  elements.configurationCompleteModal.classList.add("hidden");
  uiState.configurationCompleteVmId = null;
  activateNavigation("machines");
}


async function loadHosts() {
  try {
    const hosts = await invoke("list_hosts");
    uiState.hosts = hosts;
    elements.hostSelector.innerHTML = "";
    for (const host of hosts) {
      const option = document.createElement("option");
      option.value = host.id;
      option.textContent = `${host.label} (${host.mode.toUpperCase()})`;
      option.selected = host.active;
      elements.hostSelector.appendChild(option);
    }
  } catch (error) {
    showError(String(error));
  }
}

async function handleHostChange() {
  const hostId = elements.hostSelector.value;
  elements.hostSelector.disabled = true;
  try {
    const dashboard = await invoke("select_host", { hostId });
    renderDashboard(dashboard);
    clearError();
    await loadHosts();
  } catch (error) {
    renderDisconnected(String(error));
  } finally {
    elements.hostSelector.disabled = false;
  }
}

async function refreshDashboard() {
  try {
    const dashboard = await invoke("get_dashboard");
    renderDashboard(dashboard);
    clearError();
    return true;
  } catch (error) {
    renderDisconnected(String(error));
    return false;
  }
}

function renderDashboard(dashboard) {
  uiState.dashboard = dashboard;
  setStatus(elements.engineStatus, dashboard.engine_ready ? "HAZIR" : "KULLANILAMIYOR", dashboard.engine_ready);
  elements.engineMiniStatus.textContent = dashboard.engine_ready ? "Motor HAZIR" : "Motor KULLANILAMIYOR";
  if (elements.resourceEngineStatus) elements.resourceEngineStatus.textContent = dashboard.engine_ready ? "Hazir" : "Kullanilamiyor";
  const qemuText = dashboard.qemu_ready
    ? (dashboard.qemu_img_ready ? "HAZIR + IMG" : "HAZIR / IMG YOK")
    : "NOT FOUND";
  setStatus(elements.qemuStatus, qemuText, dashboard.qemu_ready && dashboard.qemu_img_ready);
  elements.hostEyebrow.textContent = dashboard.host_label.toUpperCase();
  elements.hostStatus.textContent = `${dashboard.platform} / ${dashboard.architecture}`;
  const authText = dashboard.authentication_required ? "AUTH" : "NO AUTH";
  elements.transportStatus.textContent = `${dashboard.transport_security.toUpperCase()} / ${authText}`;
  elements.accelerationStatus.textContent = dashboard.acceleration.toUpperCase();
  const gpuText = `${dashboard.gpu_backend.toUpperCase()}${dashboard.gpu_vulkan_ready ? " / VK" : ""}`;
  setStatus(elements.gpuStatus, gpuText, dashboard.gpu_accelerated || dashboard.gpu_backend === "virtio_2d");
  const adbText = dashboard.android_adb_ready
    ? (dashboard.android_adb_version || "HAZIR")
    : "NOT FOUND";
  setStatus(elements.adbStatus, adbText, dashboard.android_adb_ready);
  uiState.machines = Array.isArray(dashboard.machines) ? dashboard.machines : [];
  uiState.qemuReady = Boolean(dashboard.qemu_ready);
  uiState.qemuImgReady = Boolean(dashboard.qemu_img_ready);
  uiState.hostArchitecture = dashboard.architecture || null;
  uiState.hostPlatform = dashboard.platform || null;
  uiState.hostAcceleration = dashboard.acceleration || null;
  if (uiState.selectedVmId && !uiState.machines.some((machine) => machine.id === uiState.selectedVmId)) {
    uiState.selectedVmId = null;
  }
  renderAndroidImageQuickVmOptions();
  renderMachineTree(uiState.machines);
  renderMachines(uiState.machines, uiState.qemuReady, uiState.qemuImgReady);
  renderHomeOverview();
  void refreshAndroidAssignmentReadiness();
}

function renderDisconnected(message) {
  setStatus(elements.engineStatus, "DISCONNECTED", false);
  elements.engineMiniStatus.textContent = "Motor BAGLANTI YOK";
  if (elements.resourceEngineStatus) elements.resourceEngineStatus.textContent = "Baglanti yok";
  setStatus(elements.qemuStatus, "UNKNOWN", false);
  elements.hostStatus.textContent = "-";
  elements.transportStatus.textContent = "-";
  elements.accelerationStatus.textContent = "-";
  setStatus(elements.gpuStatus, "UNKNOWN", false);
  setStatus(elements.adbStatus, "UNKNOWN", false);
  uiState.dashboard = null;
  uiState.machines = [];
  uiState.qemuReady = false;
  uiState.qemuImgReady = false;
  uiState.hostArchitecture = null;
  uiState.hostPlatform = null;
  uiState.hostAcceleration = null;
  uiState.selectedVmId = null;
  uiState.androidAssignments = {};
  renderAndroidImageQuickVmOptions();
  renderMachineTree([]);
  renderMachines([], false, false);
  renderHomeOverview();
  showError(message);
}

function setStatus(element, text, ok) {
  element.textContent = text;
  element.classList.toggle("status-ok", ok);
  element.classList.toggle("status-bad", !ok);
}

function activateNavigation(key) {
  const navigationMap = new Map([
    ["home", elements.homeButton],
    ["machines", elements.machinesButton],
    ["storage", elements.storageButton],
    ["artifact-cache", elements.artifactCacheButton],
    ["network", elements.networkButton],
    ["gaming", elements.gamingButton],
    ["android-images", elements.androidImagesButton],
    ["logs", elements.logsButton]
  ]);
  const workspacePageMap = new Map([
    ["storage", elements.storageModal],
    ["artifact-cache", elements.artifactCacheModal],
    ["network", elements.networkModal],
    ["gaming", elements.gpuModal],
    ["android-images", elements.androidImagesModal],
    ["logs", elements.logsModal]
  ]);
  const sectionLabels = {
    home: ["YEREL CALISMA ALANI", "Ana Sayfa"],
    machines: ["SANAL MAKINE YONETIMI", "Sanal Makineler"],
    storage: ["ALTYAPI", "Depolama"],
    network: ["ALTYAPI", "Ag"],
    "android-images": ["GORUNTU MERKEZI", "Goruntuler"],
    logs: ["OPERASYON", "Gorevler"],
    "artifact-cache": ["ALTYAPI", "Onbellek"],
    gaming: ["UYGULAMA", "Ayarlar"]
  };
  uiState.activeNavigation = key;
  for (const [navKey, button] of navigationMap.entries()) {
    if (button) button.classList.toggle("active", navKey === key);
  }
  if (elements.homePage) elements.homePage.classList.toggle("hidden", key !== "home");
  if (elements.machinesPage) elements.machinesPage.classList.toggle("hidden", key !== "machines");
  for (const [pageKey, modal] of workspacePageMap.entries()) {
    if (modal && pageKey !== key) modal.classList.add("hidden");
  }
  const [kicker, title] = sectionLabels[key] || ["TURKUAZVM", "Workspace"];
  if (elements.workspaceSectionKicker) elements.workspaceSectionKicker.textContent = kicker;
  if (elements.workspaceSectionTitle) elements.workspaceSectionTitle.textContent = title;
}

function renderHomeOverview() {
  if (!elements.homeOverview || !window.TurkuazVmWorkspaceView) return;
  elements.homeOverview.innerHTML = window.TurkuazVmWorkspaceView.renderHome({
    dashboard: uiState.dashboard,
    machines: uiState.machines,
    escapeHtml,
    escapeAttribute,
    getStateLabel,
    getGuestProfileLabel,
    formatMemory
  });
}

function showHomeNavigation() {
  activateNavigation("home");
  renderHomeOverview();
}

function getFilteredMachines(machines) {
  const query = uiState.vmSearchQuery.trim().toLocaleLowerCase("tr-TR");
  const filtered = machines.filter((machine) => {
    const state = String(machine.state || "").toLowerCase();
    const profile = String(machine.guest_profile || "").toLowerCase();
    if (uiState.vmQuickFilter !== VM_FILTER_ALL) {
      const filterMatch = uiState.vmQuickFilter === state || uiState.vmQuickFilter === profile;
      if (!filterMatch) return false;
    }
    if (!query) return true;
    const haystack = [machine.name, machine.id, machine.guest_profile, machine.guest_template_id, machine.state, machine.acceleration]
      .map((value) => String(value || "").toLocaleLowerCase("tr-TR"))
      .join(" ");
    return haystack.includes(query);
  });
  return sortMachines(filtered);
}

function renderMachineTree(machines) {
  elements.treeMachineList.innerHTML = "";
  if (machines.length === 0) {
    const empty = document.createElement("div");
    empty.className = "tree-empty";
    empty.textContent = "VM yok";
    elements.treeMachineList.appendChild(empty);
    return;
  }
  for (const machine of machines) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "tree-item tree-leaf tree-machine-button";
    button.dataset.treeVmId = machine.id;
    button.classList.toggle("is-selected", uiState.selectedVmId === machine.id);
    button.classList.toggle("is-running", String(machine.state || "").toLowerCase() === STATE_RUNNING);
    button.innerHTML = `<span class="tree-machine-name">${escapeHtml(machine.name)}</span><span class="tree-machine-state">${escapeHtml(getStateLabel(machine.state))}</span>`;
    elements.treeMachineList.appendChild(button);
  }
}

function renderMachineViews() {
  renderMachineTree(uiState.machines);
  renderMachines(uiState.machines, uiState.qemuReady, uiState.qemuImgReady);
  renderVmDetail();
}

function clearVmFilters() {
  uiState.vmSearchQuery = "";
  uiState.vmQuickFilter = VM_FILTER_ALL;
  elements.vmSearchInput.value = "";
  for (const button of elements.vmFilterGroup.querySelectorAll("[data-vm-filter]")) {
    button.classList.toggle("active", button.dataset.vmFilter === VM_FILTER_ALL);
  }
  renderMachineViews();
}

function showMachinesNavigation() {
  activateNavigation("machines");
  clearVmFilters();
  document.querySelector(".machine-section")?.scrollIntoView({ behavior: "smooth", block: "start" });
}

function handleVmSearchInput(event) {
  uiState.vmSearchQuery = event.target.value || "";
  renderMachineViews();
}

function handleTreeMachineClick(event) {
  const button = event.target.closest("[data-tree-vm-id]");
  if (!button) return;
  const vmId = button.dataset.treeVmId;
  uiState.selectedVmId = vmId;
  uiState.selectedVmTab = "overview";
  activateNavigation("machines");
  renderMachineViews();
}

function getStateLabel(state) {
  const normalized = String(state || "").toLowerCase();
  if (normalized === STATE_RUNNING) return "Calisiyor";
  if (normalized === STATE_STOPPED) return "Durdu";
  if (normalized === "starting") return "Baslatiliyor";
  if (normalized === "stopping") return "Durduruluyor";
  if (normalized === "error") return "Hata";
  return normalized ? normalized.toUpperCase() : "-";
}

function getGuestProfileLabel(profile) {
  const normalized = String(profile || "").toLowerCase();
  if (normalized === "android") return "Android";
  if (normalized === "linux") return "Linux";
  if (normalized === "windows") return "Windows";
  return normalized ? normalized.toUpperCase() : "Standart";
}

function getGuestAvatar(profile) {
  const normalized = String(profile || "").toLowerCase();
  if (normalized === "android") return "A";
  if (normalized === "linux") return "L";
  if (normalized === "windows") return "W";
  return "V";
}

async function refreshAndroidAssignmentReadiness() {
  const androidMachines = uiState.machines.filter((machine) => machine.guest_profile === "android");
  const validIds = new Set(androidMachines.map((machine) => machine.id));
  const nextAssignments = {};
  for (const [vmId, assignment] of Object.entries(uiState.androidAssignments)) {
    if (validIds.has(vmId)) nextAssignments[vmId] = assignment;
  }
  uiState.androidAssignments = nextAssignments;
  if (androidMachines.length === 0) return;

  await Promise.all(androidMachines.map(async (machine) => {
    try {
      const assignment = await invoke("get_android_image_assignment", { vmId: machine.id });
      uiState.androidAssignments[machine.id] = assignment || null;
    } catch (_) {
      uiState.androidAssignments[machine.id] = null;
    }
  }));
  renderMachines(uiState.machines, uiState.qemuReady, uiState.qemuImgReady);
}

function isRecoverableStartPreflightError(machine) {
  const state = String(machine.state || "").toLowerCase();
  if (state !== "error") return false;
  const error = String(machine.last_error || "");
  return error.includes("DiskBootRequiresDisk")
    || error.includes("android_image_required")
    || error.includes("Android VM requires an assigned Ready Android image");
}

function hasLoadedAndroidAssignment(vmId) {
  return Object.prototype.hasOwnProperty.call(uiState.androidAssignments, vmId);
}

function hasAndroidImageAssignment(machine) {
  if (machine.guest_profile !== "android") return true;
  return hasLoadedAndroidAssignment(machine.id) && Boolean(uiState.androidAssignments[machine.id]);
}


function renderMachines(machines, qemuReady, qemuImgReady) {
  renderFleetSummary(machines);
  const visibleMachines = getFilteredMachines(machines);
  const hasFleet = machines.length > 0;
  elements.vmWorkspaceShell?.classList.toggle("is-empty-fleet", !hasFleet);
  elements.fleetEmptyWorkspace?.classList.toggle("hidden", hasFleet);
  elements.machinesPage?.classList.toggle("empty-fleet-page", !hasFleet);
  const filtering = Boolean(uiState.vmSearchQuery.trim() || uiState.vmQuickFilter !== VM_FILTER_ALL);
  elements.bulkStartButton.disabled = !qemuReady || !visibleMachines.some((machine) => {
    const state = String(machine.state || "").toLowerCase();
    return (state === STATE_STOPPED || isRecoverableStartPreflightError(machine)) && Number(machine.disk_count || 0) > 0 && hasAndroidImageAssignment(machine);
  });
  elements.bulkStopButton.disabled = !visibleMachines.some((machine) => String(machine.state || "").toLowerCase() === STATE_RUNNING);
  elements.machineCount.textContent = filtering ? `${visibleMachines.length} / ${machines.length} makine` : `${machines.length} makine`;
  elements.selectionSummary.textContent = uiState.selectedVmId ? `Secili: ${uiState.machines.find((machine) => machine.id === uiState.selectedVmId)?.name || uiState.selectedVmId}` : "Bir VM secin";
  elements.clearVmFilterButton.classList.toggle("hidden", !filtering);
  elements.machineGrid.innerHTML = "";
  elements.emptyState.classList.toggle("hidden", visibleMachines.length !== 0);
  const emptyTitle = elements.emptyState.querySelector("h3");
  const emptyText = elements.emptyState.querySelector("p");
  if (visibleMachines.length === 0 && machines.length > 0) {
    emptyTitle.textContent = "Filtreye uygun VM yok";
    emptyText.textContent = "Arama veya filtreyi temizleyerek tum sanal makineleri gorebilirsiniz.";
  } else {
    emptyTitle.textContent = "Henuz sanal makine yok";
    emptyText.textContent = "Ilk VM'inizi olusturarak baslayin.";
  }

  if (uiState.selectedVmId && !machines.some((machine) => machine.id === uiState.selectedVmId)) uiState.selectedVmId = null;

  for (const machine of visibleMachines) {
    const state = String(machine.state || "").toLowerCase();
    const card = document.createElement("article");
    card.className = `vm-card${uiState.selectedVmId === machine.id ? " is-selected" : ""}`;
    card.dataset.action = "select";
    card.dataset.vmId = machine.id;
    const stateLabel = getStateLabel(state);
    const quickCanStart = (state === STATE_STOPPED || isRecoverableStartPreflightError(machine))
      && qemuReady
      && Number(machine.disk_count || 0) > 0
      && hasAndroidImageAssignment(machine);
    const quickAction = state === STATE_RUNNING
      ? `<button class="quick-action running" data-action="display" data-vm-id="${escapeAttribute(machine.id)}">Konsol</button>`
      : `<button class="quick-action" data-action="start" data-vm-id="${escapeAttribute(machine.id)}" ${quickCanStart ? "" : "disabled"}>Baslat</button>`;
    card.innerHTML = `
      <div class="vm-list-avatar">${escapeHtml(getGuestAvatar(machine.guest_profile))}</div>
      <div class="vm-list-main">
        <div class="vm-list-name-row"><strong>${escapeHtml(machine.name)}</strong></div>
        <div class="vm-list-sub"><span>${escapeHtml(getGuestProfileLabel(machine.guest_profile))}</span><span>${machine.vcpu_count} CPU / ${escapeHtml(formatMemory(machine.memory_mib))}</span></div>
      </div>
      <div class="vm-list-status"><span class="vm-list-state-label ${escapeAttribute(state)}"><i class="state-dot ${escapeAttribute(state)}"></i>${escapeHtml(stateLabel)}</span>${quickAction}</div>
    `;
    elements.machineGrid.appendChild(card);
  }
  renderVmDetail();
}

function vmPrimaryAddress(machine) {
  const networks = Array.isArray(machine?.networks) ? machine.networks : [];
  return networks.find((network) => network.ipv4_address)?.ipv4_address || "-";
}

function vmPublishedServiceCount(machine) {
  return (Array.isArray(machine?.networks) ? machine.networks : []).reduce((total, network) => total + (Array.isArray(network.published_services) ? network.published_services.length : 0), 0);
}

function renderVmDetail() {
  if (!elements.vmDetailPanel || !elements.vmDetailEmpty) return;
  const machine = uiState.machines.find((item) => item.id === uiState.selectedVmId);
  elements.vmDetailEmpty.classList.toggle("hidden", Boolean(machine));
  elements.vmDetailPanel.classList.toggle("hidden", !machine);
  if (!machine) {
    elements.vmDetailPanel.innerHTML = "";
    return;
  }
  if (!window.TurkuazVmWorkspaceView) {
    elements.vmDetailPanel.innerHTML = '<div class="workspace-inline-empty">VM Workspace View yuklenemedi.</div>';
    return;
  }
  const state = String(machine.state || "").toLowerCase();
  const running = state === STATE_RUNNING;
  const stopped = state === STATE_STOPPED;
  const hasDisk = Number(machine.disk_count || 0) > 0;
  const hasNetwork = Number(machine.network_count || 0) > 0;
  const publishedServiceCount = vmPublishedServiceCount(machine);
  const hasRemote = publishedServiceCount > 0;
  const androidImageMissing = machine.guest_profile === "android" && !hasAndroidImageAssignment(machine);
  const recoverableError = isRecoverableStartPreflightError(machine);
  const canStart = (stopped || recoverableError) && uiState.qemuReady && hasDisk && hasAndroidImageAssignment(machine);
  const canDelete = stopped || state === "error";
  elements.vmDetailPanel.innerHTML = window.TurkuazVmWorkspaceView.renderVmDetail(machine, {
    escapeHtml,
    escapeAttribute,
    getStateLabel,
    getGuestProfileLabel,
    formatMemory,
    selectedTab: uiState.selectedVmTab,
    canStart,
    canDelete,
    running,
    stopped,
    hasDisk,
    hasNetwork,
    hasRemote,
    androidImageMissing,
    primaryAddress: vmPrimaryAddress(machine),
    publishedServiceCount,
    recoverableError
  });
  if (uiState.selectedVmTab === "snapshots") void renderInlineSnapshots(machine);
  if (uiState.selectedVmTab === "logs") void renderInlineLogs(machine);
}

async function renderInlineSnapshots(machine) {
  const target = elements.vmDetailPanel?.querySelector("#inline-snapshot-content");
  if (!target || !machine) return;
  try {
    const snapshots = await invoke("list_snapshots", { vmId: machine.id });
    if (!Array.isArray(snapshots) || snapshots.length === 0) {
      target.innerHTML = `<div class="workspace-inline-empty">Bu VM icin snapshot yok.<div class="workspace-section-actions"><button class="detail-secondary" data-action="snapshots" data-vm-id="${escapeAttribute(machine.id)}" data-vm-name="${escapeAttribute(machine.name)}">Snapshot Yonet</button></div></div>`;
      return;
    }
    target.innerHTML = `<div class="workspace-section-heading"><div><strong>Snapshot Zaman Cizgisi</strong><span>${snapshots.length} geri donus noktasi</span></div><button class="detail-secondary" data-action="snapshots" data-vm-id="${escapeAttribute(machine.id)}" data-vm-name="${escapeAttribute(machine.name)}">Yonet</button></div><div class="inline-timeline">${snapshots.map((snapshot) => `<div class="inline-timeline-row"><i></i><div><strong>${escapeHtml(snapshot.name || snapshot.id)}</strong><small>${escapeHtml(formatTimestamp(snapshot.created_at_unix_ms))} / ${Number(snapshot.disk_count || 0)} disk</small></div><span class="resource-pill">SNAPSHOT</span></div>`).join("")}</div>`;
  } catch (error) {
    target.innerHTML = `<div class="workspace-inline-empty">Snapshotlar yuklenemedi: ${escapeHtml(String(error))}</div>`;
  }
}

async function renderInlineLogs(machine) {
  const target = elements.vmDetailPanel?.querySelector("#inline-log-content");
  if (!target || !machine) return;
  try {
    const logs = await invoke("list_local_logs");
    const machineLogs = Array.isArray(logs) ? logs.filter((log) => !log.relative_path || String(log.relative_path).includes(machine.id) || String(log.category || "").toLowerCase() === "engine") : [];
    if (machineLogs.length === 0) {
      target.innerHTML = `<div class="workspace-inline-empty">Bu VM icin gosterilecek gunluk yok.<div class="workspace-section-actions"><button class="detail-secondary" data-action="logs" data-vm-id="${escapeAttribute(machine.id)}">Gunluk Merkezi</button></div></div>`;
      return;
    }
    target.innerHTML = `<div class="workspace-section-heading"><div><strong>Calisma Gunlukleri</strong><span>${machineLogs.length} kayit</span></div><button class="detail-secondary" data-action="logs" data-vm-id="${escapeAttribute(machine.id)}">Tum Gunlukler</button></div>${machineLogs.slice(0, 8).map((log) => `<div class="inline-log-row"><div><strong>${escapeHtml(log.relative_path || "log")}</strong><small>${escapeHtml(log.category || "runtime")} / ${escapeHtml(formatBytes(log.size_bytes))} / ${escapeHtml(formatTimestamp(log.modified_at_unix_ms))}</small></div><span class="resource-pill">LOG</span></div>`).join("")}`;
  } catch (error) {
    target.innerHTML = `<div class="workspace-inline-empty">Gunlukler yuklenemedi: ${escapeHtml(String(error))}</div>`;
  }
}

async function handleBulkVmAction(action) {
  const visibleMachines = getFilteredMachines(uiState.machines);
  const candidates = visibleMachines.filter((machine) => {
    const state = String(machine.state || "").toLowerCase();
    const recoverableStartError = isRecoverableStartPreflightError(machine);
    return action === "start"
      ? ((state === STATE_STOPPED || recoverableStartError)
        && Number(machine.disk_count || 0) > 0
        && hasAndroidImageAssignment(machine))
      : state === STATE_RUNNING;
  });
  if (candidates.length === 0) {
    showToast("Toplu islem", action === "start" ? "Baslatilabilir VM yok." : "Durdurulabilir VM yok.", "info");
    return;
  }
  if (action === "stop" && !window.confirm(`${candidates.length} VM durdurulsun mu?`)) return;

  setButtonBusy(elements.bulkStartButton, true);
  setButtonBusy(elements.bulkStopButton, true);
  let success = 0;
  const failures = [];
  for (const machine of candidates) {
    try {
      if (action === "start") await invoke("start_vm", { vmId: machine.id });
      else await invoke("stop_vm", { vmId: machine.id });
      success += 1;
    } catch (error) {
      failures.push(`${machine.name}: ${String(error)}`);
    }
  }
  const operation = action === "start" ? "Toplu VM Baslat" : "Toplu VM Durdur";
  const detail = `${success}/${candidates.length} basarili${failures.length ? ` / ${failures.length} hata` : ""}`;
  recordActivity(operation, `${candidates.length} VM`, failures.length ? "error" : "success", detail);
  showToast(operation, detail, failures.length ? "error" : "success");
  if (failures.length) showError(failures.join(" | "));
  setButtonBusy(elements.bulkStartButton, false);
  setButtonBusy(elements.bulkStopButton, false);
  await refreshDashboard();
}

async function handleVmAction(event) {
  const button = event.target.closest("[data-action]");
  if (!button) return;
  const vmId = button.dataset.vmId;
  const machine = uiState.machines.find((item) => item.id === vmId);
  const vmName = button.dataset.vmName || machine?.name || vmId;
  const action = button.dataset.action;

  if (action === "select") {
    uiState.selectedVmId = vmId;
    uiState.selectedVmTab = "overview";
    renderMachineViews();
    return;
  }

  if (action === "network") {
    await openNetworkModal(vmId, { vmId, fromAndroid: false });
    return;
  }

  if (action === "logs") {
    await openLogsModal();
    return;
  }

  if (action === "add-disk") {
    await openStorageModal(vmId);
    return;
  }

  if (action === "assign-image") {
    await openAndroidImagesForVm(vmId);
    return;
  }

  if (action === "media") {
    openInstallerMediaModal(vmId);
    return;
  }

  if (action === "android") {
    await openAndroidModal(vmId, vmName, button.dataset.vmState || STATE_STOPPED);
    return;
  }
  if (action === "snapshots") {
    await openSnapshotModal(vmId, vmName);
    return;
  }
  if (action === "clone") {
    openCloneModal(vmId, vmName);
    return;
  }

  if (action === "edit") {
    openVmEditModal(machine);
    return;
  }

  if (action === "delete-vm") {
    if (!machine || ![STATE_STOPPED, "error"].includes(String(machine.state || "").toLowerCase())) return;
    if (!window.confirm(`${vmName} ve VM klasorundeki disk/medya verileri kalici olarak silinsin mi?`)) return;
    setButtonBusy(button, true);
    try {
      await invoke("delete_vm", { vmId });
      recordActivity("VM Sil", vmName, "success", vmId);
      showToast("VM silindi", vmName, "success");
      if (uiState.selectedVmId === vmId) uiState.selectedVmId = null;
      await refreshDashboard();
    } catch (error) {
      recordActivity("VM Sil", vmName, "error", String(error));
      showError(String(error));
      setButtonBusy(button, false);
    }
    return;
  }

  if (action === "connect") {
    openConnectionModal(machine);
    return;
  }

  if (action === "display") {
    setButtonBusy(button, true);
    try {
      await invoke("open_display", { vmId });
      recordActivity("Konsol Ac", vmName, "success", vmId);
      showToast("Konsol acildi", vmName, "success");
    } catch (error) {
      recordActivity("Konsol Ac", vmName, "error", String(error));
      showError(String(error));
    } finally {
      setButtonBusy(button, false);
    }
    return;
  }

  setButtonBusy(button, true);
  const operation = action === "start" ? "VM Baslat" : "VM Durdur";
  try {
    if (action === "start") await invoke("start_vm", { vmId });
    if (action === "stop") await invoke("stop_vm", { vmId });
    recordActivity(operation, vmName, "success", vmId);
    showToast(action === "start" ? "VM baslatildi" : "VM durduruldu", vmName, "success");
    await refreshDashboard();
  } catch (error) {
    recordActivity(operation, vmName, "error", String(error));
    showError(String(error));
    setButtonBusy(button, false);
  }
}

function uniqueBy(items, key) {
  const seen = new Set();
  return items.filter((item) => {
    const value = item[key];
    if (seen.has(value)) return false;
    seen.add(value);
    return true;
  });
}

function catalogForSelectedFamily() {
  return uiState.guestCatalog.filter((template) => template.family === uiState.selectedGuestFamily);
}

function renderGuestFamilies() {
  const families = uniqueBy(uiState.guestCatalog, "family");
  elements.guestFamilySelect.innerHTML = families.map((template) => `<option value="${escapeAttribute(template.family)}">${escapeHtml(template.family_label)}</option>`).join("");
  if (families.some((template) => template.family === uiState.selectedGuestFamily)) elements.guestFamilySelect.value = uiState.selectedGuestFamily;
}

function fillSelect(select, entries, valueKey, labelKey, preferredValue = null) {
  select.innerHTML = entries.map((entry) => `<option value="${escapeAttribute(entry[valueKey])}">${escapeHtml(entry[labelKey])}</option>`).join("");
  if (preferredValue && entries.some((entry) => entry[valueKey] === preferredValue)) select.value = preferredValue;
}

function currentGuestTemplate() {
  return uiState.guestCatalog.find((template) =>
    template.family === uiState.selectedGuestFamily
    && template.product_id === elements.guestProductSelect.value
    && template.release_id === elements.guestReleaseSelect.value
    && template.profile_id === elements.guestProfileSelect.value
  ) || null;
}

function renderGuestCatalogSelectors(preserve = {}) {
  const familyTemplates = catalogForSelectedFamily();
  const products = uniqueBy(familyTemplates, "product_id");
  fillSelect(elements.guestProductSelect, products, "product_id", "product_label", preserve.product);

  const productTemplates = familyTemplates.filter((template) => template.product_id === elements.guestProductSelect.value);
  const releases = uniqueBy(productTemplates, "release_id");
  fillSelect(elements.guestReleaseSelect, releases, "release_id", "release_label", preserve.release);

  const releaseTemplates = productTemplates.filter((template) => template.release_id === elements.guestReleaseSelect.value);
  const profiles = uniqueBy(releaseTemplates, "profile_id");
  fillSelect(elements.guestProfileSelect, profiles, "profile_id", "profile_label", preserve.profile);
  applyGuestTemplateSelection();
}

function slugifyVmId(value) {
  return String(value || "")
    .toLowerCase()
    .replaceAll(/[^a-z0-9]+/g, "-")
    .replaceAll(/^-+|-+$/g, "")
    .slice(0, 64) || "new-vm";
}

function suggestUniqueVmId(value) {
  const base = slugifyVmId(value);
  const used = new Set(uiState.machines.map((machine) => String(machine.id || "").toLowerCase()));
  if (!used.has(base)) return base;
  let index = 2;
  while (used.has(`${base}-${index}`)) index += 1;
  return `${base}-${index}`;
}

function nextResourceId(machine, kind) {
  if (!machine) return `${kind}-${String(1).padStart(RESOURCE_INDEX_WIDTH, "0")}`;
  const base = slugifyVmId(machine.id || machine.name);
  const resources = kind === RESOURCE_KIND_DISK
    ? (Array.isArray(machine.disks) ? machine.disks : [])
    : (Array.isArray(machine.networks) ? machine.networks : []);
  const prefix = `${base}-${kind}-`;
  let highest = 0;
  for (const resource of resources) {
    const id = String(resource?.id || "");
    if (!id.startsWith(prefix)) continue;
    const suffix = Number(id.slice(prefix.length));
    if (Number.isInteger(suffix) && suffix > highest) highest = suffix;
  }
  let index = Math.max(highest + 1, resources.length + 1, 1);
  const used = new Set(resources.map((resource) => String(resource?.id || "")));
  let candidate = `${prefix}${String(index).padStart(RESOURCE_INDEX_WIDTH, "0")}`;
  while (used.has(candidate)) {
    index += 1;
    candidate = `${prefix}${String(index).padStart(RESOURCE_INDEX_WIDTH, "0")}`;
  }
  return candidate;
}

function refreshStorageDiskSuggestion() {
  const machine = selectedStorageMachine();
  if (!machine) return;
  elements.storageDiskId.value = nextResourceId(machine, RESOURCE_KIND_DISK);
}

function refreshNetworkIdSuggestion() {
  const machine = selectedNetworkMachine();
  if (!machine) return;
  elements.networkId.value = nextResourceId(machine, RESOURCE_KIND_NETWORK);
}

function applyGuestTemplateSelection() {
  const template = currentGuestTemplate();
  if (!template) {
    elements.guestTemplateInfo.textContent = "Bu secim icin guest template bulunamadi.";
    if (elements.createSelectedFamily) elements.createSelectedFamily.textContent = "-";
    if (elements.createSelectedProduct) elements.createSelectedProduct.textContent = "-";
    if (elements.createSelectedRelease) elements.createSelectedRelease.textContent = "-";
    if (elements.createSelectedProfile) elements.createSelectedProfile.textContent = "-";
    if (elements.createSelectedArchitecture) elements.createSelectedArchitecture.textContent = "-";
    if (elements.createSelectedFirmware) elements.createSelectedFirmware.textContent = "-";
    if (elements.createRecommendedDisk) elements.createRecommendedDisk.textContent = "-";
    if (elements.createNextDisk) elements.createNextDisk.textContent = "Bekleniyor";
    if (elements.createNextMedia) elements.createNextMedia.textContent = "Bekleniyor";
    if (elements.createNextNetwork) elements.createNextNetwork.textContent = "Turkuaz NAT";
    elements.createSubmit.disabled = true;
    return;
  }
  elements.createSubmit.disabled = false;
  if (uiState.createMediaTemplateId && uiState.createMediaTemplateId !== template.id) {
    stopCreateMediaPolling();
    uiState.createMediaTemplateId = null;
    uiState.createMediaSources = [];
    uiState.createMediaSource = null;
    uiState.createMediaSelectedMediaId = null;
    uiState.createMediaLocalPath = null;
    uiState.createMediaReady = false;
    uiState.createAndroidImageId = null;
    uiState.createAndroidImageDeferred = false;
    stopCreateAndroidImagePolling();
    if (elements.createMediaLocalPath) elements.createMediaLocalPath.value = "";
  }
  elements.vmGuestProfile.value = template.guest_profile;
  elements.vmGuestTemplateId.value = template.id;
  elements.guestArchitecture.innerHTML = `<option value="${escapeAttribute(template.architecture)}">${escapeHtml(template.architecture)}</option>`;
  const suggestedName = `${template.product_label} ${template.release_label} ${template.profile_label}`.trim();
  if (!uiState.vmIdentityTouched) elements.vmName.value = suggestedName;
  if (!uiState.vmIdTouched) {
    const identitySource = uiState.vmIdentityTouched ? elements.vmName.value : `${template.product_id}-${template.release_id}-${template.profile_id}`;
    elements.vmId.value = suggestUniqueVmId(identitySource);
  }
  elements.vmCpu.value = String(template.recommended_vcpu_count);
  elements.vmMemory.value = String(template.recommended_memory_mib);
  if (elements.createDiskSize && !uiState.createDiskTouched) elements.createDiskSize.value = String(template.recommended_disk_size_gib);
  syncCreatePlannedResourceIds();
  elements.guestTemplateInfo.textContent = `${template.description} Kaynak: ${template.source_label}. Firmware: ${template.firmware.toUpperCase()}.`;
  elements.guestRecommendation.textContent = `Onerilen kaynaklar: ${template.recommended_vcpu_count} vCPU / ${formatMemory(template.recommended_memory_mib)} RAM / ${template.recommended_disk_size_gib} GiB disk.`;
  updateCreateSummary();
}

function currentCreateStepIndex() {
  const index = CREATE_STEP_ORDER.indexOf(uiState.createStep);
  return index >= 0 ? index : 0;
}

function createDiskId() {
  return `${slugifyVmId(elements.vmId.value)}-disk-${String(1).padStart(RESOURCE_INDEX_WIDTH, "0")}`;
}

function createNetworkId() {
  return `${slugifyVmId(elements.vmId.value)}-net-${String(1).padStart(RESOURCE_INDEX_WIDTH, "0")}`;
}

function syncCreatePlannedResourceIds() {
  const diskId = createDiskId();
  const networkId = createNetworkId();
  if (elements.createDiskIdPreview) elements.createDiskIdPreview.textContent = diskId;
  if (elements.createNetworkId && (!elements.createNetworkId.value || elements.createNetworkId.dataset.auto === "true")) {
    elements.createNetworkId.value = networkId;
    elements.createNetworkId.dataset.auto = "true";
  }
}

function stopCreateMediaPolling() {
  if (uiState.createMediaPollTimer) {
    window.clearInterval(uiState.createMediaPollTimer);
    uiState.createMediaPollTimer = null;
  }
}

function renderCreateMediaDownload(status) {
  if (!status) return;
  const state = status.state || "not_started";
  const title = uiState.createMediaSource?.label || "Kurulum ISO'su";
  if (state === "not_started") {
    uiState.createMediaProgressSample = null;
    downloadProgressView?.hide(elements.createMediaDownloadProgress);
  } else {
    const progress = installerMediaProgressModel(status, uiState.createMediaProgressSample, title);
    uiState.createMediaProgressSample = progress.sample;
    downloadProgressView?.render(elements.createMediaDownloadProgress, progress.model);
  }
  uiState.createMediaReady = state === "ready";
  const active = ["downloading", "verifying", "cancelling"].includes(state);
  elements.createMediaDownloadButton.disabled = active;
  elements.createMediaCancelButton.classList.toggle("hidden", !active);
  elements.createMediaCancelButton.disabled = state === "cancelling";
  if (uiState.createMediaReady || ["failed", "cancelled"].includes(state)) stopCreateMediaPolling();
}

async function refreshCreateMediaDownloadStatus(showFailure = false) {
  if (!uiState.createMediaTemplateId || !uiState.createMediaSelectedMediaId || !installerMediaSourceManagedDownload(uiState.createMediaSource)) return null;
  try {
    const status = await invoke("get_installer_media_download", { request: { guest_template_id: uiState.createMediaTemplateId, media_id: uiState.createMediaSelectedMediaId } });
    renderCreateMediaDownload(status);
    return status;
  } catch (error) {
    if (showFailure) showError(String(error));
    return null;
  }
}

function startCreateMediaPolling() {
  stopCreateMediaPolling();
  uiState.createMediaPollTimer = window.setInterval(() => refreshCreateMediaDownloadStatus(false), 1200);
}

function updateCreateMediaModePanels() {
  const mode = elements.createMediaMode.value;
  elements.createMediaOfficialPanel.classList.toggle("hidden", mode !== "official");
  elements.createMediaLocalPanel.classList.toggle("hidden", mode !== "local");
  elements.createMediaLaterPanel.classList.toggle("hidden", mode !== "later");
}

async function applyCreateMediaSource(source, template) {
  stopCreateMediaPolling();
  uiState.createMediaSource = source || null;
  uiState.createMediaSelectedMediaId = source?.id || null;
  uiState.createMediaReady = false;
  downloadProgressView?.hide(elements.createMediaDownloadProgress);
  uiState.createMediaProgressSample = null;
  elements.createMediaSourceNote.textContent = source?.note || "Uygun yonetilen ISO kaynagi bulunamadi. Yerel ISO veya daha sonra ayarla secenegini kullanin.";
  if (!source || !installerMediaSourceManagedDownload(source) || !installerMediaSourceCompatible(source, template)) {
    elements.createMediaDownloadButton.disabled = true;
    return;
  }
  elements.createMediaDownloadButton.disabled = false;
  elements.createMediaDownloadButton.textContent = source.recommended ? "Onerilen ISO'yu Indir" : "Secilen ISO'yu Indir";
  const status = await refreshCreateMediaDownloadStatus(false);
  if (status && ["downloading", "verifying", "cancelling"].includes(status.state)) startCreateMediaPolling();
}

function currentCreateAndroidRelease() {
  const template = currentGuestTemplate();
  if (!template || template.source_kind !== "android_image") return null;
  return normalizeAndroidRelease(template.release_id || template.release_label);
}

function stopCreateAndroidImagePolling() {
  if (!uiState.createAndroidImagePollTimer) return;
  clearTimeout(uiState.createAndroidImagePollTimer);
  uiState.createAndroidImagePollTimer = null;
}

function createAndroidImagePanelActive() {
  return !elements.createModal.classList.contains("hidden")
    && uiState.createStep === "resource"
    && !elements.createMediaAndroidPanel.classList.contains("hidden")
    && Boolean(currentCreateAndroidRelease());
}

function scheduleCreateAndroidImagePoll(images, releaseId) {
  stopCreateAndroidImagePolling();
  if (!releaseId || !createAndroidImagePanelActive()) return;
  const image = androidImageForRelease(images, releaseId);
  if (image?.state !== "installing") return;
  uiState.createAndroidImagePollTimer = window.setTimeout(() => { void refreshCreateAndroidImageOptions(false); }, 1000);
}

function androidInstallProgressMetrics(progress) {
  const downloadedBytes = Number(progress?.downloaded_bytes || 0);
  const totalBytes = Number(progress?.total_bytes || 0);
  const percent = totalBytes > 0 ? Math.min(100, Math.max(0, Math.round((downloadedBytes / totalBytes) * 100))) : null;
  return {
    downloadedBytes,
    totalBytes,
    percent,
    downloadedLabel: formatByteCount(downloadedBytes),
    totalLabel: totalBytes > 0 ? formatByteCount(totalBytes) : null,
    speedLabel: progress?.bytes_per_second ? `${formatByteCount(progress.bytes_per_second)}/s` : "-",
    etaLabel: progress?.eta_seconds !== null && progress?.eta_seconds !== undefined ? formatDuration(progress.eta_seconds) : "-",
    elapsedLabel: formatDuration(progress?.elapsed_seconds || 0),
  };
}

function androidInstallStagePosition(stage) {
  const index = ANDROID_INSTALL_ACTIVE_STAGE_ORDER.indexOf(String(stage || ""));
  return index >= 0 ? `${index + 1}/${ANDROID_INSTALL_ACTIVE_STAGE_ORDER.length}` : null;
}

function resetCreateAndroidProgress() {
  downloadProgressView?.hide(elements.createAndroidDownloadProgress);
  elements.createAndroidImagesRefresh?.classList.remove("is-busy");
  if (elements.createAndroidProgressLog) {
    elements.createAndroidProgressLog.disabled = true;
    delete elements.createAndroidProgressLog.dataset.logPath;
  }
}

function androidDownloadProgressModel(image, releaseId) {
  const state = String(image?.state || "");
  const progress = image?.install_progress || null;
  const failed = state === "failed";
  const ready = state === "ready";
  const stage = String(progress?.stage || (failed ? "failed" : (ready ? "finalizing" : "discovering")));
  const stageLabel = ANDROID_INSTALL_STAGE_LABELS[stage] || stage.toUpperCase();
  const stagePosition = androidInstallStagePosition(stage);
  const metrics = androidInstallProgressMetrics(progress);
  const determinate = metrics.percent !== null && ["downloading_device", "downloading_host"].includes(stage);
  return {
    state: failed ? "failed" : (ready ? "ready" : "active"),
    title: failed ? `Android ${releaseId} hazirlama basarisiz` : (ready ? `Android ${releaseId} hazir` : `Android ${releaseId} hazirlaniyor`),
    stageLabel: stagePosition ? `Asama ${stagePosition} - ${stageLabel}` : stageLabel,
    percent: ready ? 100 : (determinate ? metrics.percent : null),
    indeterminate: !ready && !failed && !determinate,
    bytesLabel: metrics.totalLabel
      ? `${metrics.downloadedLabel} / ${metrics.totalLabel}`
      : (metrics.downloadedBytes > 0 ? `${metrics.downloadedLabel} indirildi` : "Boyut bekleniyor"),
    speedLabel: metrics.speedLabel,
    etaLabel: metrics.etaLabel,
    elapsedLabel: metrics.elapsedLabel,
    detail: progress?.detail || image?.last_error || (state === "installing" ? "Android CI gorevi devam ediyor." : "Android image durumu bekleniyor."),
  };
}

function renderCreateAndroidImageProgress(image, releaseId) {
  const state = String(image?.state || "");
  const installing = state === "installing";
  const failed = state === "failed";
  if (!installing && !failed) {
    resetCreateAndroidProgress();
    return;
  }

  elements.createAndroidImagesRefresh?.classList.toggle("is-busy", installing);
  downloadProgressView?.render(elements.createAndroidDownloadProgress, androidDownloadProgressModel(image, releaseId));
  if (elements.createAndroidProgressCancel) {
    elements.createAndroidProgressCancel.classList.toggle("hidden", !installing);
    elements.createAndroidProgressCancel.disabled = !installing;
  }
  if (elements.createAndroidProgressLog) {
    const logPath = image?.install_progress?.log_path || "";
    elements.createAndroidProgressLog.disabled = !logPath;
    if (logPath) elements.createAndroidProgressLog.dataset.logPath = logPath;
    else delete elements.createAndroidProgressLog.dataset.logPath;
  }
}

async function refreshCreateAndroidImageOptions(showFailure = false) {
  const template = currentGuestTemplate();
  const releaseId = currentCreateAndroidRelease();
  if (!template || !releaseId) {
    stopCreateAndroidImagePolling();
    elements.createAndroidImageSelect.innerHTML = '<option value="">Daha sonra ayarla</option>';
    elements.createAndroidImagesRefresh.disabled = true;
    elements.createAndroidImagesRefresh.textContent = "Android Image Hazirla";
    elements.createAndroidImageNote.textContent = "Android surumu secilmedi.";
    resetCreateAndroidProgress();
    return [];
  }

  const images = await refreshAndroidImages(showFailure);
  const ready = images.filter((image) => image.state === "ready"
    && isAndroidSdkImage(image)
    && androidImageRelease(image) === releaseId
    && (!image.architecture || image.architecture === template.architecture));
  elements.createAndroidImageSelect.innerHTML = `<option value="">Daha sonra ayarla</option>${ready.map((image) => `<option value="${escapeAttribute(image.id)}">${escapeHtml(image.name)} / Android ${escapeHtml(releaseId)}</option>`).join("")}`;

  const selectedReady = ready.find((image) => image.id === uiState.createAndroidImageId) || null;
  const preferredReady = selectedReady || (!uiState.createAndroidImageDeferred ? ready[0] || null : null);
  uiState.createAndroidImageId = preferredReady?.id || null;
  elements.createAndroidImageSelect.value = uiState.createAndroidImageId || "";

  const releaseImage = androidImageForRelease(images, releaseId);
  const installing = releaseImage?.state === "installing";
  const failed = releaseImage?.state === "failed";
  renderCreateAndroidImageProgress(releaseImage, releaseId);
  elements.createAndroidImagesRefresh.disabled = installing;
  if (installing) elements.createAndroidImagesRefresh.textContent = `Android ${releaseId} indiriliyor...`;
  else if (ready.length > 0) elements.createAndroidImagesRefresh.textContent = `Android ${releaseId} READY Yenile`;
  else if (failed) elements.createAndroidImagesRefresh.textContent = `Android ${releaseId} Tekrar Indir`;
  else elements.createAndroidImagesRefresh.textContent = `Android ${releaseId} Image Indir`;

  if (ready.length > 0) {
    elements.createAndroidImageNote.textContent = `Android ${releaseId} icin ${ready.length} uyumlu READY image bulundu. Secilen image VM olusurken otomatik atanir.`;
  } else if (installing) {
    elements.createAndroidImageNote.textContent = `Android ${releaseId} resmi Android SDK System Image katalogundan indiriliyor ve dogrulaniyor. Gorev durumu otomatik yenilenir.`;
  } else if (failed) {
    elements.createAndroidImageNote.textContent = `Android ${releaseId} image hazirlama son denemede basarisiz oldu. Tekrar Indir ile yeniden deneyebilirsiniz.`;
  } else {
    elements.createAndroidImageNote.textContent = `Android ${releaseId} icin READY image yok. Image Indir ile bu surumu resmi Android SDK System Image katalogundan hazirlayin veya daha sonra ayarlayin.`;
  }
  scheduleCreateAndroidImagePoll(images, releaseId);
  return ready;
}

async function handleCreateAndroidImageAction() {
  const releaseId = currentCreateAndroidRelease();
  if (!releaseId) { showError("Android surumu secilmedi."); return; }
  const currentImage = androidImageForRelease(uiState.androidImages, releaseId);
  if (currentImage?.state === "ready" && isAndroidSdkImage(currentImage) && androidImageRelease(currentImage) === releaseId) {
    await refreshCreateAndroidImageOptions(true);
    return;
  }
  uiState.createAndroidImageDeferred = false;
  setButtonBusy(elements.createAndroidImagesRefresh, true);
  try {
    await ensureAndroidRelease(releaseId);
    await refreshCreateAndroidImageOptions(true);
  } finally {
    setButtonBusy(elements.createAndroidImagesRefresh, false);
    const latest = androidImageForRelease(uiState.androidImages, releaseId);
    if (latest?.state === "installing") {
      elements.createAndroidImagesRefresh.disabled = true;
      elements.createAndroidImagesRefresh.classList.add("is-busy");
      elements.createAndroidImagesRefresh.textContent = `Android ${releaseId} indiriliyor...`;
      renderCreateAndroidImageProgress(latest, releaseId);
    }
  }
}

async function handleCreateAndroidProgressCancel() {
  const releaseId = currentCreateAndroidRelease();
  if (!releaseId) return;
  const image = androidImageForRelease(uiState.androidImages, releaseId);
  if (!image || image.state !== "installing") return;
  try {
    await invoke("cancel_android_image_distribution", { request: { image_id: image.id } });
    await refreshCreateAndroidImageOptions(true);
  } catch (error) {
    showError(String(error));
  }
}

async function handleCreateAndroidProgressLog() {
  const logPath = elements.createAndroidProgressLog?.dataset.logPath || "";
  if (!logPath) { showError("Android image kurulum log yolu henuz hazir degil."); return; }
  try {
    await invoke("open_android_image_install_log", { logPath });
  } catch (error) {
    showError(String(error));
  }
}

async function prepareCreateMediaStep() {
  const template = currentGuestTemplate();
  if (!template) return;
  elements.createMediaIsoPanel.classList.toggle("hidden", template.source_kind === "android_image");
  elements.createMediaAndroidPanel.classList.toggle("hidden", template.source_kind !== "android_image");
  if (uiState.createMediaTemplateId === template.id) {
    if (template.source_kind === "android_image") await refreshCreateAndroidImageOptions();
    else {
      updateCreateMediaModePanels();
      const status = await refreshCreateMediaDownloadStatus(false);
      if (status && ["downloading", "verifying", "cancelling"].includes(status.state)) startCreateMediaPolling();
    }
    return;
  }
  stopCreateMediaPolling();
  uiState.createMediaTemplateId = template.id;
  uiState.createMediaSources = [];
  uiState.createMediaSource = null;
  uiState.createMediaSelectedMediaId = null;
  uiState.createMediaLocalPath = null;
  uiState.createMediaReady = false;
  uiState.createAndroidImageId = null;
  uiState.createAndroidImageDeferred = false;
  stopCreateAndroidImagePolling();
  elements.createMediaLocalPath.value = "";
  if (template.source_kind === "android_image") {
    await refreshCreateAndroidImageOptions();
    return;
  }
  const managedSources = installerMediaSourcesForTemplate(template).filter((source) => installerMediaSourceManagedDownload(source) && installerMediaSourceCompatible(source, template));
  uiState.createMediaSources = managedSources;
  elements.createMediaSourceSelect.innerHTML = managedSources.map((source) => `<option value="${escapeAttribute(source.id)}">${escapeHtml(installerMediaOptionLabel(source, template))}</option>`).join("");
  const recommended = managedSources.find((source) => source.recommended) || managedSources[0] || null;
  if (recommended) {
    elements.createMediaMode.value = "official";
    elements.createMediaSourceSelect.value = recommended.id;
    await applyCreateMediaSource(recommended, template);
  } else if (template.source_kind === "manual") {
    elements.createMediaMode.value = "local";
    await applyCreateMediaSource(null, template);
  } else {
    elements.createMediaMode.value = "later";
    await applyCreateMediaSource(null, template);
  }
  updateCreateMediaModePanels();
}

async function handleCreateMediaSourceChange() {
  const template = currentGuestTemplate();
  const source = uiState.createMediaSources.find((item) => item.id === elements.createMediaSourceSelect.value) || null;
  await applyCreateMediaSource(source, template);
}

async function startCreateMediaDownload() {
  if (!uiState.createMediaTemplateId || !uiState.createMediaSelectedMediaId || !installerMediaSourceManagedDownload(uiState.createMediaSource)) return;
  setButtonBusy(elements.createMediaDownloadButton, true);
  try {
    const status = await invoke("start_installer_media_download", { request: { guest_template_id: uiState.createMediaTemplateId, media_id: uiState.createMediaSelectedMediaId } });
    renderCreateMediaDownload(status);
    if (["downloading", "verifying", "cancelling"].includes(status.state)) startCreateMediaPolling();
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(elements.createMediaDownloadButton, false);
  }
}

async function cancelCreateMediaDownload() {
  if (!uiState.createMediaTemplateId || !uiState.createMediaSelectedMediaId) return;
  setButtonBusy(elements.createMediaCancelButton, true);
  try {
    const status = await invoke("cancel_installer_media_download", { request: { guest_template_id: uiState.createMediaTemplateId, media_id: uiState.createMediaSelectedMediaId } });
    renderCreateMediaDownload(status);
    if (["downloading", "verifying", "cancelling"].includes(status.state)) startCreateMediaPolling();
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(elements.createMediaCancelButton, false);
  }
}

async function pickCreateMediaLocalIso() {
  try {
    const selected = await invoke("pick_installer_iso");
    if (!selected) return;
    uiState.createMediaLocalPath = selected;
    elements.createMediaLocalPath.value = selected;
    clearError();
  } catch (error) {
    showError(String(error));
  }
}

function updateCreateNetworkFields() {
  const bridge = elements.createNetworkProfile.value === "bridge";
  elements.createNetworkBridgeWrap.classList.toggle("hidden", !bridge);
  elements.createNetworkTapWrap.classList.toggle("hidden", !bridge);
}

function createMediaSummaryLabel(template) {
  if (!template) return "-";
  if (template.source_kind === "android_image") {
    const selected = elements.createAndroidImageSelect?.value || "";
    if (!selected) return "Android Image: daha sonra";
    const image = uiState.androidImages.find((item) => item.id === selected);
    return `Android Image: ${image?.name || selected}`;
  }
  const mode = elements.createMediaMode?.value || "later";
  if (mode === "official") return uiState.createMediaReady ? `Resmi ISO: ${uiState.createMediaSource?.label || "Hazir"}` : "Resmi ISO: hazir degil";
  if (mode === "local") return uiState.createMediaLocalPath ? `Yerel ISO: ${uiState.createMediaLocalPath.split(/[\\/]/).pop()}` : "Yerel ISO secilmedi";
  return "Medya daha sonra ayarlanacak";
}

function updateCreateReviewFields() {
  const template = currentGuestTemplate();
  if (elements.createReviewName) elements.createReviewName.textContent = elements.vmName.value.trim() || "-";
  if (elements.createReviewId) elements.createReviewId.textContent = elements.vmId.value.trim() || "-";
  if (elements.createReviewCpu) elements.createReviewCpu.textContent = `${Number(elements.vmCpu.value || 0)} vCPU`;
  if (elements.createReviewMemory) elements.createReviewMemory.textContent = formatMemory(Number(elements.vmMemory.value || 0));
  if (elements.createReviewDisk) elements.createReviewDisk.textContent = `${Number(elements.createDiskSize.value || 0)} GiB / ${createDiskId()}`;
  if (elements.createReviewMedia) elements.createReviewMedia.textContent = createMediaSummaryLabel(template);
  if (elements.createReviewNetwork) elements.createReviewNetwork.textContent = `${elements.createNetworkProfile.options[elements.createNetworkProfile.selectedIndex]?.text || elements.createNetworkProfile.value} / ${elements.createNetworkId.value}`;
  updateCreateSummary();
}

function showCreateStep(step) {
  const normalized = CREATE_STEP_ORDER.includes(step) ? step : CREATE_STEP_ORDER[0];
  uiState.createStep = normalized;
  const currentIndex = CREATE_STEP_ORDER.indexOf(normalized);
  document.querySelectorAll("[data-create-section]").forEach((node) => node.classList.toggle("hidden", node.dataset.createSection !== normalized));
  document.querySelectorAll("[data-create-progress]").forEach((node, index) => {
    node.classList.toggle("active", index === currentIndex);
    node.classList.toggle("done", index < currentIndex);
  });
  elements.createBackButton.classList.toggle("hidden", currentIndex === 0);
  elements.createNextButton.classList.toggle("hidden", currentIndex === CREATE_STEP_ORDER.length - 1);
  elements.createSubmit.classList.toggle("hidden", currentIndex !== CREATE_STEP_ORDER.length - 1);
  elements.createSubmit.textContent = "VM Olustur";
  if (normalized !== "resource") stopCreateAndroidImagePolling();
  if (normalized === "resource") {
    syncCreatePlannedResourceIds();
    updateCreateNetworkFields();
  }
  if (normalized === "summary") updateCreateReviewFields();
}

function validateCreateStep(step) {
  clearError();
  if (step === "catalog") {
    if (!currentGuestTemplate()) { showError("Isletim sistemi secimini tamamlayin."); return false; }
    if (!elements.vmName.reportValidity() || !elements.vmId.reportValidity()) return false;
    if (!elements.vmName.value.trim() || !elements.vmId.value.trim()) { showError("Makine adi zorunludur."); return false; }
    syncCreatePlannedResourceIds();
    return true;
  }
  if (step === "resource") {
    if (!elements.vmCpu.reportValidity() || !elements.vmMemory.reportValidity() || !elements.createDiskSize.reportValidity()) return false;
    if (Number(elements.vmCpu.value) < 1 || Number(elements.vmMemory.value) < 512) { showError("CPU veya bellek degeri gecersiz."); return false; }
    if (Number(elements.createDiskSize.value) < 8) { showError("Disk boyutu en az 8 GiB olmalidir."); return false; }
    const template = currentGuestTemplate();
    if (template?.source_kind !== "android_image") {
      const mode = elements.createMediaMode.value;
      if (mode === "official" && !uiState.createMediaReady) { showError("Resmi ISO indirmesini tamamlayin veya Yerel ISO / Daha Sonra secin."); return false; }
      if (mode === "local" && !uiState.createMediaLocalPath) { showError("Yerel ISO dosyasini secin."); return false; }
    }
    if (!elements.createNetworkId.reportValidity() || !elements.createNetworkId.value.trim()) { showError("Ag baglanti ID zorunludur."); return false; }
    return true;
  }
  return true;
}

async function handleCreateNext() {
  if (!validateCreateStep(uiState.createStep)) return;
  const index = currentCreateStepIndex();
  if (index >= CREATE_STEP_ORDER.length - 1) return;
  const next = CREATE_STEP_ORDER[index + 1];
  if (next === "resource") {
    await prepareCreateMediaStep();
    syncCreatePlannedResourceIds();
    updateCreateNetworkFields();
  }
  showCreateStep(next);
}

function handleCreateBack() {
  const index = currentCreateStepIndex();
  if (index <= 0) return;
  showCreateStep(CREATE_STEP_ORDER[index - 1]);
}

function wireCreateFlowNavigation() {
  elements.createNextButton?.addEventListener("click", () => { void handleCreateNext(); });
  elements.createBackButton?.addEventListener("click", handleCreateBack);
  elements.createMediaMode?.addEventListener("change", updateCreateMediaModePanels);
  elements.createMediaSourceSelect?.addEventListener("change", () => { void handleCreateMediaSourceChange(); });
  elements.createMediaDownloadButton?.addEventListener("click", () => { void startCreateMediaDownload(); });
  elements.createMediaCancelButton?.addEventListener("click", () => { void cancelCreateMediaDownload(); });
  elements.createMediaPickButton?.addEventListener("click", () => { void pickCreateMediaLocalIso(); });
  elements.createAndroidImagesRefresh?.addEventListener("click", () => { void handleCreateAndroidImageAction(); });
  elements.createAndroidProgressCancel?.addEventListener("click", () => { void handleCreateAndroidProgressCancel(); });
  elements.createAndroidProgressLog?.addEventListener("click", () => { void handleCreateAndroidProgressLog(); });
  elements.createAndroidImageSelect?.addEventListener("change", () => {
    uiState.createAndroidImageId = elements.createAndroidImageSelect.value || null;
    uiState.createAndroidImageDeferred = !uiState.createAndroidImageId;
  });
  elements.createNetworkProfile?.addEventListener("change", updateCreateNetworkFields);
  elements.createNetworkId?.addEventListener("input", () => { elements.createNetworkId.dataset.auto = "false"; });
}

function updateCreateServbaySummary(template) {
  if (!template) return;
  if (elements.createSelectedFamily) elements.createSelectedFamily.textContent = template.family_label || template.family || "-";
  if (elements.createSelectedProduct) elements.createSelectedProduct.textContent = template.product_label || "-";
  if (elements.createSelectedRelease) elements.createSelectedRelease.textContent = template.release_label || "-";
  if (elements.createSelectedProfile) elements.createSelectedProfile.textContent = template.profile_label || "-";
  if (elements.createSelectedArchitecture) elements.createSelectedArchitecture.textContent = template.architecture || "-";
  if (elements.createSelectedFirmware) elements.createSelectedFirmware.textContent = String(template.firmware || "-").toUpperCase();
  if (elements.createRecommendedDisk) elements.createRecommendedDisk.textContent = `${template.recommended_disk_size_gib} GiB`;
  if (elements.createNextDisk) elements.createNextDisk.textContent = `${Number(elements.createDiskSize?.value || template.recommended_disk_size_gib)} GiB disk`;
  if (elements.createNextMedia) {
    elements.createNextMedia.textContent = template.source_kind === "android_image"
      ? "Android Image baglanacak"
      : (["iso", "manual"].includes(template.source_kind) ? "Kurulum ISO baglanacak" : "Ek medya gerekmiyor");
  }
  if (elements.createNextNetwork) elements.createNextNetwork.textContent = "Turkuaz NAT";
}

function updateCreateSummary() {
  const template = currentGuestTemplate();
  if (!template) return;
  elements.createSummary.textContent = `${template.product_label} ${template.release_label} / ${template.profile_label} / ${template.architecture} -> ${elements.vmCpu.value} vCPU, ${formatMemory(Number(elements.vmMemory.value))} RAM, ${Number(elements.createDiskSize?.value || template.recommended_disk_size_gib)} GiB disk, ${elements.createNetworkProfile?.options[elements.createNetworkProfile.selectedIndex]?.text || "Turkuaz NAT"}.`;
  updateCreateServbaySummary(template);
}

async function loadGuestCatalog() {
  if (uiState.guestCatalog.length === 0) {
    uiState.guestCatalog = await invoke("list_guest_catalog");
  }
  if (!uiState.guestCatalog.some((template) => template.family === uiState.selectedGuestFamily)) {
    uiState.selectedGuestFamily = uiState.guestCatalog[0]?.family || "other";
  }
  renderGuestFamilies();
  renderGuestCatalogSelectors();
}

async function rollbackCreateWizardVm(vmId, networkId = null) {
  if (!vmId) return;
  if (networkId) {
    try {
      await invoke("detach_vm_network", { request: { vm_id: vmId, network_id: networkId } });
    } catch (_) {
      // Best-effort compensation. VM delete below remains authoritative cleanup.
    }
  }
  try {
    await invoke("delete_vm", { vmId });
  } catch (_) {
    // Surface the original create error. Recovery can be completed from VM management if needed.
  }
}

async function handleCreate(event) {
  event.preventDefault();
  if (uiState.createStep !== "summary") {
    await handleCreateNext();
    return;
  }
  for (const step of CREATE_STEP_ORDER.slice(0, -1)) {
    if (!validateCreateStep(step)) {
      showCreateStep(step);
      return;
    }
  }
  const template = currentGuestTemplate();
  if (!template) {
    showError("Guest template secimi tamamlanmadi.");
    showCreateStep("catalog");
    return;
  }

  const request = {
    vm_id: elements.vmId.value.trim(),
    name: elements.vmName.value.trim(),
    vcpu_count: Number(elements.vmCpu.value),
    memory_mib: Number(elements.vmMemory.value),
    guest_profile: elements.vmGuestProfile.value,
    guest_template_id: elements.vmGuestTemplateId.value || null
  };
  const diskId = createDiskId();
  const networkId = elements.createNetworkId.value.trim() || createNetworkId();
  const diskSize = Number(elements.createDiskSize.value);
  const networkProfile = elements.createNetworkProfile.value;
  let vmCreated = false;
  let networkAttached = false;

  setButtonBusy(elements.createSubmit, true);
  try {
    await invoke("create_vm", { request });
    vmCreated = true;
    recordActivity("VM Olustur", request.name, "success", `${template.product_label} ${template.release_label} / ${request.vm_id}`);

    await invoke("create_vm_disk", { request: { vm_id: request.vm_id, disk_id: diskId, size_gib: diskSize } });
    recordActivity("Disk Olustur", request.vm_id, "success", `${diskId} / ${diskSize} GiB`);

    await invoke("attach_network_profile", { request: {
      vm_id: request.vm_id,
      network_id: networkId,
      profile: networkProfile,
      bridge_name: networkProfile === "bridge" ? (elements.createNetworkBridgeName.value.trim() || null) : null,
      tap_name: networkProfile === "bridge" ? (elements.createNetworkTapName.value.trim() || null) : null,
    } });
    networkAttached = true;
    recordActivity("Ag Ekle", request.vm_id, "success", `${networkId} / ${networkProfile}`);

    if (template.source_kind === "android_image") {
      const imageId = elements.createAndroidImageSelect.value || null;
      if (imageId) {
        await invoke("assign_android_image", { request: { vm_id: request.vm_id, image_id: imageId } });
        recordActivity("Android Image Ata", request.vm_id, "success", imageId);
      }
    } else {
      const mediaMode = elements.createMediaMode.value;
      if (mediaMode === "official") {
        await invoke("attach_downloaded_installer_media", { request: {
          vm_id: request.vm_id,
          guest_template_id: template.id,
          media_id: uiState.createMediaSelectedMediaId,
        } });
        recordActivity("ISO Bagla", request.vm_id, "success", uiState.createMediaSource?.label || template.id);
      } else if (mediaMode === "local") {
        await invoke("configure_installer_media", { request: { vm_id: request.vm_id, source_path: uiState.createMediaLocalPath } });
        recordActivity("Kurulum Medyasi", request.vm_id, "success", uiState.createMediaLocalPath);
      }
    }

    uiState.createdVmId = request.vm_id;
    uiState.createdVmRecommendedDiskGib = diskSize;
    uiState.createdVmSourceKind = template.source_kind;
    await refreshDashboard();
    elements.createConfigPanel.classList.add("hidden");
    elements.createCompletePanel.classList.remove("hidden");
    const mediaLabel = createMediaSummaryLabel(template);
    elements.createCompleteText.textContent = `${request.name} (${request.vm_id}) hazir. ${diskSize} GiB disk, ${elements.createNetworkProfile.options[elements.createNetworkProfile.selectedIndex]?.text || networkProfile} ve ${mediaLabel} plani uygulandi.`;
    showToast("VM hazir", request.name, "success");
    clearError();
  } catch (error) {
    if (vmCreated) await rollbackCreateWizardVm(request.vm_id, networkAttached ? networkId : null);
    recordActivity("VM Wizard", request.name, "error", String(error));
    showError(`VM olusturma akisi tamamlanamadi. Yapilan draft kaynaklar geri alindi. ${String(error)}`);
  } finally {
    setButtonBusy(elements.createSubmit, false);
  }
}

async function handlePostCreateDisk() {
  if (!uiState.createdVmId) return;
  const vmId = uiState.createdVmId;
  const size = uiState.createdVmRecommendedDiskGib;
  const nextStep = uiState.createdVmSourceKind === "android_image" ? "android-image" : (["iso", "manual"].includes(uiState.createdVmSourceKind) ? "installer-media" : "network");
  closeCreateModal();
  await openStorageModal(vmId, size, { vmId, nextStep });
}

function setAndroidImageFlowReady(ready) {
  uiState.androidImageFlowReady = Boolean(ready);
  elements.androidImageNextButton.disabled = !uiState.androidImageFlowReady;
  if (!uiState.androidImageFlowVmId) return;
  elements.androidImageFlowStatus.textContent = uiState.androidImageFlowReady ? "Android image hazir" : "Android image bekleniyor";
  elements.androidImageFlowHint.textContent = uiState.androidImageFlowReady
    ? "Image atandi. Ileri ile Ag yapilandirmasina gecin."
    : "READY image secili VM'e atandiginda Ileri aktif olur.";
}

function configureAndroidImageFlow(flowContext = null) {
  const active = Boolean(flowContext?.vmId);
  uiState.androidImageFlowVmId = active ? flowContext.vmId : null;
  uiState.androidImageFlowReady = false;
  elements.imageCenterTabs?.classList.toggle("hidden", active);
  elements.androidImageFlowProgress.classList.toggle("hidden", !active);
  elements.androidImageFlowFooter.classList.toggle("hidden", !active);
  elements.androidImageNextButton.disabled = true;

  if (!active) {
    elements.androidImagesEyebrow.textContent = "GORUNTU MERKEZI";
    elements.androidImagesTitle.textContent = "Goruntu Merkezi";
    elements.androidImagesSubtitle.textContent = "Windows/Linux kurulum ISO'larini ve Android sistem goruntulerini tek merkezden yonetin.";
    elements.androidImageFlowStatus.textContent = "Once Android image atayin.";
    elements.androidImageFlowHint.textContent = "Image atamasi basarili oldugunda Ileri aktif olur.";
    return;
  }

  elements.androidImagesEyebrow.textContent = "VM OLUSTURMA / ADIM 3";
  elements.androidImagesTitle.textContent = "Android Sistem Goruntusu";
  elements.androidImagesSubtitle.textContent = "Secili VM icin Android sistem goruntusunu hazirlayin.";
  setAndroidImageFlowReady(false);
}

async function openAndroidImagesForVm(vmId, flowContext = null) {
  const context = flowContext?.vmId ? flowContext : null;
  await openAndroidImagesModal(context);
  if ([...elements.androidImageQuickVm.options].some((option) => option.value === vmId)) {
    elements.androidImageQuickVm.value = vmId;
    renderAndroidImages(uiState.androidImages);
  }
  if (context) {
    elements.androidImageQuickVm.disabled = true;
    try {
      const assignment = await invoke("get_android_image_assignment", { vmId });
      uiState.androidAssignments[vmId] = assignment || null;
      setAndroidImageFlowReady(Boolean(assignment));
    } catch (_) {
      uiState.androidAssignments[vmId] = null;
      setAndroidImageFlowReady(false);
    }
  }
}

function openVmEditModal(machine) {
  if (!machine) return;
  elements.vmEditId.value = machine.id;
  elements.vmEditName.value = machine.name;
  elements.vmEditCpu.value = String(machine.vcpu_count);
  elements.vmEditMemory.value = String(machine.memory_mib);
  elements.vmEditModal.classList.remove("hidden");
}

function closeVmEditModal() {
  elements.vmEditModal.classList.add("hidden");
}

async function handleVmEdit(event) {
  event.preventDefault();
  const request = {
    vm_id: elements.vmEditId.value,
    name: elements.vmEditName.value.trim(),
    vcpu_count: Number(elements.vmEditCpu.value),
    memory_mib: Number(elements.vmEditMemory.value)
  };
  try {
    await invoke("update_vm", { request });
    recordActivity("VM Duzenle", request.name, "success", `${request.vcpu_count} vCPU / ${request.memory_mib} MiB`);
    showToast("VM guncellendi", request.name, "success");
    closeVmEditModal();
    await refreshDashboard();
  } catch (error) {
    recordActivity("VM Duzenle", request.name, "error", String(error));
    showError(String(error));
  }
}

async function openSnapshotModal(vmId, vmName) {
  uiState.snapshotVmId = vmId;
  uiState.snapshotVmName = vmName;
  elements.snapshotVmLabel.textContent = `${vmName} / ${vmId}`;
  elements.snapshotModal.classList.remove("hidden");
  await refreshSnapshots();
}

function closeSnapshotModal() {
  elements.snapshotModal.classList.add("hidden");
  uiState.snapshotVmId = null;
  uiState.snapshotVmName = null;
}

async function refreshSnapshots() {
  if (!uiState.snapshotVmId) return;
  try {
    const snapshots = await invoke("list_snapshots", { vmId: uiState.snapshotVmId });
    renderSnapshots(snapshots);
  } catch (error) {
    showError(String(error));
  }
}

function renderSnapshots(snapshots) {
  elements.snapshotList.innerHTML = "";
  elements.snapshotEmpty.classList.toggle("hidden", snapshots.length !== 0);
  for (const snapshot of snapshots) {
    const row = document.createElement("article");
    row.className = "snapshot-row";
    row.innerHTML = `
      <div>
        <strong>${escapeHtml(snapshot.name)}</strong>
        <span>${escapeHtml(snapshot.id)} / ${snapshot.disk_count} disk / ${formatTimestamp(snapshot.created_at_unix_ms)}</span>
      </div>
      <div class="snapshot-actions">
        <button class="vm-action secondary" data-snapshot-action="restore" data-snapshot-id="${escapeAttribute(snapshot.id)}">Geri Yukle</button>
        <button class="vm-action stop" data-snapshot-action="delete" data-snapshot-id="${escapeAttribute(snapshot.id)}">Sil</button>
      </div>
    `;
    elements.snapshotList.appendChild(row);
  }
}

async function handleSnapshotCreate(event) {
  event.preventDefault();
  if (!uiState.snapshotVmId) return;
  const request = {
    vm_id: uiState.snapshotVmId,
    snapshot_id: document.querySelector("#snapshot-id").value.trim(),
    name: document.querySelector("#snapshot-name").value.trim()
  };
  try {
    await invoke("create_snapshot", { request });
    recordActivity("Snapshot Olustur", uiState.snapshotVmName || request.vm_id, "success", request.snapshot_id);
    showToast("Snapshot olusturuldu", request.snapshot_id, "success");
    await refreshSnapshots();
    await refreshDashboard();
  } catch (error) {
    recordActivity("Snapshot Olustur", uiState.snapshotVmName || request.vm_id, "error", String(error));
    showError(String(error));
  }
}

async function handleSnapshotAction(event) {
  const button = event.target.closest("[data-snapshot-action]");
  if (!button || !uiState.snapshotVmId) return;
  const snapshotId = button.dataset.snapshotId;
  const action = button.dataset.snapshotAction;
  if (action === "delete" && !window.confirm(`Snapshot silinsin mi? ${snapshotId}`)) return;
  const request = { vm_id: uiState.snapshotVmId, snapshot_id: snapshotId };
  button.disabled = true;
  try {
    if (action === "restore") await invoke("restore_snapshot", { request });
    if (action === "delete") await invoke("delete_snapshot", { request });
    const operation = action === "restore" ? "Snapshot Geri Yukle" : "Snapshot Sil";
    recordActivity(operation, uiState.snapshotVmName || request.vm_id, "success", snapshotId);
    showToast(action === "restore" ? "Snapshot geri yuklendi" : "Snapshot silindi", snapshotId, "success");
    await refreshSnapshots();
    await refreshDashboard();
  } catch (error) {
    recordActivity(action === "restore" ? "Snapshot Geri Yukle" : "Snapshot Sil", uiState.snapshotVmName || request.vm_id, "error", String(error));
    showError(String(error));
    button.disabled = false;
  }
}

function openCloneModal(vmId, vmName) {
  uiState.cloneSourceVmId = vmId;
  uiState.cloneSourceVmName = vmName;
  elements.cloneSourceLabel.textContent = `${vmName} / ${vmId}`;
  document.querySelector("#clone-name").value = `${vmName} Clone`;
  document.querySelector("#clone-id").value = `${vmId}-clone`;
  document.querySelector("#clone-mode").value = "full";
  elements.cloneModal.classList.remove("hidden");
}

function closeCloneModal() {
  elements.cloneModal.classList.add("hidden");
  uiState.cloneSourceVmId = null;
  uiState.cloneSourceVmName = null;
}

async function handleClone(event) {
  event.preventDefault();
  if (!uiState.cloneSourceVmId) return;
  const request = {
    source_vm_id: uiState.cloneSourceVmId,
    target_vm_id: document.querySelector("#clone-id").value.trim(),
    target_name: document.querySelector("#clone-name").value.trim(),
    mode: document.querySelector("#clone-mode").value
  };
  try {
    await invoke("clone_vm", { request });
    recordActivity("VM Klonla", request.target_name, "success", `${request.source_vm_id} -> ${request.target_vm_id}`);
    showToast("VM klonlandi", request.target_name, "success");
    closeCloneModal();
    await refreshDashboard();
  } catch (error) {
    recordActivity("VM Klonla", request.target_name, "error", String(error));
    showError(String(error));
  }
}



function imageCenterIsoEntries() {
  const entries = new Map();
  for (const template of uiState.guestCatalog) {
    if (template.guest_profile === "android" || template.source_kind === "android_image") continue;
    for (const source of installerMediaSourcesForTemplate(template)) {
      if (!source?.id || entries.has(source.id)) continue;
      entries.set(source.id, {
        source,
        template,
        familyLabel: template.family_label || template.family || "Sistem",
        productLabel: template.product_label || template.product_id || template.id,
        releaseLabel: template.release_label || template.release_id || ""
      });
    }
  }
  return [...entries.values()].sort((left, right) => {
    const family = left.familyLabel.localeCompare(right.familyLabel, "tr");
    return family || left.productLabel.localeCompare(right.productLabel, "tr");
  });
}

function setImageCenterTab(tab) {
  const resolved = tab === "android" ? "android" : "iso";
  uiState.imageCenterTab = resolved;
  elements.imageCenterIsoTab?.classList.toggle("active", resolved === "iso");
  elements.imageCenterAndroidTab?.classList.toggle("active", resolved === "android");
  elements.imageCenterIsoPanel?.classList.toggle("hidden", resolved !== "iso");
  elements.imageCenterAndroidPanel?.classList.toggle("hidden", resolved !== "android");
}

function renderImageCenterIsoDownloadProgress() {
  const root = elements.imageCenterIsoDownloadProgress;
  const actionButton = elements.imageCenterStandardIsoAction;
  if (!root || !actionButton) return;
  const templateId = actionButton.dataset.templateId || "";
  const mediaId = actionButton.dataset.mediaId || "";
  const key = templateId && mediaId ? `${templateId}:${mediaId}` : "";
  const item = key ? uiState.imageCenterIsoDownloads.get(key) : null;
  if (!item?.status) {
    downloadProgressView?.hide(root);
    return;
  }
  const progress = installerMediaProgressModel(item.status, item.sample || null, item.label || mediaId || "Kurulum ISO'su");
  item.sample = progress.sample;
  downloadProgressView?.render(root, progress.model);
}

function renderImageCenterStandardIso(entries) {
  const select = elements.imageCenterStandardIsoSelect;
  const actionButton = elements.imageCenterStandardIsoAction;
  if (!select || !actionButton) return;

  const prior = uiState.imageCenterIsoSourceId;
  select.innerHTML = entries.map((entry) => {
    const source = entry.source;
    const label = source.label || `${entry.productLabel} ${entry.releaseLabel}`.trim();
    return `<option value="${escapeAttribute(source.id)}">${escapeHtml(label)}</option>`;
  }).join("");

  if (entries.length === 0) {
    uiState.imageCenterIsoSourceId = null;
    actionButton.disabled = true;
    elements.imageCenterStandardIsoLabel.textContent = "Kurulum kaynagi bulunamadi";
    elements.imageCenterStandardIsoState.textContent = "Katalog bos";
    elements.imageCenterStandardIsoNote.textContent = "Uzman Modu'nda katalog ve kaynak ayarlarini kontrol edin.";
    return;
  }

  const selected = entries.find((entry) => entry.source.id === prior) || entries[0];
  uiState.imageCenterIsoSourceId = selected.source.id;
  select.value = selected.source.id;
  const managed = installerMediaSourceManagedDownload(selected.source);
  const label = selected.source.label || `${selected.productLabel} ${selected.releaseLabel}`.trim();
  elements.imageCenterStandardIsoBadge.textContent = String(selected.familyLabel || "OS").slice(0, 2).toUpperCase();
  elements.imageCenterStandardIsoLabel.textContent = label;
  elements.imageCenterStandardIsoState.textContent = managed ? "TurkuazVM ile indirilebilir" : "Resmi indirme sayfasi";
  elements.imageCenterStandardIsoNote.textContent = managed
    ? "Secilen resmi kurulum ISO'su TurkuazVM indirme klasorune kaydedilir."
    : "Secilen sistemin resmi indirme sayfasi acilir.";
  actionButton.disabled = false;
  actionButton.textContent = managed ? "Indir" : "Resmi Sayfayi Ac";
  actionButton.dataset.imageCenterIsoAction = managed ? "download" : "official";
  actionButton.dataset.templateId = selected.template.id;
  actionButton.dataset.mediaId = selected.source.id;
  actionButton.dataset.url = selected.source.url || "";
  renderImageCenterIsoDownloadProgress();
}

async function renderImageCenterIsoCatalog() {
  if (!elements.imageCenterIsoList) return;
  if (uiState.guestCatalog.length === 0) await loadGuestCatalog();
  const entries = imageCenterIsoEntries();
  renderImageCenterStandardIso(entries);
  elements.imageCenterIsoList.innerHTML = "";
  elements.imageCenterIsoEmpty?.classList.toggle("hidden", entries.length !== 0);
  for (const entry of entries) {
    const { source, template } = entry;
    const card = document.createElement("article");
    card.className = "image-center-source-card";
    const managed = installerMediaSourceManagedDownload(source);
    const actionLabel = managed ? "Indir" : "Resmi Sayfayi Ac";
    const action = managed ? "download" : "official";
    const sourceNote = source.note || (managed ? "TurkuazVM icinden indirilebilir." : "Resmi indirme sayfasi acilir.");
    card.innerHTML = `
      <div class="image-center-source-main">
        <div class="image-center-source-icon">${escapeHtml((entry.familyLabel || "OS").slice(0, 2).toUpperCase())}</div>
        <div><strong>${escapeHtml(source.label || `${entry.productLabel} ${entry.releaseLabel}`.trim())}</strong><span>${escapeHtml(entry.familyLabel)} / ${escapeHtml(entry.productLabel)}${entry.releaseLabel ? ` / ${escapeHtml(entry.releaseLabel)}` : ""}</span></div>
      </div>
      <div class="image-center-source-meta expert-only"><span>${escapeHtml(source.provider || "Resmi kaynak")}</span><span>${escapeHtml(source.architecture || template.architecture || "-")}</span></div>
      <p class="expert-only">${escapeHtml(sourceNote)}</p>
      <button class="${managed ? "primary-button" : "ghost-button"}" type="button" data-image-center-iso-action="${action}" data-template-id="${escapeAttribute(template.id)}" data-media-id="${escapeAttribute(source.id)}" data-url="${escapeAttribute(source.url || "")}">${actionLabel}</button>`;
    elements.imageCenterIsoList.appendChild(card);
  }
}

function stopImageCenterIsoPollingIfIdle() {
  if (uiState.imageCenterIsoDownloads.size !== 0 || !uiState.imageCenterIsoPollTimer) return;
  window.clearInterval(uiState.imageCenterIsoPollTimer);
  uiState.imageCenterIsoPollTimer = null;
}

async function pollImageCenterIsoDownloads() {
  const downloads = [...uiState.imageCenterIsoDownloads.entries()];
  for (const [key, item] of downloads) {
    try {
      const status = await invoke("get_installer_media_download", { request: { guest_template_id: item.templateId, media_id: item.mediaId } });
      const state = String(status.state || "not_started");
      item.status = status;
      renderImageCenterIsoDownloadProgress();
      if (["ready", "failed", "cancelled"].includes(state)) {
        uiState.imageCenterIsoDownloads.delete(key);
        if (state === "ready") {
          recordActivity("ISO Hazir", item.label, "success", status.local_path || "Indirme tamamlandi");
          showToast("ISO hazir", item.label, "success");
        } else if (state === "failed") {
          recordActivity("ISO Indir", item.label, "error", status.detail || "Indirme basarisiz");
        } else {
          recordActivity("ISO Indir", item.label, "info", "Indirme iptal edildi");
        }
      }
    } catch (error) {
      uiState.imageCenterIsoDownloads.delete(key);
      recordActivity("ISO Indir", item.label, "error", String(error));
    }
  }
  stopImageCenterIsoPollingIfIdle();
  renderTaskDockStatus();
}

function trackImageCenterIsoDownload(status, templateId, mediaId) {
  const state = String(status?.state || "not_started");
  if (!["downloading", "verifying", "cancelling"].includes(state)) return;
  const key = `${templateId}:${mediaId}`;
  uiState.imageCenterIsoDownloads.set(key, {
    templateId,
    mediaId,
    label: elements.imageCenterStandardIsoLabel?.textContent || mediaId,
    status,
    sample: null
  });
  if (!uiState.imageCenterIsoPollTimer) {
    uiState.imageCenterIsoPollTimer = window.setInterval(() => {
      pollImageCenterIsoDownloads().catch((error) => recordActivity("ISO Izleme", mediaId, "error", String(error)));
    }, 1200);
  }
  renderTaskDockStatus();
  renderImageCenterIsoDownloadProgress();
}

async function handleImageCenterIsoAction(event) {
  const button = event.target.closest("[data-image-center-iso-action]");
  if (!button) return;
  const action = button.dataset.imageCenterIsoAction;
  setButtonBusy(button, true);
  try {
    if (action === "official") {
      if (!button.dataset.url) throw new Error("Resmi indirme adresi tanimli degil.");
      await invoke("open_external_url", { url: button.dataset.url });
      recordActivity("ISO Kaynagi Ac", button.dataset.templateId || "ISO", "success", button.dataset.url);
    } else if (action === "download") {
      const status = await invoke("start_installer_media_download", { request: { guest_template_id: button.dataset.templateId, media_id: button.dataset.mediaId } });
      recordActivity("ISO Indir", status.label || button.dataset.mediaId, "info", status.detail || status.destination_path || "Arka planda indiriliyor");
      trackImageCenterIsoDownload(status, button.dataset.templateId, button.dataset.mediaId);
      showToast("ISO indirme basladi", status.label || button.dataset.mediaId, "info");
      toggleTaskDock(true);
    }
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(button, false);
  }
}

async function openImagesCenter(tab = "iso") {
  activateNavigation("android-images");
  configureAndroidImageFlow(null);
  elements.androidImagesEyebrow.textContent = "GORUNTU MERKEZI";
  elements.androidImagesTitle.textContent = "Goruntu Merkezi";
  elements.androidImagesSubtitle.textContent = "Windows/Linux kurulum ISO'larini ve Android sistem goruntulerini tek merkezden yonetin.";
  elements.androidImagesModal.classList.remove("hidden");
  if (uiState.guestCatalog.length === 0) await loadGuestCatalog();
  setImageCenterTab(tab);
  await renderImageCenterIsoCatalog();
  if (tab === "android") await refreshAndroidImages();
}


function normalizeAndroidRelease(value) {
  const text = String(value || "").trim().toUpperCase().replace(/^ANDROID\s+/, "");
  if (["12L", "12.1", "12_1"].includes(text)) return "12L";
  const match = text.match(/^\d+/);
  return match ? match[0] : text;
}

function androidReleaseRank(releaseId) {
  const index = ANDROID_RELEASE_ORDER.indexOf(normalizeAndroidRelease(releaseId));
  return index >= 0 ? index : ANDROID_RELEASE_ORDER.length + 1;
}

function androidReleaseEntries() {
  const releases = new Map();
  for (const template of uiState.guestCatalog.filter((item) => item.guest_profile === "android")) {
    const releaseId = normalizeAndroidRelease(template.release_id || template.release_label);
    if (!releaseId) continue;
    const current = releases.get(releaseId) || { releaseId, label: template.release_label || `Android ${releaseId}`, profiles: [] };
    current.profiles.push(template.profile_label || template.profile_id || "Android");
    releases.set(releaseId, current);
  }
  return [...releases.values()].sort((left, right) => androidReleaseRank(left.releaseId) - androidReleaseRank(right.releaseId));
}

function androidImageRelease(image) {
  if (image?.requested_release) return normalizeAndroidRelease(image.requested_release);
  if (Number(image?.sdk_level) === 32) return "12L";
  return normalizeAndroidRelease(image?.android_release || "");
}

function androidSdkImageId(releaseId) {
  return `turkuaz-android-${normalizeAndroidRelease(releaseId).toLowerCase()}${ANDROID_SDK_IMAGE_SUFFIX}`;
}

function isAndroidSdkImage(image) {
  return Boolean(image?.id && String(image.id).endsWith(ANDROID_SDK_IMAGE_SUFFIX));
}

function androidImageForRelease(images, releaseId) {
  const normalizedRelease = normalizeAndroidRelease(releaseId);
  const canonicalId = androidSdkImageId(normalizedRelease);
  const canonical = images.find((image) => image.id === canonicalId) || null;
  if (canonical) return canonical;
  const sdkReady = images.find((image) => image.state === "ready" && isAndroidSdkImage(image) && androidImageRelease(image) === normalizedRelease);
  if (sdkReady) return sdkReady;
  return images.find((image) => image.state === "ready" && androidImageRelease(image) === normalizedRelease) || null;
}

function selectedAndroidVmRelease() {
  const vmId = uiState.androidImageFlowVmId || elements.androidImageQuickVm?.value || null;
  const machine = uiState.machines.find((item) => item.id === vmId);
  if (!machine) return { vmId: null, releaseId: null, machine: null };
  const template = uiState.guestCatalog.find((item) => item.id === machine.guest_template_id);
  const releaseId = template ? normalizeAndroidRelease(template.release_id || template.release_label) : null;
  return { vmId, releaseId, machine };
}

function latestAndroidReleaseId(releases = androidReleaseEntries()) {
  return releases.find((release) => release.releaseId === "17")?.releaseId || releases[0]?.releaseId || null;
}

function renderAndroidStandardDownloadProgress(image, releaseId) {
  if (!elements.androidStandardDownloadProgress) return;
  const state = String(image?.state || "");
  if (!["installing", "failed"].includes(state)) {
    downloadProgressView?.hide(elements.androidStandardDownloadProgress);
    return;
  }
  downloadProgressView?.render(elements.androidStandardDownloadProgress, androidDownloadProgressModel(image, releaseId));
}

function renderAndroidStandardRelease(releases, images, selected) {
  if (!elements.androidStandardReleaseSelect) return;
  const latestRelease = latestAndroidReleaseId(releases);
  const previous = normalizeAndroidRelease(uiState.androidStandardReleaseId);
  const preferred = selected.releaseId || (releases.some((release) => release.releaseId === previous) ? previous : latestRelease);
  uiState.androidStandardReleaseId = preferred;
  elements.androidStandardReleaseSelect.innerHTML = releases.map((release) => {
    const suffix = release.releaseId === latestRelease ? " (Guncel)" : "";
    return `<option value="${escapeAttribute(release.releaseId)}">${escapeHtml(release.label)}${suffix}</option>`;
  }).join("");
  if (preferred) elements.androidStandardReleaseSelect.value = preferred;

  const release = releases.find((item) => item.releaseId === preferred) || releases[0];
  if (!release) {
    elements.androidStandardReleaseBadge.textContent = "-";
    elements.androidStandardReleaseLabel.textContent = "Android";
    elements.androidStandardReleaseState.textContent = "Surum bulunamadi";
    elements.androidStandardReleaseAction.disabled = true;
    elements.androidStandardReleaseNote.textContent = "Android guest katalogu bos.";
    downloadProgressView?.hide(elements.androidStandardDownloadProgress);
    return;
  }

  const image = androidImageForRelease(images, release.releaseId);
  const imageReady = image?.state === "ready" && androidImageRelease(image) === release.releaseId;
  const installing = image?.state === "installing";
  const failed = image?.state === "failed";
  const vmMismatch = Boolean(selected.vmId && selected.releaseId !== release.releaseId);

  renderAndroidStandardDownloadProgress(image, release.releaseId);
  elements.androidStandardReleaseBadge.textContent = release.releaseId;
  elements.androidStandardReleaseLabel.textContent = release.label;
  elements.androidStandardReleaseAction.dataset.releaseId = release.releaseId;
  elements.androidStandardReleaseAction.dataset.imageId = image?.id || "";
  elements.androidStandardReleaseAction.dataset.androidReleaseAction = "";

  if (vmMismatch) {
    elements.androidStandardReleaseState.textContent = `VM Android ${selected.releaseId}`;
    elements.androidStandardReleaseAction.textContent = "Bu VM ile uyusmuyor";
    elements.androidStandardReleaseAction.disabled = true;
    elements.androidStandardReleaseNote.textContent = `Bu VM Android ${selected.releaseId} profiliyle olusturuldu. Farkli surumu yeni VM olustururken secin.`;
    downloadProgressView?.hide(elements.androidStandardDownloadProgress);
    return;
  }

  if (imageReady && selected.vmId) {
    elements.androidStandardReleaseState.textContent = "Image hazir";
    elements.androidStandardReleaseAction.textContent = "VM'e Ata";
    elements.androidStandardReleaseAction.dataset.androidReleaseAction = "assign";
    elements.androidStandardReleaseAction.disabled = false;
    elements.androidStandardReleaseNote.textContent = "Hazir sistem goruntusu secili VM'e atanabilir.";
  } else if (imageReady) {
    elements.androidStandardReleaseState.textContent = "Hazir";
    elements.androidStandardReleaseAction.textContent = "Hazir";
    elements.androidStandardReleaseAction.disabled = true;
    elements.androidStandardReleaseNote.textContent = "Bu Android surumu icin hazir sistem goruntusu mevcut.";
  } else if (installing) {
    elements.androidStandardReleaseState.textContent = "Indiriliyor";
    elements.androidStandardReleaseAction.textContent = "Hazirlaniyor...";
    elements.androidStandardReleaseAction.disabled = true;
    elements.androidStandardReleaseNote.textContent = "Indirme ilerlemesi asagida ve Gorevler cubugunda izlenebilir.";
  } else {
    elements.androidStandardReleaseState.textContent = failed ? "Tekrar denenebilir" : "Indirilebilir";
    elements.androidStandardReleaseAction.textContent = selected.vmId ? "Indir ve Ata" : "Indir";
    elements.androidStandardReleaseAction.dataset.androidReleaseAction = "auto";
    elements.androidStandardReleaseAction.disabled = false;
    elements.androidStandardReleaseNote.textContent = `Android ${release.releaseId} icin surume sabitlenmis resmi kaynak kullanilir.`;
  }
}

function renderAndroidReleaseCatalog(images = uiState.androidImages) {
  if (!elements.androidReleaseCatalog) return;
  const releases = androidReleaseEntries();
  const latestRelease = latestAndroidReleaseId(releases);
  const selected = selectedAndroidVmRelease();
  renderAndroidStandardRelease(releases, images, selected);
  if (elements.androidReleaseContext) {
    elements.androidReleaseContext.textContent = selected.machine
      ? `${selected.machine.name}: Android ${selected.releaseId || "?"} profili secili.`
      : "Bir Android VM secerseniz uyumlu surum otomatik vurgulanir.";
  }
  elements.androidReleaseCatalog.innerHTML = "";
  for (const release of releases) {
    const image = androidImageForRelease(images, release.releaseId);
    const isSelected = selected.releaseId === release.releaseId;
    const imageReady = image?.state === "ready" && androidImageRelease(image) === release.releaseId;
    const installing = image?.state === "installing";
    const failed = image?.state === "failed";
    const canAutoInstall = true;
    const selectedReleaseMatches = !selected.vmId || selected.releaseId === release.releaseId;
    let action = "";
    if (imageReady && selected.vmId && selectedReleaseMatches) {
      action = `<button class="primary-button" type="button" data-android-release-action="assign" data-release-id="${escapeAttribute(release.releaseId)}" data-image-id="${escapeAttribute(image.id)}">VM'e Ata</button>`;
    } else if (imageReady && selected.vmId && !selectedReleaseMatches) {
      action = `<button class="ghost-button" type="button" disabled>VM Android ${escapeHtml(selected.releaseId || "?")}</button>`;
    } else if (imageReady) {
      action = `<button class="ghost-button" type="button" disabled>Hazir</button>`;
    } else if (installing) {
      action = `<button class="primary-button" type="button" disabled>Indiriliyor...</button>`;
    } else if (canAutoInstall) {
      const autoLabel = selected.vmId && selectedReleaseMatches ? "Otomatik Indir ve Ata" : (failed ? "Tekrar Indir" : "Otomatik Indir");
      action = `<button class="primary-button" type="button" data-android-release-action="auto" data-release-id="${escapeAttribute(release.releaseId)}">${autoLabel}</button>`;
    } else {
      action = `<button class="ghost-button" type="button" disabled>Hazir image gerekli</button>`;
    }
    const stateText = imageReady ? "HAZIR" : (installing ? "INDIRILIYOR" : (failed ? "HATA" : (canAutoInstall ? "OTOMATIK KANAL" : "UYUMLULUK")));
    const note = imageReady
      ? `${image.name} / ${image.architecture}`
      : `Android ${release.releaseId} icin surume sabitlenmis resmi Android CI kanali kullanilir; SDK dogrulanmadan image READY olmaz.`;
    const card = document.createElement("article");
    card.className = `android-release-card${isSelected ? " selected" : ""}${imageReady ? " ready" : ""}`;
    card.innerHTML = `
      <div class="android-release-card-head"><div><span class="android-version-badge">${escapeHtml(release.releaseId)}</span><div><strong>${escapeHtml(release.label)}</strong><small>${escapeHtml([...new Set(release.profiles)].join(" / "))}</small></div></div><span class="android-release-state">${escapeHtml(stateText)}</span></div>
      <p>${escapeHtml(note)}</p>
      <div class="android-release-card-actions">${isSelected ? '<span class="selected-release-label">Secili VM surumu</span>' : '<span></span>'}${action}</div>`;
    elements.androidReleaseCatalog.appendChild(card);
  }
}

function handleAndroidStandardReleaseChange() {
  uiState.androidStandardReleaseId = normalizeAndroidRelease(elements.androidStandardReleaseSelect?.value);
  renderAndroidReleaseCatalog(uiState.androidImages);
}

async function handleAndroidStandardReleaseAction() {
  const button = elements.androidStandardReleaseAction;
  if (!button || button.disabled) return;
  const action = button.dataset.androidReleaseAction;
  const releaseId = normalizeAndroidRelease(button.dataset.releaseId);
  const imageId = button.dataset.imageId;
  setButtonBusy(button, true);
  try {
    if (action === "auto") {
      await ensureAndroidRelease(releaseId);
    } else if (action === "assign" && imageId) {
      const vmId = uiState.androidImageFlowVmId || elements.androidImageQuickVm?.value;
      if (!vmId) throw new Error("Android image atamak icin bir VM secin.");
      const selected = selectedAndroidVmRelease();
      const image = uiState.androidImages.find((item) => item.id === imageId);
      const imageRelease = androidImageRelease(image);
      if (!image || image.state !== "ready" || !imageRelease || selected.releaseId !== imageRelease) {
        throw new Error(`Android image surumu VM profiliyle uyusmuyor. VM: Android ${selected.releaseId || "?"}, image: Android ${imageRelease || "?"}.`);
      }
      const assignment = await invoke("assign_android_image", { request: { vm_id: vmId, image_id: imageId } });
      uiState.androidAssignments[vmId] = assignment;
      recordActivity("Android Image Ata", vmId, "success", imageId);
      if (uiState.androidImageFlowVmId === vmId) setAndroidImageFlowReady(true);
      showToast("Android image atandi", vmId, "success");
      await refreshDashboard();
      await refreshAndroidImages(false);
    }
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(button, false);
    renderAndroidReleaseCatalog(uiState.androidImages);
  }
}

async function ensureAndroidRelease(releaseId) {
  const imageId = androidSdkImageId(releaseId);
  const selected = selectedAndroidVmRelease();
  const vmId = selected.vmId && selected.releaseId === releaseId ? selected.vmId : null;
  uiState.androidAutoAssignVmId = vmId;
  uiState.androidAutoAssignImageId = vmId ? imageId : null;
  let image = uiState.androidImages.find((item) => item.id === imageId) || null;
  try {
    if (!image) {
      image = await invoke("define_android_image", { request: { image_id: imageId, name: `Turkuaz Android ${releaseId} SDK x86_64`, requested_release: releaseId } });
      recordActivity("Android Image Tanimla", imageId, "success", `Android ${releaseId}`);
    }
    if (image.state === "ready") {
      await completePendingAndroidAutoAssign([image]);
      await refreshAndroidImages();
      return;
    }
    if (image.state !== "installing") {
      await invoke("install_android_image_distribution", { request: { image_id: imageId } });
      recordActivity("Android Image Indir", imageId, "info", `Android ${releaseId} resmi SDK System Image katalogu`);
      showToast("Android image indiriliyor", `Android ${releaseId} arka planda hazirlaniyor.`, "info");
    }
    await refreshAndroidImages();
    toggleTaskDock(true);
  } catch (error) {
    uiState.androidAutoAssignVmId = null;
    uiState.androidAutoAssignImageId = null;
    recordActivity("Android Image Indir", imageId, "error", String(error));
    showError(String(error));
  }
}

async function completePendingAndroidAutoAssign(images = uiState.androidImages) {
  const vmId = uiState.androidAutoAssignVmId;
  const imageId = uiState.androidAutoAssignImageId;
  if (!vmId || !imageId) return false;
  const image = images.find((item) => item.id === imageId);
  if (!image || image.state !== "ready") return false;
  const release = androidImageRelease(image);
  const machine = uiState.machines.find((item) => item.id === vmId);
  const template = machine ? uiState.guestCatalog.find((item) => item.id === machine.guest_template_id) : null;
  const vmRelease = template ? normalizeAndroidRelease(template.release_id || template.release_label) : null;
  if (!machine || !release || vmRelease !== release) {
    uiState.androidAutoAssignVmId = null;
    uiState.androidAutoAssignImageId = null;
    recordActivity("Android Image Ata", vmId, "error", `Surum uyusmazligi: VM Android ${vmRelease || "?"} / image Android ${release || "?"}`);
    showError(`Android image surumu VM profiliyle uyusmuyor. VM: Android ${vmRelease || "?"}, image: Android ${release || "?"}.`);
    return false;
  }
  try {
    const assignment = await invoke("assign_android_image", { request: { vm_id: vmId, image_id: imageId } });
    uiState.androidAssignments[vmId] = assignment;
    uiState.androidAutoAssignVmId = null;
    uiState.androidAutoAssignImageId = null;
    recordActivity("Android Image Ata", vmId, "success", `${imageId} / Android ${release || "?"}`);
    showToast("Android hazir", `${image.name} ${vmId} VM'ine atandi.`, "success");
    if (uiState.androidImageFlowVmId === vmId) setAndroidImageFlowReady(true);
    await refreshDashboard();
    return true;
  } catch (error) {
    recordActivity("Android Image Ata", vmId, "error", String(error));
    showError(String(error));
    return false;
  }
}

async function handleAndroidReleaseAction(event) {
  const button = event.target.closest("[data-android-release-action]");
  if (!button) return;
  const releaseId = button.dataset.releaseId;
  const action = button.dataset.androidReleaseAction;
  setButtonBusy(button, true);
  try {
    if (action === "auto") {
      await ensureAndroidRelease(releaseId);
    } else if (action === "assign") {
      const vmId = uiState.androidImageFlowVmId || elements.androidImageQuickVm?.value;
      const imageId = button.dataset.imageId;
      if (!vmId) throw new Error("Android image atamak icin once durmus bir Android VM secin.");
      const selected = selectedAndroidVmRelease();
      const image = uiState.androidImages.find((item) => item.id === imageId);
      const imageRelease = androidImageRelease(image);
      if (!image || image.state !== "ready" || !imageRelease || selected.releaseId !== imageRelease) {
        throw new Error(`Android image surumu VM profiliyle uyusmuyor. VM: Android ${selected.releaseId || "?"}, image: Android ${imageRelease || "?"}.`);
      }
      const assignment = await invoke("assign_android_image", { request: { vm_id: vmId, image_id: imageId } });
      uiState.androidAssignments[vmId] = assignment;
      recordActivity("Android Image Ata", vmId, "success", imageId);
      if (uiState.androidImageFlowVmId === vmId) setAndroidImageFlowReady(true);
      showToast("Android image atandi", vmId, "success");
      await refreshDashboard();
      await refreshAndroidImages(false);
    }
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    setButtonBusy(button, false);
  }
}

async function openAndroidImagesModal(flowContext = null) {
  activateNavigation("android-images");
  configureAndroidImageFlow(flowContext);
  setImageCenterTab("android");
  elements.androidImagesModal.classList.remove("hidden");
  try {
    const machines = await invoke("list_vms");
    uiState.machines = Array.isArray(machines) ? machines : uiState.machines;
  } catch (_) {
    // Dashboard cache is sufficient when a direct VM refresh is temporarily unavailable.
  }
  renderAndroidImageQuickVmOptions();
  if (flowContext?.vmId && [...elements.androidImageQuickVm.options].some((option) => option.value === flowContext.vmId)) {
    elements.androidImageQuickVm.value = flowContext.vmId;
    elements.androidImageQuickVm.disabled = true;
  }
  await refreshAndroidImages();
}

function closeAndroidImagesModal() {
  activateNavigation("machines");
  elements.androidImagesModal.classList.add("hidden");
  elements.androidImageQuickVm.disabled = false;
  configureAndroidImageFlow(null);
  elements.androidImagePlan.textContent = "Build plani secilmedi.";
  if (uiState.androidImagePollTimer) {
    clearTimeout(uiState.androidImagePollTimer);
    uiState.androidImagePollTimer = null;
  }
}

async function handleAndroidImageFlowNext() {
  if (!uiState.androidImageFlowReady || !uiState.androidImageFlowVmId) return;
  const vmId = uiState.androidImageFlowVmId;
  closeAndroidImagesModal();
  showToast("Siradaki adim", "Ag yapilandirmasini tamamlayin.", "info");
  await openNetworkModal(vmId, { vmId, fromAndroid: true });
}

function scheduleAndroidImagePoll(images) {
  if (uiState.androidImagePollTimer) clearTimeout(uiState.androidImagePollTimer);
  uiState.androidImagePollTimer = null;
  if (!images.some((image) => image.state === "installing") || elements.androidImagesModal.classList.contains("hidden")) return;
  uiState.androidImagePollTimer = setTimeout(() => refreshAndroidImages(false), 3000);
}

async function refreshAndroidImages(showFailure = true) {
  try {
    const images = await invoke("list_android_images");
    uiState.androidImages = Array.isArray(images) ? images : [];
    await completePendingAndroidAutoAssign(uiState.androidImages);
    renderAndroidImages(uiState.androidImages);
    renderAndroidImageOptions(uiState.androidImages);
    renderAndroidReleaseCatalog(uiState.androidImages);
    renderTaskDockStatus();
    scheduleAndroidImagePoll(uiState.androidImages);
    return uiState.androidImages;
  } catch (error) {
    uiState.androidImages = [];
    renderAndroidImages([]);
    renderAndroidImageOptions([]);
    renderAndroidReleaseCatalog([]);
    renderTaskDockStatus();
    if (showFailure) showError(String(error));
    return [];
  }
}

function renderAndroidImages(images) {
  elements.androidImagesList.innerHTML = "";
  elements.androidImagesEmpty.classList.toggle("hidden", images.length !== 0);
  const quickVmReady = Boolean(elements.androidImageQuickVm.value);
  for (const image of images) {
    const row = document.createElement("article");
    row.className = "snapshot-row";
    const release = image.android_release || "unregistered";
    const sdk = image.sdk_level ? `API ${image.sdk_level}` : "API -";
    const artifactSummary = `${image.artifacts.length} artifact / ${release} / ${sdk}`;
    const errorSummary = image.last_error ? `<span class="error-inline">${escapeHtml(image.last_error)}</span>` : "";
    const progressSummary = renderAndroidInstallProgress(image.install_progress);
    const isInstalling = image.state === "installing";
    const isReady = image.state === "ready";
    const isFailed = image.state === "failed";
    const installDisabled = isInstalling || isReady ? "disabled" : "";
    const installLabel = isInstalling ? "Kuruluyor..." : isReady ? "Hazir" : isFailed ? "Tekrar Dene" : "Otomatik Kur";
    const cancelButton = isInstalling
      ? `<button class="vm-action stop" type="button" data-android-image-action="cancel" data-image-id="${escapeAttribute(image.id)}">Iptal</button>`
      : "";
    const cleanupButton = isFailed
      ? `<button class="ghost-button" type="button" data-android-image-action="cleanup" data-image-id="${escapeAttribute(image.id)}">Stage Temizle</button>`
      : "";
    const logButton = image.install_progress && image.install_progress.log_path
      ? `<button class="ghost-button" type="button" data-android-image-action="log" data-image-id="${escapeAttribute(image.id)}" data-log-path="${escapeAttribute(image.install_progress.log_path)}">Logu Ac</button>`
      : "";
    const assignButton = isReady
      ? `<button class="primary-button" type="button" data-android-image-action="assign" data-image-id="${escapeAttribute(image.id)}" ${quickVmReady ? "" : "disabled"}>Secili VM'e Ata</button>`
      : "";
    row.innerHTML = `
      <div>
        <strong>${escapeHtml(image.name)} / ${escapeHtml(image.state.toUpperCase())}</strong>
        <span>${escapeHtml(image.architecture)} / ${escapeHtml(image.branch)} / ${escapeHtml(image.lunch_target)}</span>
        <code>${escapeHtml(image.id)} / ${escapeHtml(artifactSummary)}</code>
        ${progressSummary}
        ${errorSummary}
      </div>
      <div class="section-actions">
        <button class="ghost-button" type="button" data-android-image-action="plan" data-image-id="${escapeAttribute(image.id)}">Build Plan</button>
        <button class="ghost-button" type="button" data-android-image-action="register" data-image-id="${escapeAttribute(image.id)}" ${isInstalling || isReady ? "disabled" : ""}>Register</button>
        <button class="primary-button" type="button" data-android-image-action="install" data-image-id="${escapeAttribute(image.id)}" ${installDisabled}>${installLabel}</button>
        ${cancelButton}
        ${cleanupButton}
        ${logButton}
        ${assignButton}
      </div>`;
    elements.androidImagesList.appendChild(row);
  }
}
function renderAndroidInstallProgress(progress) {
  if (!progress) return "";
  const stageLabel = ANDROID_INSTALL_STAGE_LABELS[progress.stage] || progress.stage.toUpperCase();
  const metrics = androidInstallProgressMetrics(progress);
  const meter = metrics.percent === null
    ? `<span>${escapeHtml(metrics.downloadedLabel)}</span>`
    : `<progress class="install-progress" max="100" value="${metrics.percent}"></progress><span>${metrics.percent}% / ${escapeHtml(metrics.downloadedLabel)} / ${escapeHtml(metrics.totalLabel)}</span>`;
  return `<div class="install-progress-detail"><strong>${escapeHtml(stageLabel)}</strong>${meter}<span>${escapeHtml(progress.detail || "")}</span><span>Hiz ${escapeHtml(metrics.speedLabel)} / ETA ${escapeHtml(metrics.etaLabel)} / Sure ${escapeHtml(metrics.elapsedLabel)}</span></div>`;
}
function formatByteCount(value) {
  const bytes = Number(value) || 0;
  const units = ["B", "KiB", "MiB", "GiB"];
  let amount = bytes;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  return `${amount.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}


function formatDuration(value) {
  const totalSeconds = Math.max(0, Math.floor(Number(value) || 0));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (hours > 0) return `${hours}s ${minutes}dk`;
  if (minutes > 0) return `${minutes}dk ${seconds}sn`;
  return `${seconds}sn`;
}

function renderAndroidImageQuickVmOptions() {
  if (!elements.androidImageQuickVm) return;
  const previous = elements.androidImageQuickVm.value;
  const candidates = uiState.machines.filter((machine) => machine.guest_profile === "android" && machine.state === STATE_STOPPED);
  elements.androidImageQuickVm.innerHTML = "";
  const empty = document.createElement("option");
  empty.value = "";
  empty.textContent = candidates.length ? "Android VM sec" : "Durmus Android VM yok";
  elements.androidImageQuickVm.appendChild(empty);
  for (const machine of candidates) {
    const option = document.createElement("option");
    option.value = machine.id;
    option.textContent = `${machine.name} / ${machine.id}`;
    elements.androidImageQuickVm.appendChild(option);
  }
  if (candidates.some((machine) => machine.id === previous)) elements.androidImageQuickVm.value = previous;
  elements.androidImageQuickVmNote.textContent = candidates.length
    ? "READY image kartindan secili VM'e tek tikla ata."
    : "Hizli atama icin once durmus bir Android VM olustur.";
}

function renderAndroidImageOptions(images) {
  const previous = elements.androidImageSelect.value;
  elements.androidImageSelect.innerHTML = "";
  const ready = images.filter((image) => image.state === "ready");
  for (const image of ready) {
    const option = document.createElement("option");
    option.value = image.id;
    option.textContent = `${image.name} / ${image.id}`;
    elements.androidImageSelect.appendChild(option);
  }
  if (ready.some((image) => image.id === previous)) elements.androidImageSelect.value = previous;
  if (uiState.androidVmId) setAndroidActionAvailability();
}

async function handleAndroidImageDefine(event) {
  event.preventDefault();
  const request = {
    image_id: document.querySelector("#android-image-id").value.trim(),
    name: document.querySelector("#android-image-name").value.trim()
  };
  try {
    await invoke("define_android_image", { request });
    await refreshAndroidImages();
    clearError();
  } catch (error) {
    showError(String(error));
  }
}

async function handleAndroidImageListAction(event) {
  const button = event.target.closest("[data-android-image-action]");
  if (!button) return;
  const request = { image_id: button.dataset.imageId };
  button.disabled = true;
  try {
    if (button.dataset.androidImageAction === "plan") {
      const plan = await invoke("prepare_android_image_build", { request });
      elements.androidImagePlan.textContent = [
        `Host: ${plan.supported_host ? "SUPPORTED" : "UNSUPPORTED"}`,
        `Branch: ${plan.branch}`,
        `Lunch: ${plan.lunch_target}`,
        `AOSP: ${plan.source_root || "NOT CONFIGURED"}`,
        `Output: ${plan.output_root}`,
        `Script: ${plan.build_script}`,
        `Minimum free disk: ${plan.minimum_free_disk_gib} GiB`,
        ...plan.notes.map((note) => `- ${note}`)
      ].join("\n");
    } else if (button.dataset.androidImageAction === "install") {
      await invoke("install_android_image_distribution", { request });
      elements.androidImagePlan.textContent = `${request.image_id} resmi Android SDK System Image katalogundan arka planda kuruluyor.`;
      await refreshAndroidImages();
    } else if (button.dataset.androidImageAction === "cancel") {
      await invoke("cancel_android_image_distribution", { request });
      elements.androidImagePlan.textContent = `${request.image_id} icin kontrollu iptal istendi.`;
      await refreshAndroidImages();
    } else if (button.dataset.androidImageAction === "cleanup") {
      if (!window.confirm(`${request.image_id} yarim kurulum staging dosyalari temizlensin mi?`)) return;
      await invoke("cleanup_android_image_distribution", { request });
      elements.androidImagePlan.textContent = `${request.image_id} staging alani temizlendi.`;
      await refreshAndroidImages();
    } else if (button.dataset.androidImageAction === "log") {
      const logPath = button.dataset.logPath || "";
      await invoke("open_android_image_install_log", { logPath });
    } else if (button.dataset.androidImageAction === "assign") {
      const vmId = elements.androidImageQuickVm.value;
      if (!vmId) throw new Error("Hizli atama icin durmus bir Android VM sec.");
      const assignment = await invoke("assign_android_image", { request: { vm_id: vmId, image_id: request.image_id } });
      uiState.androidAssignments[vmId] = assignment;
      elements.androidImagePlan.textContent = `${request.image_id} -> ${vmId} atamasi kaydedildi. Ilk boot bekleniyor.`;
      recordActivity("Android Image Ata", vmId, "success", request.image_id);
      showToast("Android image atandi", vmId, "success");
      await refreshDashboard();
      if (uiState.androidImageFlowVmId === vmId) {
        elements.androidImageQuickVm.disabled = true;
        setAndroidImageFlowReady(true);
      }
    } else {
      await invoke("register_android_image_build", { request });
      await refreshAndroidImages();
      elements.androidImagePlan.textContent = `${request.image_id} bundle checksum ile registry'ye alindi.`;
    }
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    button.disabled = false;
  }
}

async function refreshAndroidImageAssignment(showFailure = true) {
  if (!uiState.androidVmId) return null;
  await refreshAndroidImages(false);
  try {
    const assignment = await invoke("get_android_image_assignment", { vmId: uiState.androidVmId });
    elements.androidImageAssignmentStatus.textContent = assignment ? `${assignment.image_id} / ${assignment.provisioning_state.toUpperCase()} / BOOT #${assignment.boot_attempts}` : "Atanmadi";
    if (assignment && Array.from(elements.androidImageSelect.options).some((option) => option.value === assignment.image_id)) {
      elements.androidImageSelect.value = assignment.image_id;
    }
    setAndroidActionAvailability();
    return assignment;
  } catch (error) {
    elements.androidImageAssignmentStatus.textContent = "KULLANILAMIYOR";
    if (showFailure) showError(String(error));
    return null;
  }
}

async function assignSelectedAndroidImage() {
  if (!uiState.androidVmId || !elements.androidImageSelect.value) return;
  const request = { vm_id: uiState.androidVmId, image_id: elements.androidImageSelect.value };
  try {
    const assignment = await invoke("assign_android_image", { request });
    uiState.androidAssignments[uiState.androidVmId] = assignment;
    elements.androidImageAssignmentStatus.textContent = `${assignment.image_id} / ${assignment.provisioning_state.toUpperCase()} / BOOT #${assignment.boot_attempts}`;
    recordActivity("Android Image Ata", uiState.androidVmName || uiState.androidVmId, "success", assignment.image_id);
    showToast("Android image atandi", uiState.androidVmName || uiState.androidVmId, "success");
    await refreshDashboard();
    clearError();
  } catch (error) {
    showError(String(error));
  }
}

async function openAndroidModal(vmId, vmName, vmState) {
  uiState.androidVmId = vmId;
  uiState.androidVmName = vmName;
  uiState.androidVmState = vmState;
  elements.androidVmLabel.textContent = `${vmName} / ${vmId} / ${vmState}`;
  elements.androidModal.classList.remove("hidden");
  setAndroidActionAvailability();
  await refreshAndroidProfile(false);
  await refreshAndroidImageAssignment(false);
  await refreshGameCatalog(false);
  if (vmState === STATE_RUNNING) {
    await refreshAndroidStatus(false);
    await refreshGuestAgentStatus(false);
  } else {
    elements.androidGuestAgentStatus.textContent = "STOPPED";
  }
}

function closeAndroidModal() {
  elements.androidModal.classList.add("hidden");
  uiState.androidVmId = null;
  uiState.androidVmName = null;
  uiState.androidVmState = null;
  elements.androidPackageList.innerHTML = "";
  elements.gameCatalogList.innerHTML = "";
  uiState.gameCatalog = [];
}

function setAndroidActionAvailability() {
  const running = uiState.androidVmState === STATE_RUNNING;
  const stopped = uiState.androidVmState === STATE_STOPPED;
  elements.androidProfileForm.querySelector('button[type="submit"]').disabled = !stopped;
  elements.androidStatusButton.disabled = !running;
  elements.androidReadyButton.disabled = !running;
  elements.androidDisplayButton.disabled = !running;
  elements.androidApkForm.querySelector('button[type="submit"]').disabled = !running;
  elements.androidPackagesRefresh.disabled = !running;
  elements.androidTapForm.querySelector('button[type="submit"]').disabled = !running;
  elements.androidGamingInputButton.disabled = false;
  elements.gameDetectButton.disabled = !running;
  elements.androidImageAssign.disabled = !stopped || !elements.androidImageSelect.value;
}

async function refreshAndroidProfile(showFailure = true) {
  if (!uiState.androidVmId) return null;
  try {
    const profile = await invoke("get_android_profile", { vmId: uiState.androidVmId });
    elements.androidProfileStatus.textContent = `${profile.adb_endpoint} / ${profile.width}x${profile.height} @ ${profile.density_dpi}dpi / ${profile.target_fps} FPS`;
    document.querySelector("#android-width").value = profile.width;
    document.querySelector("#android-height").value = profile.height;
    document.querySelector("#android-density").value = profile.density_dpi;
    document.querySelector("#android-fps").value = profile.target_fps;
    return profile;
  } catch (error) {
    elements.androidProfileStatus.textContent = "Yapilandirilmadi";
    if (showFailure) showError(String(error));
    return null;
  }
}

async function handleAndroidConfigure(event) {
  event.preventDefault();
  if (!uiState.androidVmId) return;
  const request = {
    vm_id: uiState.androidVmId,
    width: Number(document.querySelector("#android-width").value),
    height: Number(document.querySelector("#android-height").value),
    density_dpi: Number(document.querySelector("#android-density").value),
    target_fps: Number(document.querySelector("#android-fps").value)
  };
  try {
    await invoke("configure_android_runtime", { request });
    await refreshAndroidProfile();
    await refreshDashboard();
  } catch (error) {
    showError(String(error));
  }
}

async function refreshAndroidStatus(showFailure = true) {
  if (!uiState.androidVmId) return null;
  try {
    const status = await invoke("get_android_status", { vmId: uiState.androidVmId });
    renderAndroidStatus(status);
    return status;
  } catch (error) {
    elements.androidDeviceStatus.textContent = "KULLANILAMIYOR";
    if (showFailure) showError(String(error));
    return null;
  }
}

function renderAndroidStatus(status) {
  elements.androidDeviceStatus.textContent = `${status.connection_state.toUpperCase()}${status.boot_completed ? " / BOOT OK" : ""}`;
  elements.androidAbiStatus.textContent = status.abi || "-";
  elements.androidSdkStatus.textContent = status.sdk_level ? `API ${status.sdk_level}` : "-";
}

async function refreshGuestAgentStatus(showFailure = true) {
  if (!uiState.androidVmId) return null;
  try {
    const status = await invoke("get_guest_agent_status", { vmId: uiState.androidVmId });
    const detail = status.available
      ? `${status.persistent_multi_touch ? "HAZIR" : "LIMITED"} / ${status.input_backend || "agent"}`
      : "KULLANILAMIYOR";
    elements.androidGuestAgentStatus.textContent = detail;
    return status;
  } catch (error) {
    elements.androidGuestAgentStatus.textContent = "KULLANILAMIYOR";
    if (showFailure) showError(String(error));
    return null;
  }
}

async function refreshGameCatalog(showFailure = true) {
  try {
    const games = await invoke("list_game_catalog");
    uiState.gameCatalog = Array.isArray(games) ? games : [];
    renderGameCatalog(uiState.gameCatalog, new Set());
    elements.gameCatalogResult.textContent = `${uiState.gameCatalog.length} katalog profili yuklendi.`;
    return uiState.gameCatalog;
  } catch (error) {
    uiState.gameCatalog = [];
    renderGameCatalog([], new Set());
    elements.gameCatalogResult.textContent = "Katalog yuklenemedi.";
    if (showFailure) showError(String(error));
    return [];
  }
}

async function detectInstalledGames() {
  if (!uiState.androidVmId) return;
  try {
    const detected = await invoke("detect_games", { vmId: uiState.androidVmId });
    const ids = new Set(detected.map((game) => game.id));
    if (uiState.gameCatalog.length === 0) await refreshGameCatalog(false);
    renderGameCatalog(uiState.gameCatalog, ids);
    elements.gameCatalogResult.textContent = `${detected.length} katalog oyunu kurulu olarak eslesti.`;
    clearError();
  } catch (error) {
    showError(String(error));
  }
}

function renderGameCatalog(games, detectedIds) {
  elements.gameCatalogList.innerHTML = "";
  elements.gameCatalogEmpty.classList.toggle("hidden", games.length !== 0);
  for (const game of games) {
    const row = document.createElement("article");
    row.className = "snapshot-row game-catalog-row";
    const installed = detectedIds.has(game.id);
    const preferredGpu = game.preferred_gpu_backends.length > 0 ? game.preferred_gpu_backends.join(", ") : "fallback";
    const canApply = uiState.androidVmState === STATE_STOPPED;
    row.innerHTML = `
      <div>
        <strong>${escapeHtml(game.name)}${installed ? " / INSTALLED" : ""}</strong>
        <span>${escapeHtml(game.maturity.toUpperCase())} / ${game.width}x${game.height} @ ${game.target_fps} FPS / GPU ${escapeHtml(preferredGpu.toUpperCase())}</span>
        <code>${escapeHtml(game.packages.join(" | "))}</code>
      </div>
      <div class="section-actions">
        <button class="ghost-button" type="button" data-game-action="compatibility" data-game-id="${escapeAttribute(game.id)}">Uyumluluk</button>
        <button class="primary-button" type="button" data-game-action="apply" data-game-id="${escapeAttribute(game.id)}" ${canApply ? "" : "disabled"}>Profili Uygula</button>
      </div>`;
    elements.gameCatalogList.appendChild(row);
  }
}

async function handleGameCatalogAction(event) {
  const button = event.target.closest("[data-game-action]");
  if (!button || !uiState.androidVmId) return;
  const request = { vm_id: uiState.androidVmId, game_id: button.dataset.gameId };
  button.disabled = true;
  try {
    if (button.dataset.gameAction === "compatibility") {
      const report = await invoke("get_game_compatibility", { request });
      const blockers = report.blockers.length ? ` Blocker: ${report.blockers.join("; ")}` : "";
      const warnings = report.warnings.length ? ` Warning: ${report.warnings.join("; ")}` : "";
      elements.gameCatalogResult.textContent = `${report.game.name}: ${report.status.toUpperCase()}.${blockers}${warnings}`;
    } else if (button.dataset.gameAction === "apply") {
      const applied = await invoke("apply_game_profile", { request });
      elements.gameCatalogResult.textContent = `${applied.game.name} runtime + input profili uygulandi.`;
      await refreshAndroidProfile(false);
    }
    clearError();
  } catch (error) {
    showError(String(error));
  } finally {
    button.disabled = button.dataset.gameAction === "apply" && uiState.androidVmState !== STATE_STOPPED;
  }
}

async function waitAndroidReady() {
  if (!uiState.androidVmId) return;
  elements.androidReadyButton.disabled = true;
  try {
    const status = await invoke("wait_android_ready", { vmId: uiState.androidVmId });
    renderAndroidStatus(status);
  } catch (error) {
    showError(String(error));
  } finally {
    setAndroidActionAvailability();
  }
}

async function applyAndroidDisplay() {
  if (!uiState.androidVmId) return;
  try {
    await invoke("apply_android_display", { vmId: uiState.androidVmId });
    await refreshAndroidStatus(false);
  } catch (error) {
    showError(String(error));
  }
}

async function refreshAndroidPackages() {
  if (!uiState.androidVmId) return;
  try {
    const packages = await invoke("list_android_packages", { vmId: uiState.androidVmId });
    renderAndroidPackages(packages);
  } catch (error) {
    elements.androidPackageList.innerHTML = "";
    elements.androidPackageEmpty.classList.remove("hidden");
    showError(String(error));
  }
}

function renderAndroidPackages(packages) {
  elements.androidPackageList.innerHTML = "";
  elements.androidPackageEmpty.classList.toggle("hidden", packages.length !== 0);
  for (const packageInfo of packages) {
    const row = document.createElement("article");
    row.className = "snapshot-row";
    row.innerHTML = `
      <div class="package-name"><strong>${escapeHtml(packageInfo.package_name)}</strong><span>Android package</span></div>
      <div class="snapshot-actions">
        <button class="vm-action secondary" data-package-action="launch" data-package-name="${escapeAttribute(packageInfo.package_name)}">Ac</button>
        <button class="vm-action secondary" data-package-action="stop" data-package-name="${escapeAttribute(packageInfo.package_name)}">Durdur</button>
        <button class="vm-action stop" data-package-action="uninstall" data-package-name="${escapeAttribute(packageInfo.package_name)}">Kaldir</button>
      </div>`;
    elements.androidPackageList.appendChild(row);
  }
}

async function handleAndroidApkInstall(event) {
  event.preventDefault();
  if (!uiState.androidVmId) return;
  const request = {
    vm_id: uiState.androidVmId,
    relative_apk_path: document.querySelector("#android-apk-path").value.trim()
  };
  try {
    await invoke("install_android_apk", { request });
    await refreshAndroidPackages();
  } catch (error) {
    showError(String(error));
  }
}

async function handleAndroidPackageAction(event) {
  const button = event.target.closest("[data-package-action]");
  if (!button || !uiState.androidVmId) return;
  const action = button.dataset.packageAction;
  const packageName = button.dataset.packageName;
  if (action === "uninstall" && !window.confirm(`Paket kaldirilsin mi? ${packageName}`)) return;
  const request = { vm_id: uiState.androidVmId, package_name: packageName };
  button.disabled = true;
  try {
    if (action === "launch") await invoke("launch_android_package", { request });
    if (action === "stop") await invoke("stop_android_package", { request });
    if (action === "uninstall") await invoke("uninstall_android_package", { request });
    if (action === "uninstall") await refreshAndroidPackages();
  } catch (error) {
    showError(String(error));
  } finally {
    button.disabled = false;
  }
}

async function handleAndroidTap(event) {
  event.preventDefault();
  if (!uiState.androidVmId) return;
  const request = {
    vm_id: uiState.androidVmId,
    input: {
      type: "tap",
      x: Number(document.querySelector("#android-tap-x").value),
      y: Number(document.querySelector("#android-tap-y").value)
    }
  };
  try {
    await invoke("inject_android_input", { request });
  } catch (error) {
    showError(String(error));
  }
}

async function openGamingInputModal() {
  if (!uiState.androidVmId) return;
  uiState.gamingInputVmId = uiState.androidVmId;
  uiState.gamingInputVmName = uiState.androidVmName || uiState.androidVmId;
  uiState.gamingBindings = [];
  elements.gamingInputVmLabel.textContent = `${uiState.gamingInputVmName} / ${uiState.gamingInputVmId}`;
  elements.gamingInputModal.classList.remove("hidden");
  try {
    const capabilities = await invoke("get_gaming_input_capabilities", { vmId: uiState.gamingInputVmId });
    renderGamingInputCapabilities(capabilities);
  } catch (error) {
    showError(String(error));
  }
  try {
    const profile = await invoke("get_gaming_input_profile", { vmId: uiState.gamingInputVmId });
    renderGamingInputProfile(profile);
  } catch (_) {
    renderGamingInputDefaults();
  }
}

function closeGamingInputModal() {
  elements.gamingInputModal.classList.add("hidden");
  uiState.gamingInputVmId = null;
  uiState.gamingInputVmName = null;
  uiState.gamingBindings = [];
  renderGamingBindings();
}

function renderGamingInputCapabilities(capabilities) {
  elements.gamingCapKeyboard.textContent = capabilities.raw_keyboard ? "HAZIR" : "NO";
  elements.gamingCapMouse.textContent = capabilities.raw_mouse_motion ? "HAZIR" : "NO";
  elements.gamingCapMultitouch.textContent = capabilities.persistent_multi_touch ? "HAZIR" : "GUEST AGENT";
  elements.gamingCapGamepad.textContent = capabilities.host_gamepad_observation ? "HAZIR" : "PLANNED";
}

function renderGamingInputDefaults() {
  document.querySelector("#gaming-package-name").value = "";
  document.querySelector("#gaming-joystick-enabled").checked = true;
  document.querySelector("#gaming-joystick-x").value = 1800;
  document.querySelector("#gaming-joystick-y").value = 8000;
  document.querySelector("#gaming-joystick-radius").value = 1100;
  document.querySelector("#gaming-look-enabled").checked = true;
  document.querySelector("#gaming-look-x").value = 7200;
  document.querySelector("#gaming-look-y").value = 5000;
  document.querySelector("#gaming-look-sens-x").value = 1000;
  document.querySelector("#gaming-look-sens-y").value = 1000;
  uiState.gamingBindings = [];
  renderGamingBindings();
}

function renderGamingInputProfile(profile) {
  document.querySelector("#gaming-package-name").value = profile.package_name || "";
  document.querySelector("#gaming-joystick-enabled").checked = profile.joystick.enabled;
  document.querySelector("#gaming-joystick-x").value = profile.joystick.center.x;
  document.querySelector("#gaming-joystick-y").value = profile.joystick.center.y;
  document.querySelector("#gaming-joystick-radius").value = profile.joystick.radius;
  document.querySelector("#gaming-look-enabled").checked = profile.mouse_look.enabled;
  document.querySelector("#gaming-look-x").value = profile.mouse_look.anchor.x;
  document.querySelector("#gaming-look-y").value = profile.mouse_look.anchor.y;
  document.querySelector("#gaming-look-sens-x").value = profile.mouse_look.sensitivity_x_milli;
  document.querySelector("#gaming-look-sens-y").value = profile.mouse_look.sensitivity_y_milli;
  uiState.gamingBindings = Array.isArray(profile.bindings) ? structuredClone(profile.bindings) : [];
  renderGamingBindings();
}

function handleGamingBindingAdd() {
  const sourceValue = document.querySelector("#gaming-binding-source").value;
  const [sourceType, sourceCode] = sourceValue.split(":", 2);
  const targetType = document.querySelector("#gaming-binding-target").value;
  const xValue = Number(document.querySelector("#gaming-binding-x").value);
  const yValue = Number(document.querySelector("#gaming-binding-y").value);
  const source = sourceType === "key"
    ? { type: "key", key: sourceCode }
    : sourceType === "gamepad_button"
      ? { type: "gamepad_button", button: Number(sourceCode) }
      : { type: "mouse_button", button: sourceCode };
  const target = targetType === "android_key"
    ? { type: "android_key", key_code: xValue }
    : { type: "tap", position: { x: xValue, y: yValue } };
  const duplicate = uiState.gamingBindings.some((binding) => JSON.stringify(binding.source) === JSON.stringify(source));
  if (duplicate) {
    showError("Ayni input kaynagi icin ikinci binding eklenemez.");
    return;
  }
  uiState.gamingBindings.push({ source, target });
  renderGamingBindings();
}

function renderGamingBindings() {
  elements.gamingBindingList.innerHTML = "";
  elements.gamingBindingEmpty.classList.toggle("hidden", uiState.gamingBindings.length !== 0);
  uiState.gamingBindings.forEach((binding, index) => {
    const row = document.createElement("article");
    row.className = "snapshot-row gaming-binding-row";
    const source = binding.source.type === "key"
      ? `KEY ${binding.source.key}`
      : binding.source.type === "gamepad_button"
        ? `GAMEPAD B${binding.source.button}`
        : `MOUSE ${binding.source.button}`;
    const target = binding.target.type === "android_key"
      ? `ANDROID KEY ${binding.target.key_code}`
      : binding.target.type === "tap"
        ? `TAP ${binding.target.position.x},${binding.target.position.y}`
        : `HOLD P${binding.target.pointer_id}`;
    row.innerHTML = `<div><strong>${escapeHtml(source.toUpperCase())}</strong><code>${escapeHtml(target.toUpperCase())}</code></div><button class="vm-action stop" type="button" data-gaming-binding-remove="${index}">Sil</button>`;
    elements.gamingBindingList.appendChild(row);
  });
}

function handleGamingBindingListAction(event) {
  const button = event.target.closest("[data-gaming-binding-remove]");
  if (!button) return;
  const index = Number(button.dataset.gamingBindingRemove);
  if (!Number.isInteger(index) || index < 0 || index >= uiState.gamingBindings.length) return;
  uiState.gamingBindings.splice(index, 1);
  renderGamingBindings();
}

async function handleGamingInputSave(event) {
  event.preventDefault();
  if (!uiState.gamingInputVmId) return;
  const packageValue = document.querySelector("#gaming-package-name").value.trim();
  const request = {
    profile: {
      vm_id: uiState.gamingInputVmId,
      package_name: packageValue || null,
      joystick: {
        enabled: document.querySelector("#gaming-joystick-enabled").checked,
        center: {
          x: Number(document.querySelector("#gaming-joystick-x").value),
          y: Number(document.querySelector("#gaming-joystick-y").value)
        },
        radius: Number(document.querySelector("#gaming-joystick-radius").value),
        pointer_id: JOYSTICK_POINTER_ID
      },
      mouse_look: {
        enabled: document.querySelector("#gaming-look-enabled").checked,
        anchor: {
          x: Number(document.querySelector("#gaming-look-x").value),
          y: Number(document.querySelector("#gaming-look-y").value)
        },
        sensitivity_x_milli: Number(document.querySelector("#gaming-look-sens-x").value),
        sensitivity_y_milli: Number(document.querySelector("#gaming-look-sens-y").value),
        pointer_id: MOUSE_LOOK_POINTER_ID
      },
      bindings: uiState.gamingBindings
    }
  };
  try {
    const profile = await invoke("configure_gaming_input_profile", { request });
    renderGamingInputProfile(profile);
    clearError();
  } catch (error) {
    showError(String(error));
  }
}

async function resetGamingInputState() {
  if (!uiState.gamingInputVmId) return;
  try {
    await invoke("reset_gaming_input_state", { vmId: uiState.gamingInputVmId });
    clearError();
  } catch (error) {
    showError(String(error));
  }
}

async function openLogsModal() {
  activateNavigation("logs");
  elements.logsModal.classList.remove("hidden");
  await refreshLocalLogs();
}

function closeLogsModal() {
  elements.logsModal.classList.add("hidden");
  activateNavigation("machines");
}

async function refreshLocalLogs() {
  try {
    const logs = await invoke("list_local_logs");
    elements.logsList.innerHTML = "";
    elements.logsEmpty.classList.toggle("hidden", logs.length !== 0);
    elements.logsSummary.textContent = `${logs.length} log dosyasi`;
    for (const log of logs) {
      const row = document.createElement("article");
      row.className = "snapshot-row";
      row.innerHTML = `<div><strong>${escapeHtml(log.relative_path)}</strong><span>${escapeHtml(log.category)} / ${escapeHtml(formatBytes(log.size_bytes))} / ${escapeHtml(formatTimestamp(log.modified_at_unix_ms))}</span></div><div class="snapshot-actions"><button class="ghost-button" type="button" data-log-open="${escapeAttribute(log.relative_path)}">Ac</button></div>`;
      elements.logsList.appendChild(row);
    }
  } catch (error) {
    elements.logsSummary.textContent = "Loglar yuklenemedi.";
    showError(String(error));
  }
}

async function handleLogAction(event) {
  const button = event.target.closest("[data-log-open]");
  if (!button) return;
  try {
    await invoke("open_local_log", { relativePath: button.dataset.logOpen });
  } catch (error) {
    showError(String(error));
  }
}

async function refreshDownloadSettings() {
  const settings = await invoke("get_download_settings");
  elements.downloadInstallerMediaPath.value = settings.installer_media_path || "";
  elements.downloadAndroidImagesPath.value = settings.android_images_path || "";
  elements.downloadArtifactCachePath.value = settings.artifact_cache_path || "";
  elements.downloadAndroidCiBaseUrl.value = settings.android_ci_base_url || ANDROID_CI_OFFICIAL_BASE_URL;
  elements.downloadOfficialFallback.checked = Boolean(settings.official_fallback);
  if (elements.downloadSourceSummary) {
    const official = String(settings.android_ci_base_url || "").replace(/\/$/, "") === ANDROID_CI_OFFICIAL_BASE_URL;
    elements.downloadSourceSummary.textContent = official ? "Otomatik / Google resmi kaynak" : "Otomatik / Ozel mirror";
  }
  return settings;
}

async function handleDownloadSettingsSave(event) {
  event.preventDefault();
  setButtonBusy(elements.downloadSettingsSave, true);
  try {
    const settings = await invoke("save_download_settings", { request: {
      installer_media_path: elements.downloadInstallerMediaPath.value.trim(),
      android_images_path: elements.downloadAndroidImagesPath.value.trim(),
      artifact_cache_path: elements.downloadArtifactCachePath.value.trim(),
      android_ci_base_url: elements.downloadAndroidCiBaseUrl.value.trim(),
      official_fallback: elements.downloadOfficialFallback.checked
    } });
    elements.downloadSettingsNote.textContent = "Ayarlar kaydedildi. Yeni yol ve kaynaklar Engine yeniden baslatildiginda uygulanir.";
    if (elements.downloadSourceSummary) {
      const official = String(settings.android_ci_base_url || "").replace(/\/$/, "") === ANDROID_CI_OFFICIAL_BASE_URL;
      elements.downloadSourceSummary.textContent = official ? "Otomatik / Google resmi kaynak" : "Otomatik / Ozel mirror";
    }
    recordActivity("Indirme Ayarlari", "Ayarlar", "success", "Yol ve kaynaklar kaydedildi");
    showToast("Ayarlar kaydedildi", "Yeni indirme ayarlari sonraki Engine baslangicinda uygulanir.", "success");
    clearError();
  } catch (error) {
    elements.downloadSettingsNote.textContent = "Ayarlar kaydedilemedi.";
    showError(String(error));
  } finally {
    setButtonBusy(elements.downloadSettingsSave, false);
  }
}

async function openGpuModal() {
  activateNavigation("gaming");
  elements.gpuModal.classList.remove("hidden");
  try {
    await refreshDownloadSettings();
    if (uiState.expertMode) {
      const gpu = await invoke("get_gpu_capabilities");
      renderGpuCapabilities(gpu);
    }
    clearError();
  } catch (error) {
    showError(String(error));
  }
}

function closeGpuModal() { elements.gpuModal.classList.add("hidden"); activateNavigation("machines"); }

function renderGpuCapabilities(gpu) {
  elements.gpuRequested.textContent = gpu.requested_mode.toUpperCase();
  elements.gpuEffective.textContent = gpu.effective_backend.toUpperCase();
  elements.gpuHostmem.textContent = `${gpu.hostmem_mib} MiB`;
  elements.gpuVulkan.textContent = gpu.vulkan_loader_available ? "HAZIR" : "KULLANILAMIYOR";
  elements.gpuVirtio.textContent = gpu.qemu_virtio_2d ? "YES" : "NO";
  elements.gpuVirgl.textContent = gpu.qemu_virgl ? "YES" : "NO";
  elements.gpuVenus.textContent = gpu.qemu_venus ? "YES" : "NO";
  elements.gpuRutabaga.textContent = gpu.qemu_rutabaga ? "YES" : "NO";
  elements.gpuGfxstream.textContent = gpu.qemu_gfxstream_vulkan ? "YES" : "NO";
  elements.gpuAndroidGfxstream.textContent = gpu.android_gfxstream_experimental ? "EXPERIMENTAL" : "NO";
  elements.gpuVulkanSummary.textContent = gpu.vulkan_summary || (gpu.vulkan_probe_available ? "vulkaninfo hazir, ozet alinmadi." : "vulkaninfo bulunamadi.");
  elements.gpuWarning.textContent = gpu.resolution_error || (gpu.experimental ? "Experimental GPU backend aktif." : "");
  elements.gpuWarning.classList.toggle("hidden", !elements.gpuWarning.textContent);
}

async function openCreateModal() {
  elements.createConfigPanel.classList.remove("hidden");
  elements.createCompletePanel.classList.add("hidden");
  uiState.createdVmId = null;
  uiState.createdVmRecommendedDiskGib = null;
  uiState.createdVmSourceKind = null;
  uiState.vmIdentityTouched = false;
  uiState.vmIdTouched = false;
  uiState.createDiskTouched = false;
  uiState.createMediaTemplateId = null;
  uiState.createMediaSources = [];
  uiState.createMediaSource = null;
  uiState.createMediaSelectedMediaId = null;
  uiState.createMediaLocalPath = null;
  uiState.createMediaReady = false;
  uiState.createAndroidImageId = null;
  uiState.createAndroidImageDeferred = false;
  stopCreateMediaPolling();
  stopCreateAndroidImagePolling();
  if (elements.createMediaLocalPath) elements.createMediaLocalPath.value = "";
  if (elements.createNetworkId) { elements.createNetworkId.value = ""; elements.createNetworkId.dataset.auto = "true"; }
  if (elements.createNextNetwork) elements.createNextNetwork.textContent = "Turkuaz NAT";
  elements.createModal.classList.remove("hidden");
  showCreateStep("catalog");
  try {
    await loadGuestCatalog();
    clearError();
  } catch (error) {
    elements.guestTemplateInfo.textContent = `Guest katalogu yuklenemedi: ${String(error)}`;
    elements.createSubmit.disabled = true;
    showError(String(error));
  }
}
function closeCreateModal() {
  stopCreateMediaPolling();
  stopCreateAndroidImagePolling();
  elements.createModal.classList.add("hidden");
  elements.createConfigPanel.classList.remove("hidden");
  elements.createCompletePanel.classList.add("hidden");
}
function showError(message) {
  elements.errorBanner.textContent = message;
  elements.errorBanner.classList.remove("hidden");
  showToast("Islem hatasi", String(message), "error");
}
function clearError() { elements.errorBanner.textContent = ""; elements.errorBanner.classList.add("hidden"); }
function formatMemory(mib) { return mib >= 1024 ? `${(mib / 1024).toFixed(mib % 1024 === 0 ? 0 : 1)} GiB` : `${mib} MiB`; }
function formatTimestamp(unixMs) { return new Date(unixMs).toLocaleString("tr-TR"); }
function escapeHtml(value) { const node = document.createElement("div"); node.textContent = String(value); return node.innerHTML; }
function escapeAttribute(value) { return escapeHtml(value).replaceAll('"', "&quot;"); }

function closeOnBackdrop(event, modal, close) {
  if (event.target === modal) close();
}

elements.androidGamingInputButton.addEventListener("click", openGamingInputModal);
elements.gameCatalogRefresh.addEventListener("click", () => refreshGameCatalog());
elements.gameDetectButton.addEventListener("click", detectInstalledGames);
elements.gameCatalogList.addEventListener("click", handleGameCatalogAction);
elements.closeGamingInputModal.addEventListener("click", closeGamingInputModal);
elements.gamingInputModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.gamingInputModal, closeGamingInputModal));
elements.gamingInputProfileForm.addEventListener("submit", handleGamingInputSave);
elements.gamingBindingAdd.addEventListener("click", handleGamingBindingAdd);
elements.gamingBindingList.addEventListener("click", handleGamingBindingListAction);
elements.gamingInputReset.addEventListener("click", resetGamingInputState);

elements.homeButton?.addEventListener("click", showHomeNavigation);
elements.machinesButton.addEventListener("click", showMachinesNavigation);
elements.sidebarCollapseButton.addEventListener("click", toggleSidebar);
elements.topbarSidebarToggle.addEventListener("click", toggleSidebar);
elements.shortcutsButton.addEventListener("click", openShortcutsModal);
elements.expertModeButton?.addEventListener("click", toggleExpertMode);
elements.closeShortcutsModal.addEventListener("click", closeShortcutsModal);
elements.shortcutsModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.shortcutsModal, closeShortcutsModal));
elements.vmSearchInput.addEventListener("input", handleVmSearchInput);
elements.clearVmFilterButton.addEventListener("click", clearVmFilters);
elements.treeMachineList.addEventListener("click", handleTreeMachineClick);
elements.vmFilterGroup.addEventListener("click", (event) => {
  const button = event.target.closest("[data-vm-filter]");
  if (button) setVmQuickFilter(button.dataset.vmFilter);
});
elements.vmSortSelect.addEventListener("change", (event) => setVmSortMode(event.target.value));
elements.cardViewButton.addEventListener("click", () => setVmViewMode(VM_VIEW_CARD));
elements.compactViewButton.addEventListener("click", () => setVmViewMode(VM_VIEW_COMPACT));
elements.clearActivityButton.addEventListener("click", clearActivity);
elements.taskDockToggle?.addEventListener("click", () => toggleTaskDock());
elements.bulkStartButton.addEventListener("click", () => handleBulkVmAction("start"));
elements.bulkStopButton.addEventListener("click", () => handleBulkVmAction("stop"));
document.addEventListener("keydown", (event) => {
  const control = event.ctrlKey || event.metaKey;
  const key = event.key.toLowerCase();
  if (control && key === "k") {
    event.preventDefault();
    elements.vmSearchInput.focus();
    elements.vmSearchInput.select();
    return;
  }
  if (control && key === "n") {
    event.preventDefault();
    openCreateModal();
    return;
  }
  if (event.key === "F5") {
    event.preventDefault();
    refreshDashboard().then((ok) => { if (ok) showToast("Dashboard yenilendi", "Host ve VM durumu guncellendi.", "success"); });
    return;
  }
  if (control && event.shiftKey && key === "x") {
    event.preventDefault();
    clearVmFilters();
    showToast("Filtreler temizlendi", "Tum sanal makineler goruntuleniyor.", "info");
    return;
  }
  if (event.key === "Escape") {
    closeTopModal();
  }
});

elements.storageButton.addEventListener("click", () => openStorageModal());
elements.closeStorageModal.addEventListener("click", closeStorageModal);
elements.storageModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.storageModal, closeStorageModal));
elements.storageDiskForm.addEventListener("submit", handleStorageDiskCreate);
elements.storageVm.addEventListener("change", () => { renderStorageDisks(); refreshStorageDiskSuggestion(); });
elements.storageDiskList.addEventListener("click", handleStorageDiskAction);
elements.storageNextButton.addEventListener("click", handleStorageFlowNext);
elements.artifactCacheButton.addEventListener("click", openArtifactCacheModal);
elements.closeArtifactCacheModal.addEventListener("click", closeArtifactCacheModal);
elements.artifactCacheModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.artifactCacheModal, closeArtifactCacheModal));
elements.artifactCacheRefresh.addEventListener("click", () => refreshArtifactCache());
elements.artifactCacheVerify.addEventListener("click", verifyArtifactCache);
elements.artifactCacheRevalidateAll.addEventListener("click", revalidateAllArtifactCache);
elements.artifactCacheCleanup.addEventListener("click", cleanupArtifactCache);
elements.artifactCacheList.addEventListener("click", handleArtifactCacheEntryAction);
elements.artifactCacheFetchForm.addEventListener("submit", fetchMutableArtifactCache);
elements.networkButton.addEventListener("click", () => openNetworkModal());
elements.networkProfile.addEventListener("change", updateNetworkProfileFields);
elements.networkServicePreset.addEventListener("change", applyNetworkServicePreset);
elements.networkServiceForm.addEventListener("submit", handleNetworkServicePublish);
elements.closeConnectionModal.addEventListener("click", closeConnectionModal);
elements.connectionModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.connectionModal, closeConnectionModal));
elements.connectionUser.addEventListener("input", updateConnectionCommands);
elements.connectionStartVm.addEventListener("click", startConnectionVm);
elements.prepareSshAccess.addEventListener("click", () => prepareConnectionAccess("ssh"));
elements.prepareRdpAccess.addEventListener("click", () => prepareConnectionAccess("rdp"));
elements.testSshConnection.addEventListener("click", () => testConnection("ssh"));
elements.testRdpConnection.addEventListener("click", () => testConnection("rdp"));
elements.openSshConnection.addEventListener("click", openSshConnection);
elements.openRdpConnection.addEventListener("click", openRdpConnection);
elements.copySshCommand.addEventListener("click", () => copyConnectionCommand(elements.connectionSshCommand));
elements.copyRdpCommand.addEventListener("click", () => copyConnectionCommand(elements.connectionRdpCommand));
elements.closeNetworkModal.addEventListener("click", closeNetworkModal);
elements.networkModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.networkModal, closeNetworkModal));
elements.networkAttachForm.addEventListener("submit", handleNetworkAttach);
elements.networkVm.addEventListener("change", () => { renderNetworkAttachments(); refreshNetworkIdSuggestion(); });
elements.networkAttachmentList.addEventListener("click", handleNetworkAttachmentAction);
elements.networkNextButton.addEventListener("click", handleNetworkFlowNext);

elements.androidImagesButton.addEventListener("click", () => openImagesCenter("iso"));
elements.closeAndroidImagesModal.addEventListener("click", closeAndroidImagesModal);
elements.androidImagesModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.androidImagesModal, closeAndroidImagesModal));
elements.imageCenterTabs?.addEventListener("click", async (event) => {
  const button = event.target.closest("[data-image-center-tab]");
  if (!button) return;
  const tab = button.dataset.imageCenterTab === "android" ? "android" : "iso";
  setImageCenterTab(tab);
  if (tab === "iso") await renderImageCenterIsoCatalog();
  else await refreshAndroidImages();
});
elements.imageCenterStandardIsoSelect?.addEventListener("change", () => {
  uiState.imageCenterIsoSourceId = elements.imageCenterStandardIsoSelect.value;
  renderImageCenterIsoCatalog().catch((error) => showError(String(error)));
});
elements.imageCenterStandardIsoAction?.addEventListener("click", handleImageCenterIsoAction);
elements.imageCenterIsoList?.addEventListener("click", handleImageCenterIsoAction);
elements.imageCenterIsoRefresh?.addEventListener("click", renderImageCenterIsoCatalog);
elements.androidImageDefineForm.addEventListener("submit", handleAndroidImageDefine);
elements.androidImagesRefresh.addEventListener("click", () => refreshAndroidImages());
elements.androidReleaseRefresh?.addEventListener("click", () => refreshAndroidImages());
elements.androidReleaseCatalog?.addEventListener("click", handleAndroidReleaseAction);
elements.androidStandardReleaseSelect?.addEventListener("change", handleAndroidStandardReleaseChange);
elements.androidStandardReleaseAction?.addEventListener("click", handleAndroidStandardReleaseAction);
elements.androidImagesList.addEventListener("click", handleAndroidImageListAction);
elements.androidImageQuickVm.addEventListener("change", () => { renderAndroidImages(uiState.androidImages); renderAndroidReleaseCatalog(uiState.androidImages); });
elements.androidImageNextButton.addEventListener("click", handleAndroidImageFlowNext);
elements.androidImageAssignmentRefresh.addEventListener("click", () => refreshAndroidImageAssignment());
elements.androidImageSelect.addEventListener("change", setAndroidActionAvailability);
elements.androidImageAssign.addEventListener("click", assignSelectedAndroidImage);

elements.logsButton.addEventListener("click", openLogsModal);
elements.closeLogsModal.addEventListener("click", closeLogsModal);
elements.logsRefreshButton.addEventListener("click", refreshLocalLogs);
elements.logsList.addEventListener("click", handleLogAction);
elements.logsModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.logsModal, closeLogsModal));

elements.gamingButton.addEventListener("click", openGpuModal);
elements.downloadSettingsForm?.addEventListener("submit", handleDownloadSettingsSave);
elements.closeGpuModal.addEventListener("click", closeGpuModal);
elements.gpuModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.gpuModal, closeGpuModal));
elements.homeOverview?.addEventListener("click", async (event) => {
  const actionButton = event.target.closest("[data-home-action]");
  if (actionButton?.dataset.homeAction === "images") { await openImagesCenter("iso"); return; }
  if (actionButton?.dataset.homeAction === "create") { openCreateModal(); return; }
  if (actionButton?.dataset.homeAction === "machines") { activateNavigation("machines"); renderMachineViews(); return; }
  const vmButton = event.target.closest("[data-home-vm-id]");
  if (vmButton) {
    uiState.selectedVmId = vmButton.dataset.homeVmId;
    uiState.selectedVmTab = "overview";
    activateNavigation("machines");
    renderMachineViews();
  }
});
elements.machineGrid.addEventListener("click", handleVmAction);
elements.vmDetailPanel?.addEventListener("click", (event) => {
  const tab = event.target.closest("[data-detail-tab]");
  if (tab) { uiState.selectedVmTab = tab.dataset.detailTab || "overview"; renderVmDetail(); return; }
  void handleVmAction(event);
});
document.querySelectorAll("[data-empty-create]").forEach((button) => button.addEventListener("click", openCreateModal));
document.querySelector("[data-empty-images]")?.addEventListener("click", () => openImagesCenter("iso"));
elements.refreshButton.addEventListener("click", async () => {
  setButtonBusy(elements.refreshButton, true);
  try {
    const ok = await refreshDashboard();
    if (ok) showToast("Dashboard yenilendi", "Host ve VM durumu guncellendi.", "success");
  } finally {
    setButtonBusy(elements.refreshButton, false);
  }
});
elements.hostSelector.addEventListener("change", handleHostChange);
elements.createDiskSize?.addEventListener("input", () => { uiState.createDiskTouched = true; });
elements.vmId?.addEventListener("input", syncCreatePlannedResourceIds);
wireCreateFlowNavigation();
elements.newVmButton.addEventListener("click", openCreateModal);
elements.closeModal.addEventListener("click", closeCreateModal);
elements.cancelCreate.addEventListener("click", closeCreateModal);
elements.createForm.addEventListener("submit", handleCreate);
elements.guestFamilySelect.addEventListener("change", () => {
  uiState.selectedGuestFamily = elements.guestFamilySelect.value;
  renderGuestCatalogSelectors();
});
elements.guestProductSelect.addEventListener("change", () => renderGuestCatalogSelectors({ product: elements.guestProductSelect.value }));
elements.guestReleaseSelect.addEventListener("change", () => {
  const product = elements.guestProductSelect.value;
  const release = elements.guestReleaseSelect.value;
  renderGuestCatalogSelectors({ product, release });
});
elements.guestProfileSelect.addEventListener("change", applyGuestTemplateSelection);
elements.vmName.addEventListener("input", () => {
  uiState.vmIdentityTouched = true;
  if (!uiState.vmIdTouched) elements.vmId.value = suggestUniqueVmId(elements.vmName.value);
});
elements.vmId.addEventListener("input", () => { uiState.vmIdentityTouched = true; uiState.vmIdTouched = true; });
elements.vmCpu.addEventListener("input", updateCreateSummary);
elements.vmMemory.addEventListener("input", updateCreateSummary);
elements.createAddDisk.addEventListener("click", handlePostCreateDisk);
elements.createFinish.addEventListener("click", closeCreateModal);
elements.createModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.createModal, closeCreateModal));
elements.closeInstallerMediaModal.addEventListener("click", closeInstallerMediaModal);
elements.installerMediaModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.installerMediaModal, closeInstallerMediaModal));
elements.installerMediaPickButton.addEventListener("click", pickInstallerMedia);
elements.installerMediaAttachButton.addEventListener("click", attachInstallerMedia);
elements.installerMediaEjectButton.addEventListener("click", ejectInstallerMedia);
elements.installerMediaDownloadButton.addEventListener("click", startInstallerMediaDownload);
elements.installerMediaCancelDownloadButton.addEventListener("click", cancelInstallerMediaDownload);
elements.installerMediaAttachDownloadedButton.addEventListener("click", attachDownloadedInstallerMedia);
elements.installerMediaOfficialPageButton.addEventListener("click", openInstallerMediaOfficialPage);
elements.installerMediaOptionsToggle.addEventListener("click", toggleInstallerMediaOptions);
elements.installerMediaSourceSelect.addEventListener("change", handleInstallerMediaSourceChange);
elements.installerMediaNextButton.addEventListener("click", handleInstallerMediaNext);
elements.closeConfigurationCompleteModal.addEventListener("click", closeConfigurationCompleteModal);
elements.configurationCompleteDone.addEventListener("click", closeConfigurationCompleteModal);
elements.configurationCompleteModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.configurationCompleteModal, closeConfigurationCompleteModal));
elements.closeVmEditModal.addEventListener("click", closeVmEditModal);
elements.vmEditModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.vmEditModal, closeVmEditModal));
elements.vmEditForm.addEventListener("submit", handleVmEdit);
elements.closeSnapshotModal.addEventListener("click", closeSnapshotModal);
elements.snapshotForm.addEventListener("submit", handleSnapshotCreate);
elements.snapshotList.addEventListener("click", handleSnapshotAction);
elements.snapshotModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.snapshotModal, closeSnapshotModal));
elements.closeCloneModal.addEventListener("click", closeCloneModal);
elements.cancelClone.addEventListener("click", closeCloneModal);
elements.cloneForm.addEventListener("submit", handleClone);
elements.cloneModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.cloneModal, closeCloneModal));

elements.closeAndroidModal.addEventListener("click", closeAndroidModal);
elements.androidModal.addEventListener("click", (event) => closeOnBackdrop(event, elements.androidModal, closeAndroidModal));
elements.androidProfileForm.addEventListener("submit", handleAndroidConfigure);
elements.androidStatusButton.addEventListener("click", () => refreshAndroidStatus());
elements.androidReadyButton.addEventListener("click", waitAndroidReady);
elements.androidDisplayButton.addEventListener("click", applyAndroidDisplay);
elements.androidApkForm.addEventListener("submit", handleAndroidApkInstall);
elements.androidPackagesRefresh.addEventListener("click", refreshAndroidPackages);
elements.androidPackageList.addEventListener("click", handleAndroidPackageAction);
elements.androidTapForm.addEventListener("submit", handleAndroidTap);

listen(EVENT_RUNTIME_UPDATE, (event) => {
  renderDashboard(event.payload);
  clearError();
});

for (const modal of document.querySelectorAll(".modal-backdrop")) {
  new MutationObserver(syncModalOpenState).observe(modal, { attributes: true, attributeFilter: ["class"] });
}
restoreUiPreferences();
renderActivity();
syncModalOpenState();
activateNavigation("home");
loadHosts().then(refreshDashboard);
