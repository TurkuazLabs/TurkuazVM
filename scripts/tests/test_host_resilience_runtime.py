# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_host_resilience_runtime.py
# 📌 Amac: K runtime wiring icin image, QMP, network, rebind, VirtioFS ve TVGB fail-closed regression testlerini calistirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Gercek VM gerektirmeyen adaptor ve orchestration testleriyle kritik runtime invariants davranisini dogrular
# Bagimli Oldugu Katman: Service | Repo | Tool | View

from __future__ import annotations

import json
import socket
import sys
import tempfile
import threading
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from repositories.recovery_repository import RecoveryRepository, RecoveryRepositoryError
from services.host_resilience_service import HostResilienceService, HostResilienceServiceError
from tools.command_runner import CommandResult
from tools.guest_network_tool import GuestNetworkError
from tools.network_recovery_tool import NetworkRecoveryError, NetworkRecoveryTool, NicRuntimeState
from tools.qemu_img_tool import QemuImgError, QemuImgTool
from tools.qmp_client import QmpClient
from tools.runtime_config import load_runtime_config
from tools.tvgb_transfer_tool import TvgbTransferError, TvgbTransferTool
from tools.virtiofs_tool import VirtioFsError, VirtioFsSession, VirtioFsTool


class FakeQemuRunner:
    def __init__(self) -> None:
        self.calls: list[tuple[str, ...]] = []

    def run(self, argv: Any, **kwargs: Any) -> CommandResult:
        args = tuple(str(item) for item in argv)
        self.calls.append(args)
        command = args[1]
        if command == "info":
            return CommandResult(args, 0, json.dumps({"format": "qcow2"}), "")
        if command == "check":
            return CommandResult(args, 0, json.dumps({"corruptions": 0}), "")
        if command == "create":
            Path(args[-1]).write_bytes(b"K-OVERLAY")
            return CommandResult(args, 0, "", "")
        raise AssertionError(f"unexpected qemu-img command: {args}")


class FakeTvgbRunner:
    def __init__(self, required: set[str]) -> None:
        self.required = required
        self.calls: list[tuple[str, ...]] = []

    def run(self, argv: Any, **kwargs: Any) -> CommandResult:
        args = tuple(str(item) for item in argv)
        self.calls.append(args)
        if "capabilities" in args:
            return CommandResult(args, 0, json.dumps({"capabilities": sorted(self.required)}), "")
        sha_index = args.index("--sha256") + 1
        return CommandResult(args, 0, json.dumps({"status": "PASS", "sha256": args[sha_index]}), "")


class FakeQmp:
    log: list[tuple[str, dict[str, Any]]] = []
    events: list[tuple[str, str | None]] = []

    def __init__(self, config: Any, endpoint: str) -> None:
        self.config = config
        self.endpoint = endpoint

    def __enter__(self) -> "FakeQmp":
        return self

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        return None

    def execute(self, command: str, arguments: dict[str, Any]) -> Any:
        self.log.append((command, dict(arguments)))
        if command == self.config.runtime["qmp"]["schema_command"]:
            return [
                {
                    "name": self.config.runtime["qmp"]["net_client_driver_enum"],
                    "meta-type": "enum",
                    "members": [{"name": "user"}, {"name": "passt"}],
                }
            ]
        return {}

    def wait_for_event(self, event_name: str, *, device_id: str | None = None) -> dict[str, Any]:
        self.events.append((event_name, device_id))
        return {"event": event_name, "data": {"device": device_id}}


class FakeHostProbeResult:
    def __init__(self, payload: dict[str, Any]) -> None:
        self.payload = payload

    def as_dict(self) -> dict[str, Any]:
        return dict(self.payload)


class FakeHostCapability:
    def __init__(self, payload: dict[str, Any]) -> None:
        self.payload = payload

    def probe(self) -> FakeHostProbeResult:
        return FakeHostProbeResult(self.payload)


class FakeGuestNetwork:
    def __init__(self, verify_results: list[bool], soft_renew_ok: bool = False) -> None:
        self.verify_results = list(verify_results)
        self.soft_renew_ok = soft_renew_ok
        self.calls: list[str] = []

    def soft_renew(self) -> dict[str, Any]:
        self.calls.append("soft_renew")
        if not self.soft_renew_ok:
            raise GuestNetworkError("soft renew failed")
        return {"status": "PASS"}

    def verify(self) -> dict[str, Any]:
        self.calls.append("verify")
        value = self.verify_results.pop(0) if self.verify_results else False
        if not value:
            raise GuestNetworkError("verify failed")
        return {"status": "PASS"}


class FakeNetworkRuntime:
    def __init__(self) -> None:
        self.calls: list[str] = []

    def reconnect_link(self, endpoint: str, state: NicRuntimeState) -> dict[str, Any]:
        self.calls.append("reconnect")
        return {"status": "RECONNECTED", "mac": state.mac}

    def recreate_backend(
        self,
        endpoint: str,
        state: NicRuntimeState,
        *,
        backend_type: str | None = None,
        backend_arguments: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        self.calls.append(f"recreate:{backend_type or state.backend_type}")
        return {"backend_type": backend_type or state.backend_type, "mac": state.mac}

    def rescue_backend_for_platform(self, endpoint: str, platform_name: str, capabilities: dict[str, bool]) -> str:
        self.calls.append("rescue_select")
        return "user"


class FakeQemuImgService:
    def __init__(self, source: Path) -> None:
        self.source = source

    def check_read_only(self, source: Path) -> dict[str, Any]:
        assert source == self.source
        return {"exit_code": 0, "source_sha256": "source-hash"}

    def create_rescue_overlay(self, source: Path, destination: Path) -> dict[str, Any]:
        assert source == self.source
        destination.write_bytes(b"rescue")
        return {
            "source": str(source.resolve()),
            "source_format": "qcow2",
            "source_sha256": "source-hash",
            "overlay": str(destination.resolve()),
            "overlay_format": "qcow2",
        }


def expect_error(callable_obj: Any, expected_code: str, error_type: type[BaseException]) -> None:
    try:
        callable_obj()
    except error_type as exc:
        assert expected_code in str(exc), str(exc)
        return
    raise AssertionError(f"expected {error_type.__name__}: {expected_code}")


def test_qemu_img(config: Any, temp: Path) -> None:
    source = temp / "source.qcow2"
    source.write_bytes(b"SOURCE-IMMUTABLE")
    runner = FakeQemuRunner()
    tool = QemuImgTool(config, runner=runner)
    before = tool.hash_file(source)
    check = tool.check_read_only(source)
    overlay = tool.create_rescue_overlay(source, temp / "rescue.qcow2")
    after = tool.hash_file(source)
    assert before == after == check["source_sha256"] == overlay["source_sha256"]
    create_call = next(call for call in runner.calls if call[1] == "create")
    assert "-F" in create_call and "-b" in create_call and "-B" not in create_call and "-r" not in create_call
    assert not any(command in create_call for command in config.runtime["image"]["forbidden_mutating_subcommands"])

    symlink = temp / "source-link.qcow2"
    try:
        symlink.symlink_to(source)
    except OSError:
        pass
    else:
        expect_error(lambda: tool.info(symlink), config.codes["workspace_invalid"], QemuImgError)


def test_repository(config: Any, temp: Path) -> None:
    repo = RecoveryRepository(config, root=temp / "repo")
    session = repo.create_session("vm-safe")
    sidecar = temp / "nvram.fd"
    sidecar.write_bytes(b"NVRAM")
    cloned = repo.sidecar_path(session, "firmware_nvram", sidecar)
    assert cloned.read_bytes() == b"NVRAM"
    expect_error(
        lambda: repo.sidecar_path(session, "host_bound_secret_blob", sidecar),
        config.codes["forbidden_sidecar_copy"],
        RecoveryRepositoryError,
    )
    link = temp / "nvram-link.fd"
    try:
        link.symlink_to(sidecar)
    except OSError:
        pass
    else:
        expect_error(
            lambda: repo.sidecar_path(session, "firmware_nvram", link),
            config.codes["workspace_invalid"],
            RecoveryRepositoryError,
        )


def test_network_tool(config: Any) -> None:
    FakeQmp.log.clear()
    FakeQmp.events.clear()
    tool = NetworkRecoveryTool(config, qmp_factory=FakeQmp)
    state = NicRuntimeState(
        netdev_id="net0",
        device_id="nic0",
        driver="virtio-net-pci",
        mac="52:54:00:12:34:56",
        backend_type="user",
        backend_arguments={"ipv6": True},
        device_arguments={"bus": "pci.0"},
    )
    result = tool.recreate_backend("fake", state)
    commands = [item[0] for item in FakeQmp.log]
    assert commands == ["device_del", "netdev_del", "netdev_add", "device_add"]
    assert FakeQmp.events == [("DEVICE_DELETED", "nic0")]
    assert result["mac"] == state.mac.lower()
    device_add = FakeQmp.log[-1][1]
    assert device_add["mac"] == state.mac.lower()

    reconnect_start = len(FakeQmp.log)
    tool.reconnect_link("fake", state)
    reconnect = FakeQmp.log[reconnect_start:]
    assert reconnect[0][1]["up"] is False and reconnect[1][1]["up"] is True

    bad_mac = NicRuntimeState(**{**state.__dict__, "mac": "00:11:22:33:44"})
    expect_error(lambda: tool.validate_state(bad_mac), config.codes["guest_mac_change_blocked"], NetworkRecoveryError)
    bad_args = NicRuntimeState(**{**state.__dict__, "backend_arguments": {"hostfwd": "tcp::22-:22"}})
    expect_error(lambda: tool.validate_state(bad_args), config.codes["network_arguments_rejected"], NetworkRecoveryError)

    assert tool.rescue_backend_for_platform("fake", "linux", {"passt": True}) == "passt"
    assert tool.rescue_backend_for_platform("fake", "linux", {"passt": False}) == "user"
    assert tool.qmp_backend_types("fake") == {"user", "passt"}


def _qmp_server(server: socket.socket) -> None:
    conn, _ = server.accept()
    stream = conn.makefile("rwb")
    greeting = {"QMP": {"version": {"qemu": {"major": 11, "minor": 1, "micro": 0}}, "capabilities": []}}
    stream.write(json.dumps(greeting).encode("utf-8") + b"\r\n")
    stream.flush()
    for index in range(2):
        line = stream.readline()
        request = json.loads(line.decode("utf-8"))
        if index == 1:
            event = {"event": "DEVICE_DELETED", "data": {"device": "nic0", "path": "/machine/peripheral/nic0"}}
            stream.write(json.dumps(event).encode("utf-8") + b"\r\n")
        response = {"return": {"ok": True}, "id": request["id"]}
        stream.write(json.dumps(response).encode("utf-8") + b"\r\n")
        stream.flush()
    stream.close()
    conn.close()
    server.close()


def test_real_qmp_protocol(config: Any, temp: Path) -> None:
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", 0))
    server.listen(1)
    port = int(server.getsockname()[1])
    thread = threading.Thread(target=_qmp_server, args=(server,), daemon=True)
    thread.start()
    with QmpClient(config, f"tcp://127.0.0.1:{port}") as client:
        result = client.execute("query-status", {})
        assert result == {"ok": True}
        event = client.wait_for_event("DEVICE_DELETED", device_id="nic0")
        assert event["data"]["device"] == "nic0"
    thread.join(timeout=2)
    assert not thread.is_alive()


def test_virtiofs(config: Any, temp: Path) -> None:
    tool = VirtioFsTool(config)
    allow_root = temp / "allowed"
    share = allow_root / "share"
    outside = temp / "outside"
    share.mkdir(parents=True)
    outside.mkdir()
    assert tool.confine_share_path(share, [allow_root]) == share.resolve()
    escape = allow_root / "escape"
    try:
        escape.symlink_to(outside, target_is_directory=True)
    except OSError:
        pass
    else:
        expect_error(lambda: tool.confine_share_path(escape, [allow_root]), config.codes["share_path_rejected"], VirtioFsError)

    expect_error(
        lambda: tool.validate_guest_capability("windows", {"virtiofs_driver"}),
        config.codes["windows_virtiofs_blocked"],
        VirtioFsError,
    )
    tool.validate_guest_capability("windows", {"virtiofs_driver", "virtiofs_service", "winfsp_core"})
    tool.validate_guest_capability("linux", {"virtiofs"})

    session = VirtioFsSession("safe-tag", share, temp / "vfs.sock", True, None)
    fragment = tool.qemu_device_fragment(session, "char0", "vfs0")
    assert "vhost-user-fs-pci" in fragment[-1]
    bad_session = VirtioFsSession("bad,tag", share, temp / "vfs.sock", True, None)
    expect_error(lambda: tool.qemu_device_fragment(bad_session, "char0", "vfs0"), config.codes["share_path_rejected"], VirtioFsError)


def test_tvgb(config: Any, temp: Path) -> None:
    source = temp / "drop.txt"
    source.write_text("turkuazvm-k", encoding="utf-8")
    required = set(config.runtime["drag_drop"]["required_capabilities"])
    runner = FakeTvgbRunner(required)
    tool = TvgbTransferTool(config, runner=runner)
    result = tool.send_file(source, "Desktop/drop.txt")
    assert result["status"] == "PASS"
    send_call = runner.calls[-1]
    assert "required" in send_call and "reject" in send_call
    expect_error(lambda: tool.send_file(source, "../../escape.txt"), config.codes["drag_drop_rejected"], TvgbTransferError)
    expect_error(lambda: tool.send_file(source, "C:\\escape.txt"), config.codes["drag_drop_rejected"], TvgbTransferError)


def test_rescue_service(config: Any, temp: Path) -> None:
    source = temp / "vm.qcow2"
    source.write_bytes(b"VM-SOURCE")
    sidecars: dict[str, Path] = {}
    for role in config.runtime["rebind"]["source_inventory_roles"]:
        path = temp / f"{role}.bin"
        path.write_bytes(role.encode("ascii"))
        sidecars[role] = path

    destination_host = {
        "platform": "linux",
        "machine": "x86_64",
        "architecture": "64bit",
        "cpu_count": 16,
        "memory_bytes": 34359738368,
        "binaries": {"passt": True},
        "fingerprint": "destination-host",
    }
    source_host = {**destination_host, "cpu_count": 8, "fingerprint": "source-host"}
    profile = {
        "firmware_mode": "uefi",
        "disk_bus": "virtio-blk",
        "machine_profile": "pc-q35",
        "architecture": "x86_64",
        "guest_identity": "guest-001",
    }
    repo = RecoveryRepository(config, root=temp / "service-repo")
    service = HostResilienceService(
        config,
        repository=repo,
        qemu_img=FakeQemuImgService(source),
        host_capability=FakeHostCapability(destination_host),
    )
    result = service.prepare_image_rescue(
        vm_id="vm-001",
        source_image=source,
        sidecars=sidecars,
        source_host_inventory=source_host,
        guest_profile=profile,
    )
    assert source.read_bytes() == b"VM-SOURCE"
    assert Path(result["rescue_overlay"]).is_file()
    assert result["rebind_plan"]["firmware_mode"] == "uefi"
    assert result["rebind_plan"]["disk_bus"] == "virtio-blk"
    assert result["rebind_plan"]["hardware_changes"]["cpu_count"] == {"from": 8, "to": 16}
    persisted = repo.read_state(Path(result["session"]))
    assert persisted["status"] == "READY_FOR_BOOT_TEST"

    bad_profile = {**profile, "architecture": "aarch64"}
    expect_error(
        lambda: service.prepare_image_rescue(
            vm_id="vm-arm",
            source_image=source,
            sidecars=sidecars,
            source_host_inventory=source_host,
            guest_profile=bad_profile,
        ),
        config.codes["hardware_incompatible"],
        HostResilienceServiceError,
    )


def test_network_service_order(config: Any, temp: Path) -> None:
    destination_host = {
        "platform": "linux",
        "machine": "x86_64",
        "architecture": "64bit",
        "cpu_count": 4,
        "memory_bytes": 1024,
        "binaries": {},
        "fingerprint": "host",
    }
    guest_network = FakeGuestNetwork([True], soft_renew_ok=False)
    network = FakeNetworkRuntime()
    service = HostResilienceService(
        config,
        repository=RecoveryRepository(config, root=temp / "network-repo"),
        host_capability=FakeHostCapability(destination_host),
        network=network,
        guest_network=guest_network,
    )
    state = NicRuntimeState("net0", "nic0", "virtio-net-pci", "52:54:00:aa:bb:cc", "user", {}, {})
    result = service.recover_network(endpoint="fake", state=state, platform_name="linux", host_capabilities={"passt": True})
    assert [item["mode"] for item in result["attempts"]] == ["guest_soft_renew", "same_backend_reconnect"]
    assert network.calls == ["reconnect"]

    guest_network2 = FakeGuestNetwork([False, True], soft_renew_ok=False)
    network2 = FakeNetworkRuntime()
    service2 = HostResilienceService(
        config,
        repository=RecoveryRepository(config, root=temp / "network-repo-2"),
        host_capability=FakeHostCapability(destination_host),
        network=network2,
        guest_network=guest_network2,
    )
    result2 = service2.recover_network(endpoint="fake", state=state, platform_name="linux", host_capabilities={"passt": True})
    assert [item["mode"] for item in result2["attempts"]] == ["guest_soft_renew", "same_backend_reconnect", "netdev_recreate_same_mac"]
    assert network2.calls == ["reconnect", "recreate:user"]

    guest_network3 = FakeGuestNetwork([False, False, True], soft_renew_ok=False)
    network3 = FakeNetworkRuntime()
    service3 = HostResilienceService(
        config,
        repository=RecoveryRepository(config, root=temp / "network-repo-3"),
        host_capability=FakeHostCapability(destination_host),
        network=network3,
        guest_network=guest_network3,
    )
    result3 = service3.recover_network(endpoint="fake", state=state, platform_name="linux", host_capabilities={"passt": True})
    assert [item["mode"] for item in result3["attempts"]] == [
        "guest_soft_renew",
        "same_backend_reconnect",
        "netdev_recreate_same_mac",
        "rescue_backend",
    ]
    assert network3.calls == ["reconnect", "recreate:user", "rescue_select", "recreate:user"]


def test_controller_layer() -> None:
    source = (ROOT / "controllers" / "host_resilience_controller.py").read_text(encoding="utf-8")
    forbidden = ("subprocess", "socket.", "shutil", "yaml.", "qemu-img", "virtiofsd")
    assert not any(token in source for token in forbidden)
    assert "self.service." in source


def main() -> int:
    config = load_runtime_config()
    with tempfile.TemporaryDirectory(prefix="tvm-k-test-") as temp_name:
        temp = Path(temp_name)
        test_qemu_img(config, temp)
        test_repository(config, temp)
        test_network_tool(config)
        test_real_qmp_protocol(config, temp)
        test_virtiofs(config, temp)
        test_tvgb(config, temp)
        test_rescue_service(config, temp)
        test_network_service_order(config, temp)
        test_controller_layer()
    print(f"{config.codes['pass']} PASS (9 runtime regression groups)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
