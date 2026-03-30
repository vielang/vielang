use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OTA Package type — firmware hoặc software update
/// Java: org.thingsboard.server.common.data.ota.OtaPackageType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OtaPackageType {
    Firmware,
    Software,
}

impl OtaPackageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OtaPackageType::Firmware => "FIRMWARE",
            OtaPackageType::Software => "SOFTWARE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "FIRMWARE" => OtaPackageType::Firmware,
            "SOFTWARE" => OtaPackageType::Software,
            _ => OtaPackageType::Firmware,
        }
    }
}

/// Checksum algorithm for OTA package verification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChecksumAlgorithm {
    Md5,
    Sha256,
    Sha384,
    Sha512,
    Crc32,
    Murmur3_32,
    Murmur3_128,
}

impl ChecksumAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChecksumAlgorithm::Md5 => "MD5",
            ChecksumAlgorithm::Sha256 => "SHA256",
            ChecksumAlgorithm::Sha384 => "SHA384",
            ChecksumAlgorithm::Sha512 => "SHA512",
            ChecksumAlgorithm::Crc32 => "CRC32",
            ChecksumAlgorithm::Murmur3_32 => "MURMUR3_32",
            ChecksumAlgorithm::Murmur3_128 => "MURMUR3_128",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "MD5" => Some(ChecksumAlgorithm::Md5),
            "SHA256" => Some(ChecksumAlgorithm::Sha256),
            "SHA384" => Some(ChecksumAlgorithm::Sha384),
            "SHA512" => Some(ChecksumAlgorithm::Sha512),
            "CRC32" => Some(ChecksumAlgorithm::Crc32),
            "MURMUR3_32" => Some(ChecksumAlgorithm::Murmur3_32),
            "MURMUR3_128" => Some(ChecksumAlgorithm::Murmur3_128),
            _ => None,
        }
    }
}

/// OTA Package — firmware hoặc software package cho device updates
/// Java: org.thingsboard.server.common.data.OtaPackage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtaPackage {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub device_profile_id: Option<Uuid>,

    /// Package type: FIRMWARE or SOFTWARE
    pub ota_package_type: OtaPackageType,

    /// Package title (e.g., "Main Firmware")
    pub title: String,
    /// Version string (e.g., "1.0.0", "2.1.3-beta")
    pub version: String,
    /// Optional tag for grouping (e.g., "stable", "beta")
    pub tag: Option<String>,

    /// External URL if package is hosted externally
    pub url: Option<String>,

    /// File metadata
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub data_size: Option<i64>,

    /// Checksum for verification
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    pub checksum: Option<String>,

    /// Whether binary data is stored in DB
    pub has_data: bool,

    pub additional_info: Option<serde_json::Value>,
    pub version_int: i64,
}

impl OtaPackage {
    /// Check if this package has associated binary data
    pub fn is_url_based(&self) -> bool {
        self.url.is_some() && !self.has_data
    }
}

/// OTA Package info — lightweight version for list responses
/// Java: org.thingsboard.server.common.data.OtaPackageInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtaPackageInfo {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub device_profile_id: Option<Uuid>,
    pub ota_package_type: OtaPackageType,
    pub title: String,
    pub version: String,
    pub tag: Option<String>,
    pub url: Option<String>,
    pub has_data: bool,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub data_size: Option<i64>,
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    pub checksum: Option<String>,
}

impl From<OtaPackage> for OtaPackageInfo {
    fn from(pkg: OtaPackage) -> Self {
        Self {
            id: pkg.id,
            created_time: pkg.created_time,
            tenant_id: pkg.tenant_id,
            device_profile_id: pkg.device_profile_id,
            ota_package_type: pkg.ota_package_type,
            title: pkg.title,
            version: pkg.version,
            tag: pkg.tag,
            url: pkg.url,
            has_data: pkg.has_data,
            file_name: pkg.file_name,
            content_type: pkg.content_type,
            data_size: pkg.data_size,
            checksum_algorithm: pkg.checksum_algorithm,
            checksum: pkg.checksum,
        }
    }
}
