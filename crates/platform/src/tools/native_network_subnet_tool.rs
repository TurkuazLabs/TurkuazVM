// # 📄 Dosya Yolu: /turkuazvm/crates/platform/src/tools/native_network_subnet_tool.rs
// # 📌 Amac: TurkuazVM managed network icin host aglariyla cakismayan private /24 subnet secer
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Windows route tablosunu Tool katmaninda okuyup tercih edilen 192.168.240.0/24 ve alternatif private bloklari fail-safe planlar
// # Bagimli Oldugu Katman: Tool

use std::net::Ipv4Addr;
use std::process::{Command, Stdio};

const COMMAND_POWERSHELL: &str = "powershell.exe";
const PREFIX_LENGTH: u8 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagedSubnetPlan {
    pub subnet: Ipv4Addr,
    pub gateway: Ipv4Addr,
}

pub struct NativeNetworkSubnetTool;

impl NativeNetworkSubnetTool {
    pub fn resolve_private_24(
        preferred_subnet: Ipv4Addr,
        preferred_gateway: Ipv4Addr,
        reserved_subnets: &[Ipv4Addr],
    ) -> ManagedSubnetPlan {
        let occupied = Self::host_ipv4_routes();
        let gateway_host = preferred_gateway.octets()[3].max(1);
        for candidate in Self::candidate_subnets(preferred_subnet) {
            if reserved_subnets.contains(&candidate) || Self::conflicts(candidate, &occupied) {
                continue;
            }
            let octets = candidate.octets();
            return ManagedSubnetPlan {
                subnet: candidate,
                gateway: Ipv4Addr::new(octets[0], octets[1], octets[2], gateway_host),
            };
        }
        ManagedSubnetPlan { subnet: preferred_subnet, gateway: preferred_gateway }
    }

    fn host_ipv4_routes() -> Vec<(Ipv4Addr, u8)> {
        if std::env::consts::OS != "windows" {
            return Vec::new();
        }
        let output = Command::new(COMMAND_POWERSHELL)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-NetRoute -AddressFamily IPv4 -ErrorAction SilentlyContinue | ForEach-Object { $_.DestinationPrefix }",
            ])
            .stdin(Stdio::null())
            .output();
        let Ok(output) = output else { return Vec::new(); };
        if !output.status.success() { return Vec::new(); }
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(Self::parse_prefix)
            .filter(|(_, prefix)| *prefix > 0)
            .collect()
    }

    fn parse_prefix(value: &str) -> Option<(Ipv4Addr, u8)> {
        let (address, prefix) = value.trim().split_once('/')?;
        let address = address.parse::<Ipv4Addr>().ok()?;
        let prefix = prefix.parse::<u8>().ok()?;
        (prefix <= 32).then_some((address, prefix))
    }

    fn conflicts(candidate: Ipv4Addr, occupied: &[(Ipv4Addr, u8)]) -> bool {
        occupied.iter().any(|(network, prefix)| Self::subnets_overlap(candidate, PREFIX_LENGTH, *network, *prefix))
    }

    fn subnets_overlap(left: Ipv4Addr, left_prefix: u8, right: Ipv4Addr, right_prefix: u8) -> bool {
        let common = left_prefix.min(right_prefix);
        let mask = if common == 0 { 0 } else { u32::MAX << (32 - u32::from(common)) };
        (u32::from(left) & mask) == (u32::from(right) & mask)
    }

    fn candidate_subnets(preferred: Ipv4Addr) -> Vec<Ipv4Addr> {
        let mut candidates = Vec::with_capacity(270);
        candidates.push(preferred);
        for third in 240_u8..=254 {
            let candidate = Ipv4Addr::new(192, 168, third, 0);
            if candidate != preferred { candidates.push(candidate); }
        }
        for third in 240_u8..=254 {
            candidates.push(Ipv4Addr::new(172, 31, third, 0));
        }
        for third in 0_u8..=254 {
            candidates.push(Ipv4Addr::new(10, 240, third, 0));
        }
        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::NativeNetworkSubnetTool;
    use std::net::Ipv4Addr;

    #[test]
    fn overlap_detection_handles_parent_route() {
        assert!(NativeNetworkSubnetTool::subnets_overlap(
            Ipv4Addr::new(192, 168, 240, 0), 24,
            Ipv4Addr::new(192, 168, 0, 0), 16,
        ));
        assert!(!NativeNetworkSubnetTool::subnets_overlap(
            Ipv4Addr::new(192, 168, 240, 0), 24,
            Ipv4Addr::new(10, 0, 0, 0), 8,
        ));
    }
}
