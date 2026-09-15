// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/android_composite_disk_tool.rs
// # 📌 Amac: Cuttlefish Android partitionlarini QEMU icin GPT composite diske donusturur
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Android sparse image formatini ek binary gerektirmeden acar, U-Boot environment ve A/B GPT partitionlarini tek composite.img icinde uretir
// # Bagimli Oldugu Katman: Tool

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SECTOR_SIZE: u64 = 512;
const ALIGNMENT_BYTES: u64 = 1024 * 1024;
const ALIGNMENT_LBA: u64 = ALIGNMENT_BYTES / SECTOR_SIZE;
const GPT_ENTRY_COUNT: usize = 128;
const GPT_ENTRY_SIZE: usize = 128;
const GPT_ENTRY_SECTORS: u64 = (GPT_ENTRY_COUNT * GPT_ENTRY_SIZE) as u64 / SECTOR_SIZE;
const GPT_HEADER_SIZE: usize = 92;
const UBOOT_ENV_BYTES: usize = 4096;
const ANDROID_SPARSE_MAGIC: u32 = 0xED26_FF3A;
const SPARSE_CHUNK_RAW: u16 = 0xCAC1;
const SPARSE_CHUNK_FILL: u16 = 0xCAC2;
const SPARSE_CHUNK_DONT_CARE: u16 = 0xCAC3;
const SPARSE_CHUNK_CRC32: u16 = 0xCAC4;
const UBOOT_BOOT_COMMAND: &str = "bootcmd=boot_android virtio 0#misc";
const BASIC_DATA_GUID_LE: [u8; 16] = [
    0xA2, 0xA0, 0xD0, 0xEB, 0xE5, 0xB9, 0x33, 0x44,
    0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7,
];

static GUID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
enum PartitionSource {
    File(PathBuf),
    Blank(u64),
    UbootEnvironment,
}

#[derive(Debug, Clone)]
struct PartitionLayout {
    name: String,
    source: PartitionSource,
    first_lba: u64,
    source_size: u64,
}

pub struct AndroidCompositeDiskTool;

impl AndroidCompositeDiskTool {
    pub fn assemble(product_out: &Path, output: &Path) -> Result<(), String> {
        let sources = partition_sources(product_out)?;
        let mut layouts = Vec::new();
        let mut entries = vec![0_u8; GPT_ENTRY_COUNT * GPT_ENTRY_SIZE];
        let mut next_lba = ALIGNMENT_LBA;

        for (index, (name, source)) in sources.into_iter().enumerate() {
            if index >= GPT_ENTRY_COUNT {
                return Err(String::from("android composite partition count exceeds GPT capacity"));
            }
            let source_size = source_size(&source)?;
            let partition_size = align_up(source_size.max(ALIGNMENT_BYTES), ALIGNMENT_BYTES);
            let first_lba = align_up(next_lba, ALIGNMENT_LBA);
            let sector_count = partition_size / SECTOR_SIZE;
            let last_lba = first_lba
                .checked_add(sector_count)
                .and_then(|value| value.checked_sub(1))
                .ok_or_else(|| String::from("android composite partition size overflow"))?;
            let entry = gpt_entry(&name, first_lba, last_lba)?;
            let start = index * GPT_ENTRY_SIZE;
            entries[start..start + GPT_ENTRY_SIZE].copy_from_slice(&entry);
            layouts.push(PartitionLayout { name, source, first_lba, source_size });
            next_lba = last_lba + 1;
        }

        let backup_header_lba = align_up(next_lba + GPT_ENTRY_SECTORS + 1, ALIGNMENT_LBA) - 1;
        let backup_entries_lba = backup_header_lba - GPT_ENTRY_SECTORS;
        let first_usable_lba = 2 + GPT_ENTRY_SECTORS;
        let last_usable_lba = backup_entries_lba - 1;
        let total_lba = backup_header_lba + 1;
        let entries_crc = crc32(&entries);
        let disk_guid = generated_guid();
        let primary_header = gpt_header(
            1,
            backup_header_lba,
            first_usable_lba,
            last_usable_lba,
            disk_guid,
            2,
            entries_crc,
        );
        let backup_header = gpt_header(
            backup_header_lba,
            1,
            first_usable_lba,
            last_usable_lba,
            disk_guid,
            backup_entries_lba,
            entries_crc,
        );

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut disk = File::create(output).map_err(|error| error.to_string())?;
        disk.set_len(total_lba * SECTOR_SIZE).map_err(|error| error.to_string())?;
        disk.seek(SeekFrom::Start(0)).map_err(|error| error.to_string())?;
        disk.write_all(&protective_mbr(total_lba)).map_err(|error| error.to_string())?;
        disk.seek(SeekFrom::Start(SECTOR_SIZE)).map_err(|error| error.to_string())?;
        disk.write_all(&primary_header).map_err(|error| error.to_string())?;
        disk.seek(SeekFrom::Start(2 * SECTOR_SIZE)).map_err(|error| error.to_string())?;
        disk.write_all(&entries).map_err(|error| error.to_string())?;

        for layout in &layouts {
            let offset = layout.first_lba * SECTOR_SIZE;
            write_partition_source(&mut disk, offset, &layout.source, layout.source_size)
                .map_err(|error| format!("partition {}: {error}", layout.name))?;
        }

        disk.seek(SeekFrom::Start(backup_entries_lba * SECTOR_SIZE)).map_err(|error| error.to_string())?;
        disk.write_all(&entries).map_err(|error| error.to_string())?;
        disk.seek(SeekFrom::Start(backup_header_lba * SECTOR_SIZE)).map_err(|error| error.to_string())?;
        disk.write_all(&backup_header).map_err(|error| error.to_string())?;
        disk.sync_all().map_err(|error| error.to_string())?;
        Ok(())
    }
}

fn partition_sources(product_out: &Path) -> Result<Vec<(String, PartitionSource)>, String> {
    let boot = required(product_out, "boot.img")?;
    let super_image = required(product_out, "super.img")?;
    let userdata = required(product_out, "userdata.img")?;
    let mut partitions = vec![
        (String::from("uboot_env"), PartitionSource::UbootEnvironment),
        (String::from("misc"), optional_file(product_out, "misc.img").unwrap_or(PartitionSource::Blank(ALIGNMENT_BYTES))),
        (String::from("boot_a"), PartitionSource::File(boot.clone())),
        (String::from("boot_b"), PartitionSource::File(boot)),
    ];
    append_ab_optional(&mut partitions, product_out, "init_boot.img", "init_boot");
    append_ab_optional(&mut partitions, product_out, "vendor_boot.img", "vendor_boot");
    append_ab_optional(&mut partitions, product_out, "vbmeta.img", "vbmeta");
    append_ab_optional(&mut partitions, product_out, "vbmeta_system.img", "vbmeta_system");
    partitions.push((String::from("super"), PartitionSource::File(super_image)));
    partitions.push((String::from("userdata"), PartitionSource::File(userdata)));
    partitions.push((
        String::from("metadata"),
        optional_file(product_out, "metadata.img").unwrap_or(PartitionSource::Blank(16 * ALIGNMENT_BYTES)),
    ));
    Ok(partitions)
}

fn append_ab_optional(
    partitions: &mut Vec<(String, PartitionSource)>,
    root: &Path,
    file_name: &str,
    partition_name: &str,
) {
    if let Some(PartitionSource::File(path)) = optional_file(root, file_name) {
        partitions.push((format!("{partition_name}_a"), PartitionSource::File(path.clone())));
        partitions.push((format!("{partition_name}_b"), PartitionSource::File(path)));
    }
}

fn required(root: &Path, name: &str) -> Result<PathBuf, String> {
    let path = root.join(name);
    if path.is_file() && fs::metadata(&path).map_err(|error| error.to_string())?.len() > 0 {
        Ok(path)
    } else {
        Err(format!("required Android artifact is missing: {name}"))
    }
}

fn optional_file(root: &Path, name: &str) -> Option<PartitionSource> {
    let path = root.join(name);
    let size = fs::metadata(&path).ok()?.len();
    (path.is_file() && size > 0).then_some(PartitionSource::File(path))
}

fn source_size(source: &PartitionSource) -> Result<u64, String> {
    match source {
        PartitionSource::File(path) => expanded_image_size(path),
        PartitionSource::Blank(size) => Ok(*size),
        PartitionSource::UbootEnvironment => Ok(UBOOT_ENV_BYTES as u64),
    }
}

fn expanded_image_size(path: &Path) -> Result<u64, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic).map_err(|error| error.to_string())?;
    if u32::from_le_bytes(magic) != ANDROID_SPARSE_MAGIC {
        return fs::metadata(path).map(|meta| meta.len()).map_err(|error| error.to_string());
    }
    let mut rest = [0_u8; 24];
    file.read_exact(&mut rest).map_err(|error| error.to_string())?;
    let block_size = u32::from_le_bytes(rest[8..12].try_into().map_err(|_| String::from("sparse header block size invalid"))?) as u64;
    let total_blocks = u32::from_le_bytes(rest[12..16].try_into().map_err(|_| String::from("sparse header total blocks invalid"))?) as u64;
    if block_size == 0 || total_blocks == 0 {
        return Err(String::from("sparse image declares an empty geometry"));
    }
    block_size.checked_mul(total_blocks).ok_or_else(|| String::from("sparse image expanded size overflow"))
}

fn write_partition_source(disk: &mut File, offset: u64, source: &PartitionSource, expected_size: u64) -> Result<(), String> {
    disk.seek(SeekFrom::Start(offset)).map_err(|error| error.to_string())?;
    match source {
        PartitionSource::File(path) => write_image(disk, path, expected_size),
        PartitionSource::Blank(size) => {
            disk.seek(SeekFrom::Current(i64::try_from(*size).map_err(|_| String::from("blank partition too large"))?))
                .map_err(|error| error.to_string())?;
            Ok(())
        }
        PartitionSource::UbootEnvironment => disk.write_all(&uboot_environment()).map_err(|error| error.to_string()),
    }
}

fn write_image(disk: &mut File, path: &Path, expected_size: u64) -> Result<(), String> {
    let mut source = File::open(path).map_err(|error| error.to_string())?;
    let mut magic = [0_u8; 4];
    source.read_exact(&mut magic).map_err(|error| error.to_string())?;
    source.seek(SeekFrom::Start(0)).map_err(|error| error.to_string())?;
    if u32::from_le_bytes(magic) == ANDROID_SPARSE_MAGIC {
        write_sparse_image(disk, &mut source, expected_size)
    } else {
        let copied = std::io::copy(&mut source, disk).map_err(|error| error.to_string())?;
        if copied != expected_size {
            return Err(String::from("raw image copy size mismatch"));
        }
        Ok(())
    }
}

fn write_sparse_image(disk: &mut File, source: &mut File, expected_size: u64) -> Result<(), String> {
    let magic = read_u32(source)?;
    if magic != ANDROID_SPARSE_MAGIC { return Err(String::from("invalid sparse magic")); }
    let major = read_u16(source)?;
    let _minor = read_u16(source)?;
    if major != 1 { return Err(format!("unsupported Android sparse major version: {major}")); }
    let file_header_size = read_u16(source)? as u64;
    let chunk_header_size = read_u16(source)? as u64;
    let block_size = read_u32(source)? as u64;
    let total_blocks = read_u32(source)? as u64;
    let total_chunks = read_u32(source)? as u64;
    let _checksum = read_u32(source)?;
    if file_header_size < 28 || chunk_header_size < 12 || block_size == 0 {
        return Err(String::from("invalid Android sparse header"));
    }
    if file_header_size > 28 {
        source.seek(SeekFrom::Current(i64::try_from(file_header_size - 28).map_err(|_| String::from("sparse header too large"))?))
            .map_err(|error| error.to_string())?;
    }
    if block_size.checked_mul(total_blocks) != Some(expected_size) {
        return Err(String::from("sparse expanded size mismatch"));
    }

    let start = disk.stream_position().map_err(|error| error.to_string())?;
    let mut written = 0_u64;
    for _ in 0..total_chunks {
        let chunk_type = read_u16(source)?;
        let _reserved = read_u16(source)?;
        let chunk_blocks = read_u32(source)? as u64;
        let total_size = read_u32(source)? as u64;
        if total_size < chunk_header_size {
            return Err(String::from("invalid sparse chunk size"));
        }
        if chunk_header_size > 12 {
            source.seek(SeekFrom::Current(i64::try_from(chunk_header_size - 12).map_err(|_| String::from("sparse chunk header too large"))?))
                .map_err(|error| error.to_string())?;
        }
        let data_size = total_size - chunk_header_size;
        let expanded = block_size.checked_mul(chunk_blocks).ok_or_else(|| String::from("sparse chunk size overflow"))?;
        match chunk_type {
            SPARSE_CHUNK_RAW => {
                if data_size != expanded { return Err(String::from("sparse RAW chunk payload mismatch")); }
                copy_exact(source, disk, data_size)?;
                written = written.checked_add(expanded).ok_or_else(|| String::from("sparse output overflow"))?;
            }
            SPARSE_CHUNK_FILL => {
                if data_size != 4 { return Err(String::from("sparse FILL chunk payload mismatch")); }
                let pattern = read_u32(source)?.to_le_bytes();
                write_fill(disk, pattern, expanded)?;
                written = written.checked_add(expanded).ok_or_else(|| String::from("sparse output overflow"))?;
            }
            SPARSE_CHUNK_DONT_CARE => {
                if data_size > 0 {
                    source.seek(SeekFrom::Current(i64::try_from(data_size).map_err(|_| String::from("sparse skip too large"))?))
                        .map_err(|error| error.to_string())?;
                }
                disk.seek(SeekFrom::Current(i64::try_from(expanded).map_err(|_| String::from("sparse output seek too large"))?))
                    .map_err(|error| error.to_string())?;
                written = written.checked_add(expanded).ok_or_else(|| String::from("sparse output overflow"))?;
            }
            SPARSE_CHUNK_CRC32 => {
                if data_size != 4 { return Err(String::from("sparse CRC32 chunk payload mismatch")); }
                let _chunk_crc = read_u32(source)?;
                if expanded != 0 { return Err(String::from("sparse CRC32 chunk must not expand data")); }
            }
            _ => return Err(format!("unsupported Android sparse chunk type: 0x{chunk_type:04x}")),
        }
    }
    if written != expected_size {
        return Err(format!("sparse image output size mismatch: expected {expected_size}, wrote {written}"));
    }
    disk.seek(SeekFrom::Start(start + expected_size)).map_err(|error| error.to_string())?;
    Ok(())
}

fn copy_exact(source: &mut File, target: &mut File, mut count: u64) -> Result<(), String> {
    let mut buffer = vec![0_u8; 1024 * 1024];
    while count > 0 {
        let wanted = usize::try_from(count.min(buffer.len() as u64)).map_err(|_| String::from("copy size invalid"))?;
        source.read_exact(&mut buffer[..wanted]).map_err(|error| error.to_string())?;
        target.write_all(&buffer[..wanted]).map_err(|error| error.to_string())?;
        count -= wanted as u64;
    }
    Ok(())
}

fn write_fill(target: &mut File, pattern: [u8; 4], mut count: u64) -> Result<(), String> {
    if count % 4 != 0 { return Err(String::from("sparse FILL size is not 4-byte aligned")); }
    let mut buffer = vec![0_u8; 1024 * 1024];
    for chunk in buffer.chunks_exact_mut(4) { chunk.copy_from_slice(&pattern); }
    while count > 0 {
        let wanted = usize::try_from(count.min(buffer.len() as u64)).map_err(|_| String::from("fill size invalid"))?;
        target.write_all(&buffer[..wanted]).map_err(|error| error.to_string())?;
        count -= wanted as u64;
    }
    Ok(())
}

fn read_u16(reader: &mut File) -> Result<u16, String> {
    let mut bytes = [0_u8; 2];
    reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32(reader: &mut File) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn uboot_environment() -> Vec<u8> {
    let mut payload = vec![0xFF_u8; UBOOT_ENV_BYTES - 4];
    let command = format!("{UBOOT_BOOT_COMMAND}\0\0");
    let command_bytes = command.as_bytes();
    payload[..command_bytes.len()].copy_from_slice(command_bytes);
    let crc = crc32(&payload);
    let mut result = Vec::with_capacity(UBOOT_ENV_BYTES);
    result.extend_from_slice(&crc.to_le_bytes());
    result.extend_from_slice(&payload);
    result
}

fn gpt_entry(name: &str, first_lba: u64, last_lba: u64) -> Result<[u8; GPT_ENTRY_SIZE], String> {
    let mut entry = [0_u8; GPT_ENTRY_SIZE];
    entry[0..16].copy_from_slice(&BASIC_DATA_GUID_LE);
    entry[16..32].copy_from_slice(&generated_guid());
    entry[32..40].copy_from_slice(&first_lba.to_le_bytes());
    entry[40..48].copy_from_slice(&last_lba.to_le_bytes());
    entry[48..56].copy_from_slice(&0_u64.to_le_bytes());
    let mut name_bytes = Vec::new();
    for unit in name.encode_utf16().take(36) { name_bytes.extend_from_slice(&unit.to_le_bytes()); }
    if name_bytes.len() > 72 { return Err(String::from("GPT partition name too long")); }
    entry[56..56 + name_bytes.len()].copy_from_slice(&name_bytes);
    Ok(entry)
}

fn gpt_header(
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_guid: [u8; 16],
    entries_lba: u64,
    entries_crc: u32,
) -> [u8; SECTOR_SIZE as usize] {
    let mut header = [0_u8; SECTOR_SIZE as usize];
    header[0..8].copy_from_slice(b"EFI PART");
    header[8..12].copy_from_slice(&0x0001_0000_u32.to_le_bytes());
    header[12..16].copy_from_slice(&(GPT_HEADER_SIZE as u32).to_le_bytes());
    header[24..32].copy_from_slice(&current_lba.to_le_bytes());
    header[32..40].copy_from_slice(&backup_lba.to_le_bytes());
    header[40..48].copy_from_slice(&first_usable_lba.to_le_bytes());
    header[48..56].copy_from_slice(&last_usable_lba.to_le_bytes());
    header[56..72].copy_from_slice(&disk_guid);
    header[72..80].copy_from_slice(&entries_lba.to_le_bytes());
    header[80..84].copy_from_slice(&(GPT_ENTRY_COUNT as u32).to_le_bytes());
    header[84..88].copy_from_slice(&(GPT_ENTRY_SIZE as u32).to_le_bytes());
    header[88..92].copy_from_slice(&entries_crc.to_le_bytes());
    let crc = crc32(&header[..GPT_HEADER_SIZE]);
    header[16..20].copy_from_slice(&crc.to_le_bytes());
    header
}

fn protective_mbr(total_lba: u64) -> [u8; SECTOR_SIZE as usize] {
    let mut mbr = [0_u8; SECTOR_SIZE as usize];
    let sector_count = total_lba.saturating_sub(1).max(1).min(u32::MAX as u64) as u32;
    let offset = 446;
    mbr[offset] = 0;
    mbr[offset + 1..offset + 4].copy_from_slice(&[0x00, 0x02, 0x00]);
    mbr[offset + 4] = 0xEE;
    mbr[offset + 5..offset + 8].copy_from_slice(&[0xFF, 0xFF, 0xFF]);
    mbr[offset + 8..offset + 12].copy_from_slice(&1_u32.to_le_bytes());
    mbr[offset + 12..offset + 16].copy_from_slice(&sector_count.to_le_bytes());
    mbr[510..512].copy_from_slice(&[0x55, 0xAA]);
    mbr
}

fn generated_guid() -> [u8; 16] {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|value| value.as_nanos()).unwrap_or_default();
    let counter = u128::from(GUID_COUNTER.fetch_add(1, Ordering::Relaxed));
    let mixed = now ^ (counter << 64) ^ 0xA54F_D26C_19B7_4E30_9D42_6B8E_0315_7FC1_u128;
    let mut bytes = mixed.to_le_bytes();
    bytes[7] = (bytes[7] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    bytes
}

fn align_up(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320_u32 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::{crc32, uboot_environment, UBOOT_BOOT_COMMAND, UBOOT_ENV_BYTES};

    #[test]
    fn crc32_matches_standard_vector() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn uboot_environment_has_expected_size_and_command() {
        let environment = uboot_environment();
        assert_eq!(environment.len(), UBOOT_ENV_BYTES);
        let payload = &environment[4..];
        assert!(payload.starts_with(UBOOT_BOOT_COMMAND.as_bytes()));
    }
}
