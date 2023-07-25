use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io, net::SocketAddrV4, path::PathBuf};
use thiserror::Error;
use tokio::fs;
use url::Url;

use crate::sdp::BitDepth;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub raw_sdp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub local_sdp_file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sdp_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub custom_stream: Option<CustomStreamSettings>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomStreamSettings {
    pub multicast_address: SocketAddrV4,
    pub bit_depth: BitDepth,
    pub channels: u16,
    pub sample_rate: u32,
    pub packet_time: f32,
}

pub async fn load_presets() -> PresetResult<HashMap<String, Preset>> {
    if let Some(base_dirs) = directories::BaseDirs::new() {
        let mut configs = HashMap::new();
        let config_dir = base_dirs.config_dir();
        let app_config_dir = config_dir.join(env!("CARGO_PKG_NAME"));
        fs::create_dir_all(&app_config_dir).await?;
        let presets_file = app_config_dir.join("presets.yml");
        if presets_file.exists() {
            let data = fs::read(&presets_file).await?;
            let presets: Vec<Preset> = serde_yaml::from_slice(&data)?;
            for preset in presets {
                configs.insert(preset.name.clone(), preset);
            }
        }
        Ok(configs)
    } else {
        Err(PresetError::NoConfigDir)
    }
}

pub async fn save_preset(preset: Preset) -> PresetResult<()> {
    if let Some(base_dirs) = directories::BaseDirs::new() {
        let config_dir = base_dirs.config_dir();
        let app_config_dir = config_dir.join(env!("CARGO_PKG_NAME"));
        fs::create_dir_all(&app_config_dir).await?;
        let presets_file = app_config_dir.join("presets.yml");
        let mut existing_presets = load_presets().await?;
        existing_presets.insert(preset.name.clone(), preset.clone());
        let preset_list: Vec<Preset> = existing_presets.values().map(ToOwned::to_owned).collect();
        let yaml = serde_yaml::to_string(&preset_list)?;
        fs::write(presets_file, yaml).await?;
        log::info!("Successfully saved preset '{}'", preset.name);
        Ok(())
    } else {
        Err(PresetError::NoConfigDir)
    }
}

#[derive(Error, Debug)]
pub enum PresetError {
    #[error("no config dir found")]
    NoConfigDir,
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("yaml error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

pub type PresetResult<T> = Result<T, PresetError>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_presets() {
        let yaml = include_str!("../presets/presets.yml");
        let presets: Vec<Preset> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(2, presets.len());
    }
}
