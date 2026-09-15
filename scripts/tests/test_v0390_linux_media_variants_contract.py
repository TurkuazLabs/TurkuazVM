# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0390_linux_media_variants_contract.py
# 📌 Amac: Linux Desktop/Workstation ile Server/NetInstall resmi medya ayrimini guest katalogunda fail-closed dogrular
# 📌 Modul - Python
# Version: 0.39.4
# Aciklama: Ubuntu, Debian ve Fedora ayri edition medyalarini; Fedora netinstall ve Rocky boot server varsayilanlarini korur
# Bagimli Oldugu Katman: Repo | Service | View | Config

from __future__ import annotations

from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
CATALOG = yaml.safe_load((ROOT / "config/guest-catalog.yml").read_text(encoding="utf-8"))


def template(template_id: str) -> dict:
    for item in CATALOG["templates"]:
        if item["id"] == template_id:
            return item
    raise AssertionError(f"LINUX_TEMPLATE_MISSING: {template_id}")


def main() -> None:
    ubuntu_desktop = template("ubuntu-26.04-lts-desktop")
    ubuntu_server = template("ubuntu-26.04-lts-server")
    assert "desktop-amd64.iso" in ubuntu_desktop["installer_media"]["url"]
    assert "live-server-amd64.iso" in ubuntu_server["installer_media"]["url"]
    assert ubuntu_desktop["installer_media"]["url"] != ubuntu_server["installer_media"]["url"]

    debian_desktop = template("debian-13-desktop-gnome")
    debian_server = template("debian-13-server")
    assert "current-live" in debian_desktop["installer_media"]["url"]
    assert "gnome.iso" in debian_desktop["installer_media"]["url"]
    assert "netinst.iso" in debian_server["installer_media"]["url"]

    fedora_desktop = template("fedora-44-workstation")
    fedora_server = template("fedora-44-server")
    assert "/Workstation/" in fedora_desktop["installer_media"]["url"]
    assert "Workstation-Live" in fedora_desktop["installer_media"]["filename"]
    assert "/Server/" in fedora_server["installer_media"]["url"]
    assert "Server-netinst" in fedora_server["installer_media"]["filename"]
    assert any("Server-dvd" in media.get("filename", "") for media in fedora_server["installer_media_options"])

    mint = template("linux-mint-22.3-cinnamon")
    assert mint["profile_label"] == "Cinnamon Desktop"
    assert mint["installer_media"]["mode"] == "official_page"
    assert not any(
        item.get("product_id") == "linux-mint" and "server" in item.get("profile_id", "").lower()
        for item in CATALOG["templates"]
    ), "LINUX_MINT_FAKE_SERVER_PROFILE_NOT_ALLOWED"

    rocky = template("rocky-10-server")
    assert "boot.iso" in rocky["installer_media"]["url"]
    assert any("minimal.iso" in media.get("url", "") for media in rocky["installer_media_options"])
    assert any("dvd.iso" in media.get("url", "") for media in rocky["installer_media_options"])
    assert not any(
        item.get("product_id") == "rocky-linux" and "desktop" in item.get("profile_id", "").lower()
        for item in CATALOG["templates"]
    ), "ROCKY_FAKE_DESKTOP_PROFILE_NOT_ALLOWED"

    for item in (ubuntu_desktop, ubuntu_server, debian_desktop, debian_server, fedora_desktop, fedora_server, rocky):
        media = item["installer_media"]
        if media["mode"] == "direct":
            assert media.get("checksum_url"), f"LINUX_CHECKSUM_URL_MISSING: {item['id']}"

    print("V0390_LINUX_MEDIA_VARIANTS_CONTRACT=PASS")


if __name__ == "__main__":
    main()
