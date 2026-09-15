// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/domain/build_profile.rs
// # 📌 Amac: AOSP Android image build target ve kaynak politikasini typed domain modelleriyle tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Branch, mimari, release config ve build variant degerlerini shell stringlerinden ayirir
// # Bagimli Oldugu Katman: Service | Tool | View

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidImageArchitecture {
    X86_64,
    Arm64,
}

impl AndroidImageArchitecture {
    pub const fn code(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Arm64 => "arm64",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidSourceTrack {
    LatestRelease,
}

impl AndroidSourceTrack {
    pub const fn branch(self) -> &'static str {
        match self {
            Self::LatestRelease => "android-latest-release",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidReleaseConfig {
    AospCurrent,
}

impl AndroidReleaseConfig {
    pub const fn code(self) -> &'static str {
        match self {
            Self::AospCurrent => "aosp_current",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidBuildVariant {
    Userdebug,
}

impl AndroidBuildVariant {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Userdebug => "userdebug",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidBuildProduct {
    TurkuazCuttlefishX86_64Phone,
}

impl AndroidBuildProduct {
    pub const fn code(self) -> &'static str {
        match self {
            Self::TurkuazCuttlefishX86_64Phone => "turkuazvm_cf_x86_64_phone",
        }
    }

    pub const fn architecture(self) -> AndroidImageArchitecture {
        match self {
            Self::TurkuazCuttlefishX86_64Phone => AndroidImageArchitecture::X86_64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndroidImageBuildProfile {
    pub source_track: AndroidSourceTrack,
    pub product: AndroidBuildProduct,
    pub release_config: AndroidReleaseConfig,
    pub variant: AndroidBuildVariant,
}

impl AndroidImageBuildProfile {
    pub const fn gaming_x86_64() -> Self {
        Self {
            source_track: AndroidSourceTrack::LatestRelease,
            product: AndroidBuildProduct::TurkuazCuttlefishX86_64Phone,
            release_config: AndroidReleaseConfig::AospCurrent,
            variant: AndroidBuildVariant::Userdebug,
        }
    }
}
