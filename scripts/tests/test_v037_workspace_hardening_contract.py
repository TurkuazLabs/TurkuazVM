# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v037_workspace_hardening_contract.py
# 📌 Amac: v0.37.0 Workspace UX, release toolchain ve stale runtime header temizligini fail-closed korur
# 📌 Modul - Python
# Version: 0.37.1
# Aciklama: Home/inline VM Workspace, modular View, non-admin launcher, Rust pin ve Artifact Cache SSRF hardening regresyonlarini dogrular
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(path: str, *tokens: str) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{path}: missing tokens: {missing}")


def workspace_version() -> str:
    match = re.search(r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', read("Cargo.toml"))
    if not match:
        raise AssertionError("workspace version missing")
    return match.group(1)


def main() -> int:
    version = workspace_version()
    version_parts = [int(part) for part in version.split(".")[:2]]
    assert version_parts[0] == 0 and version_parts[1] >= 37, version

    require(
        "apps/desktop/ui/index.html",
        'id="home-button"',
        'id="home-page"',
        'id="home-overview"',
        './views/workspace.css',
        './views/vm_workspace_view.js',
    )
    index = read("apps/desktop/ui/index.html")
    assert "window.__TAURI__=" not in index, "preview mock leaked into release HTML"
    assert index.index("./views/vm_workspace_view.js") < index.index("./app.js")
    assert 'id="network-button" class="nav-item expert-only"' in index
    assert 'id="storage-button" class="nav-item expert-only"' in index

    require(
        "apps/desktop/ui/app.js",
        'activeNavigation: "home"',
        "renderHomeOverview()",
        "window.TurkuazVmWorkspaceView.renderVmDetail",
        "renderInlineSnapshots",
        "renderInlineLogs",
    )
    require(
        "apps/desktop/ui/views/vm_workspace_view.js",
        "function renderHome(context)",
        "function renderVmDetail(machine, context)",
        'data-action="add-disk"',
        'data-action="network"',
        'data-action="connect"',
        'id="inline-snapshot-content"',
        'id="inline-log-content"',
    )
    require(
        "apps/desktop/ui/views/workspace.css",
        ".home-hero",
        ".home-metric-grid",
        ".workspace-resource-row",
        ".inline-timeline-row",
    )
    require(
        "scripts/verify_structure.ps1",
        "$DesktopWorkspaceViewContent",
        "$DesktopCombinedScriptContent",
        "V037_WORKSPACE_MODULE_MISSING",
    )
    app = read("apps/desktop/ui/app.js")
    for stale_function in ("getVmFilterLabel", "androidAssignmentLabel", "androidReleaseNumber"):
        assert stale_function not in app, f"dead desktop function returned: {stale_function}"

    launcher = read("scripts/start_turkuazvm.ps1")
    assert 'Start-Process -FilePath "powershell.exe"' not in launcher or "-Verb RunAs" not in launcher
    assert "Test-TurkuazManagedNetworkRuntime" not in launcher
    assert "$RequiredRustToolchain = $RustToolchainPolicy.Channel" in launcher
    require("rust-toolchain.toml", 'channel = "1.98.0"')
    require("Cargo.toml", f'version = "{version}"', 'rust-version = "1.98.0"')

    stale_runtime_tokens = {
        "repositories/recovery_repository.py": "# Version: 0.14.1\\n",
        "repositories/runtime_validation_repository.py": "# Version: 0.15.0\\n",
        "crates/repositories/src/repositories/yaml_artifact_cache_repository.rs": "# Version: 0.21.0\\n",
        "crates/repositories/src/repositories/yaml_runtime_recovery_repository.rs": "# Version: 0.18.0\\n",
        "crates/repositories/src/repositories/yaml_vm_repository.rs": "# Version: 0.18.0\\n",
        "crates/repositories/src/repositories/yaml_android_profile_repository.rs": "# Version: 0.12.4\\n",
        "crates/repositories/src/repositories/yaml_game_input_profile_repository.rs": "# Version: 0.12.4\\n",
        "crates/platform/src/tools/native_network_tool.rs": "# Version: 0.32.0\\n",
        "apps/desktop/src-tauri/src/tools/download_settings_tool.rs": "# Version: 0.36.0\\n",
    }
    for path, token in stale_runtime_tokens.items():
        assert token not in read(path), f"stale runtime version generator: {path}"
    require("tools/project_version_tool.py", "def project_version(")

    require(
        "crates/guest/src/tools/artifact_cache_http_policy_tool.rs",
        "resolved_addresses: Vec<IpAddr>",
        "to_ipv4_mapped()",
        "artifact source resolves to a non-public address",
    )
    require(
        "crates/guest/src/tools/artifact_cache_http_fetch_tool.rs",
        "validated_url.resolved_addresses",
        '.arg("--resolve")',
    )

    print("V037_WORKSPACE_HARDENING_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
