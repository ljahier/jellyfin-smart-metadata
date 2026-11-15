use std::fmt::Write;

use md5::{
    Digest,
    Md5,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Deserialize)]
pub struct PluginMeta {
    pub guid: String,
    pub name: String,
    pub description: String,
    pub overview: String,
    pub owner: String,
    pub category: String,
    pub version: String,
    #[serde(rename = "targetAbi")]
    pub target_abi: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct PackageInfo {
    pub guid: String,
    pub name: String,
    pub description: String,
    pub overview: String,
    pub owner: String,
    pub category: String,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub changelog: String,
    #[serde(rename = "targetAbi")]
    pub target_abi: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    pub checksum: String,
    pub timestamp: String,
}

pub struct Release {
    pub repository: String,
    pub tag: String,
}

pub fn zip_file_name(version: &str) -> String { format!("smart-metadata_{version}.zip") }

pub fn build_manifest(
    meta: PluginMeta,
    release: &Release,
    zip: &[u8],
    timestamp: String,
) -> Vec<PackageInfo> {
    let source_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        release.repository,
        release.tag,
        zip_file_name(&meta.version)
    );
    let version = VersionInfo {
        version: meta.version,
        changelog: format!("Release {}", release.tag),
        target_abi: meta.target_abi,
        source_url,
        checksum: md5_hex(zip),
        timestamp,
    };

    vec![PackageInfo {
        guid: meta.guid,
        name: meta.name,
        description: meta.description,
        overview: meta.overview,
        owner: meta.owner,
        category: meta.category,
        versions: vec![version],
    }]
}

fn md5_hex(bytes: &[u8]) -> String {
    Md5::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        PluginMeta,
        Release,
        build_manifest,
        md5_hex,
        zip_file_name,
    };

    fn meta() -> PluginMeta {
        PluginMeta {
            guid: String::from("3f7a9c1e-5b24-4d8a-a6f0-92c1d7e84b35"),
            name: String::from("Smart Metadata"),
            description: String::from("A long description."),
            overview: String::from("A short overview."),
            owner: String::from("ljahier"),
            category: String::from("Metadata"),
            version: String::from("0.1.0.0"),
            target_abi: String::from("12.4.0.0"),
        }
    }

    #[test]
    fn computes_the_md5_of_known_bytes() {
        assert_eq!(
            md5_hex(b"abc"),
            "900150983cd24fb0d6963f7d28e17f72",
            "The MD5 of abc is a published test vector."
        );
    }

    #[test]
    fn names_the_zip_after_the_plugin_version() {
        assert_eq!(
            zip_file_name("0.1.0.0"),
            "smart-metadata_0.1.0.0.zip",
            "The zip name must match the one built by the plugin project."
        );
    }

    #[test]
    fn builds_a_manifest_in_the_jellyfin_catalog_format() -> Result<(), serde_json::Error> {
        let release = Release {
            repository: String::from("ljahier/jellyfin-smart-metadata"),
            tag: String::from("v0.1.0.0"),
        };

        let manifest = build_manifest(
            meta(),
            &release,
            b"abc",
            String::from("2026-11-15T08:57:14Z"),
        );

        assert_eq!(
            serde_json::to_value(&manifest)?,
            json!([{
                "guid": "3f7a9c1e-5b24-4d8a-a6f0-92c1d7e84b35",
                "name": "Smart Metadata",
                "description": "A long description.",
                "overview": "A short overview.",
                "owner": "ljahier",
                "category": "Metadata",
                "versions": [{
                    "version": "0.1.0.0",
                    "changelog": "Release v0.1.0.0",
                    "targetAbi": "12.4.0.0",
                    "sourceUrl": "https://github.com/ljahier/jellyfin-smart-metadata/releases/download/v0.1.0.0/smart-metadata_0.1.0.0.zip",
                    "checksum": "900150983cd24fb0d6963f7d28e17f72",
                    "timestamp": "2026-11-15T08:57:14Z"
                }]
            }]),
            "The manifest must use the field names that Jellyfin reads."
        );
        Ok(())
    }

    #[test]
    fn reads_the_real_plugin_metadata() -> Result<(), serde_json::Error> {
        let meta = serde_json::from_str::<PluginMeta>(include_str!("../../plugin/meta.json"))?;

        assert!(
            !meta.guid.is_empty(),
            "The plugin metadata must carry a guid."
        );
        Ok(())
    }
}
