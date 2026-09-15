#!/usr/bin/env python3
# 📄 Dosya Yolu: /turkuazvm/guest/android-image/scripts/assemble_turkuaz_android_composite.py
# 📌 Amac: Cuttlefish Android partition artifactlerini QEMU icin tek GPT composite diskte birlestirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Sparse image dosyalarini raw formata cevirir, U-Boot environment ve A/B partition tablosu ile composite.img uretir
# Bagimli Oldugu Katman: Tool | Config

from __future__ import annotations

import argparse
import binascii
import shutil
import struct
import subprocess
import tempfile
import uuid
from pathlib import Path

SECTOR_SIZE = 512
ALIGNMENT_BYTES = 1024 * 1024
ALIGNMENT_LBA = ALIGNMENT_BYTES // SECTOR_SIZE
GPT_ENTRY_COUNT = 128
GPT_ENTRY_SIZE = 128
GPT_ENTRY_SECTORS = (GPT_ENTRY_COUNT * GPT_ENTRY_SIZE) // SECTOR_SIZE
GPT_HEADER_SIZE = 92
MIN_PARTITION_BYTES = ALIGNMENT_BYTES
ANDROID_SPARSE_MAGIC = 0xED26FF3A
BASIC_DATA_GUID = uuid.UUID("EBD0A0A2-B9E5-4433-87C0-68B6B72699C7")
UBOOT_ENV_BYTES = 4096
UBOOT_BOOT_COMMAND = "bootcmd=boot_android virtio 0#misc"


def align_up(value: int, alignment: int) -> int:
    return ((value + alignment - 1) // alignment) * alignment


def is_sparse(path: Path) -> bool:
    with path.open("rb") as handle:
        raw = handle.read(4)
    return len(raw) == 4 and struct.unpack("<I", raw)[0] == ANDROID_SPARSE_MAGIC


def raw_image(source: Path, work_root: Path, cache: dict[Path, Path]) -> Path:
    if source in cache:
        return cache[source]
    if not is_sparse(source):
        cache[source] = source
        return source
    simg2img = shutil.which("simg2img")
    if simg2img is None:
        raise RuntimeError("simg2img is required for Android sparse image conversion")
    target = work_root / f"{source.stem}.raw.img"
    subprocess.run([simg2img, str(source), str(target)], check=True)
    cache[source] = target
    return target


def create_blank(path: Path, size_bytes: int) -> Path:
    with path.open("wb") as handle:
        handle.truncate(size_bytes)
    return path


def create_uboot_env(path: Path) -> Path:
    data = (UBOOT_BOOT_COMMAND + "\0\0").encode("ascii")
    payload_size = UBOOT_ENV_BYTES - 4
    if len(data) > payload_size:
        raise RuntimeError("U-Boot environment payload exceeds fixed size")
    payload = data + (b"\xff" * (payload_size - len(data)))
    crc = binascii.crc32(payload) & 0xFFFFFFFF
    path.write_bytes(struct.pack("<I", crc) + payload)
    return path


def existing(product_out: Path, name: str) -> Path | None:
    path = product_out / name
    return path if path.is_file() and path.stat().st_size > 0 else None


def required(product_out: Path, name: str) -> Path:
    path = existing(product_out, name)
    if path is None:
        raise RuntimeError(f"Required Android artifact is missing: {name}")
    return path


def partition_sources(product_out: Path, work_root: Path) -> list[tuple[str, Path]]:
    env = create_uboot_env(work_root / "uboot_env.img")
    misc = existing(product_out, "misc.img") or create_blank(work_root / "misc.img", ALIGNMENT_BYTES)
    metadata = existing(product_out, "metadata.img") or create_blank(
        work_root / "metadata.img", 16 * ALIGNMENT_BYTES
    )
    boot = required(product_out, "boot.img")
    super_image = required(product_out, "super.img")
    userdata = required(product_out, "userdata.img")
    init_boot = existing(product_out, "init_boot.img")
    vendor_boot = existing(product_out, "vendor_boot.img")
    vbmeta = existing(product_out, "vbmeta.img")
    vbmeta_system = existing(product_out, "vbmeta_system.img")

    partitions: list[tuple[str, Path]] = [("uboot_env", env), ("misc", misc)]
    partitions.extend([("boot_a", boot), ("boot_b", boot)])
    if init_boot is not None:
        partitions.extend([("init_boot_a", init_boot), ("init_boot_b", init_boot)])
    if vendor_boot is not None:
        partitions.extend([("vendor_boot_a", vendor_boot), ("vendor_boot_b", vendor_boot)])
    if vbmeta is not None:
        partitions.extend([("vbmeta_a", vbmeta), ("vbmeta_b", vbmeta)])
    if vbmeta_system is not None:
        partitions.extend([("vbmeta_system_a", vbmeta_system), ("vbmeta_system_b", vbmeta_system)])
    partitions.extend([("super", super_image), ("userdata", userdata), ("metadata", metadata)])
    return partitions


def gpt_entry(name: str, first_lba: int, last_lba: int) -> bytes:
    encoded_name = name.encode("utf-16le")[:72].ljust(72, b"\0")
    return struct.pack(
        "<16s16sQQQ72s",
        BASIC_DATA_GUID.bytes_le,
        uuid.uuid4().bytes_le,
        first_lba,
        last_lba,
        0,
        encoded_name,
    )


def gpt_header(
    current_lba: int,
    backup_lba: int,
    first_usable_lba: int,
    last_usable_lba: int,
    disk_guid: uuid.UUID,
    entries_lba: int,
    entries_crc: int,
) -> bytes:
    header = bytearray(SECTOR_SIZE)
    struct.pack_into(
        "<8sIIIIQQQQ16sQIII",
        header,
        0,
        b"EFI PART",
        0x00010000,
        GPT_HEADER_SIZE,
        0,
        0,
        current_lba,
        backup_lba,
        first_usable_lba,
        last_usable_lba,
        disk_guid.bytes_le,
        entries_lba,
        GPT_ENTRY_COUNT,
        GPT_ENTRY_SIZE,
        entries_crc,
    )
    crc = binascii.crc32(header[:GPT_HEADER_SIZE]) & 0xFFFFFFFF
    struct.pack_into("<I", header, 16, crc)
    return bytes(header)


def protective_mbr(total_lba: int) -> bytes:
    mbr = bytearray(SECTOR_SIZE)
    sector_count = min(max(total_lba - 1, 1), 0xFFFFFFFF)
    partition = struct.pack(
        "<B3sB3sII",
        0,
        b"\x00\x02\x00",
        0xEE,
        b"\xff\xff\xff",
        1,
        sector_count,
    )
    mbr[446 : 446 + len(partition)] = partition
    mbr[510:512] = b"\x55\xaa"
    return bytes(mbr)


def assemble(product_out: Path, output: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="turkuazvm-composite-") as temp_dir:
        work_root = Path(temp_dir)
        cache: dict[Path, Path] = {}
        sources = partition_sources(product_out, work_root)
        resolved: list[tuple[str, Path, int, int]] = []
        next_lba = ALIGNMENT_LBA
        entries = bytearray(GPT_ENTRY_COUNT * GPT_ENTRY_SIZE)

        for index, (name, source) in enumerate(sources):
            raw = raw_image(source, work_root, cache)
            source_size = raw.stat().st_size
            partition_size = align_up(max(source_size, MIN_PARTITION_BYTES), ALIGNMENT_BYTES)
            first_lba = align_up(next_lba, ALIGNMENT_LBA)
            sector_count = partition_size // SECTOR_SIZE
            last_lba = first_lba + sector_count - 1
            entry = gpt_entry(name, first_lba, last_lba)
            start = index * GPT_ENTRY_SIZE
            entries[start : start + GPT_ENTRY_SIZE] = entry
            resolved.append((name, raw, first_lba, source_size))
            next_lba = last_lba + 1

        backup_header_lba = align_up(next_lba + GPT_ENTRY_SECTORS + 1, ALIGNMENT_LBA) - 1
        backup_entries_lba = backup_header_lba - GPT_ENTRY_SECTORS
        first_usable_lba = 2 + GPT_ENTRY_SECTORS
        last_usable_lba = backup_entries_lba - 1
        total_lba = backup_header_lba + 1
        entries_crc = binascii.crc32(entries) & 0xFFFFFFFF
        disk_guid = uuid.uuid4()
        primary_header = gpt_header(
            1,
            backup_header_lba,
            first_usable_lba,
            last_usable_lba,
            disk_guid,
            2,
            entries_crc,
        )
        backup_header = gpt_header(
            backup_header_lba,
            1,
            first_usable_lba,
            last_usable_lba,
            disk_guid,
            backup_entries_lba,
            entries_crc,
        )

        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("wb") as disk:
            disk.truncate(total_lba * SECTOR_SIZE)
            disk.seek(0)
            disk.write(protective_mbr(total_lba))
            disk.seek(SECTOR_SIZE)
            disk.write(primary_header)
            disk.seek(2 * SECTOR_SIZE)
            disk.write(entries)
            for _, source, first_lba, _ in resolved:
                disk.seek(first_lba * SECTOR_SIZE)
                with source.open("rb") as source_handle:
                    shutil.copyfileobj(source_handle, disk, length=8 * ALIGNMENT_BYTES)
            disk.seek(backup_entries_lba * SECTOR_SIZE)
            disk.write(entries)
            disk.seek(backup_header_lba * SECTOR_SIZE)
            disk.write(backup_header)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--product-out", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    assemble(args.product_out.resolve(), args.output.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
