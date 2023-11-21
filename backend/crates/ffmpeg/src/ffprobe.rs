#![allow(dead_code)]

use std::{
    collections::HashMap,
    error,
    fmt::{self, Debug},
    io, num,
    path::Path,
    time,
};

use config::CONFIG;
use serde::{Deserialize, Serialize};
use tokio::process;

pub async fn ffprobe(path: impl AsRef<Path> + Debug) -> Result<FfProbeResult, FfProbeError> {
    ffprobe_configured(path, Config::default()).await
}

#[tracing::instrument]
pub async fn ffprobe_configured(
    path: impl AsRef<Path> + Debug,
    config: Config,
) -> Result<FfProbeResult, FfProbeError> {
    let path = path.as_ref();

    let ffprobe_path = CONFIG
        .dependencies
        .ffprobe_path
        .as_ref()
        .ok_or_else(|| FfProbeError::MissingBinary("ffprobe".to_string()))?;

    logger::trace!(?ffprobe_path, "Using ffprobe binary");

    let mut cmd = process::Command::new(ffprobe_path);
    {
        cmd.args(["-v", "quiet"])
            .args(["-print_format", "json=c=1"])
            .arg("-show_format");

        if config.with_streams {
            cmd.arg("-show_streams");
        }

        if config.count_frames {
            cmd.arg("-count_frames");
        }

        cmd.arg(path);
    }

    logger::debug!(?cmd, "Running ffprobe");

    let out = cmd.output().await.map_err(FfProbeError::Io)?;

    logger::trace!(?out, "ffprobe output");

    if !out.status.success() {
        return Err(FfProbeError::Status(out));
    }

    serde_json::from_slice::<FfProbeResult>(&out.stdout).map_err(FfProbeError::Deserialize)
}

/// ffprobe configuration.
///
/// Use [`Config::builder`] for constructing a new config.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    count_frames: bool,
    with_streams: bool,
}

impl Config {
    /// Construct a new `ConfigBuilder`.
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Build the ffprobe configuration.
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Config {
                count_frames: false,
                with_streams: true,
            },
        }
    }

    /// Enable the -`count_frames` setting.
    /// Will fully decode the file and count the frames.
    /// Frame count will be available in [`Stream::nb_read_frames`].
    #[must_use]
    pub fn count_frames(mut self, count_frames: bool) -> Self {
        self.config.count_frames = count_frames;
        self
    }

    /// Enable the -`show_streams` setting.
    /// Will also show the streams.
    #[must_use]
    pub fn with_streams(mut self, with_streams: bool) -> Self {
        self.config.with_streams = with_streams;
        self
    }

    /// Finalize the builder into a [`Config`].
    #[must_use]
    pub fn build(self) -> Config {
        self.config
    }

    /// Run ffprobe with the config produced by this builder.
    pub async fn run(self, path: impl AsRef<Path> + Debug) -> Result<FfProbeResult, FfProbeError> {
        ffprobe_configured(path, self.config).await
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum FfProbeError {
    Io(io::Error),
    Status(std::process::Output),
    Deserialize(serde_json::Error),
    MissingBinary(String),
}

impl fmt::Display for FfProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfProbeError::Io(e) => write!(f, "I/O error: {e}"),
            FfProbeError::Status(o) => {
                write!(
                    f,
                    "ffprobe exited with status code {}: {}",
                    o.status,
                    String::from_utf8_lossy(&o.stderr)
                )
            }
            FfProbeError::Deserialize(e) => write!(f, "Deserialization error: {e}"),
            FfProbeError::MissingBinary(e) => write!(f, "Missing binary: {e}"),
        }
    }
}

impl error::Error for FfProbeError {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "__internal_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct FfProbeResult {
    pub streams: Option<Vec<Stream>>,
    pub format: Option<Format>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "__internal_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub index: i64,
    pub codec_name: Option<String>,
    pub sample_aspect_ratio: Option<String>,
    pub display_aspect_ratio: Option<String>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub bits_per_raw_sample: Option<String>,
    pub channel_layout: Option<String>,
    pub max_bit_rate: Option<String>,
    pub nb_frames: Option<String>,
    /// Number of frames seen by the decoder.
    /// Requires full decoding and is only available if the 'count_frames'
    /// setting was enabled.
    pub nb_read_frames: Option<String>,
    pub codec_long_name: Option<String>,
    pub codec_type: Option<String>,
    pub codec_time_base: Option<String>,
    pub codec_tag_string: Option<String>,
    pub codec_tag: Option<String>,
    pub sample_fmt: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<i64>,
    pub bits_per_sample: Option<i64>,
    pub r_frame_rate: Option<String>,
    pub avg_frame_rate: Option<String>,
    pub time_base: Option<String>,
    pub start_pts: Option<i64>,
    pub start_time: Option<String>,
    pub duration_ts: Option<i64>,
    pub duration: Option<String>,
    pub bit_rate: Option<String>,
    pub disposition: Option<Disposition>,
    pub tags: Option<StreamTags>,
    pub profile: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub coded_width: Option<i64>,
    pub coded_height: Option<i64>,
    pub closed_captions: Option<i64>,
    pub has_b_frames: Option<i64>,
    pub pix_fmt: Option<String>,
    pub level: Option<i64>,
    pub chroma_location: Option<String>,
    pub refs: Option<i64>,
    pub is_avc: Option<String>,
    pub nal_length: Option<String>,
    pub nal_length_size: Option<String>,
    pub field_order: Option<String>,
    pub id: Option<String>,
    #[serde(default)]
    pub side_data_list: Vec<SideData>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "__internal_deny_unknown_fields", serde(deny_unknown_fields))]
// Allowed to prevent having to break compatibility of float fields are added.
#[allow(clippy::derive_partial_eq_without_eq)]
#[serde(rename_all = "camelCase")]
pub struct SideData {
    pub side_data_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "__internal_deny_unknown_fields", serde(deny_unknown_fields))]
// Allowed to prevent having to break compatibility of float fields are added.
#[allow(clippy::derive_partial_eq_without_eq)]
#[serde(rename_all = "camelCase")]
pub struct Disposition {
    pub default: i64,
    pub dub: i64,
    pub original: i64,
    pub comment: i64,
    pub lyrics: i64,
    pub karaoke: i64,
    pub forced: i64,
    pub hearing_impaired: Option<i64>,
    pub visual_impaired: Option<i64>,
    pub clean_effects: Option<i64>,
    pub attached_pic: Option<i64>,
    pub timed_thumbnails: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "__internal_deny_unknown_fields", serde(deny_unknown_fields))]
// Allowed to prevent having to break compatibility of float fields are added.
#[allow(clippy::derive_partial_eq_without_eq)]
#[serde(rename_all = "camelCase")]
pub struct StreamTags {
    pub language: Option<String>,
    pub creation_time: Option<String>,
    pub handler_name: Option<String>,
    pub encoder: Option<String>,
    pub timecode: Option<String>,
    pub reel_name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "__internal_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Format {
    pub filename: Option<String>,
    pub nb_streams: Option<i64>,
    pub nb_programs: Option<i64>,
    pub format_name: Option<String>,
    pub format_long_name: Option<String>,
    pub start_time: Option<String>,
    pub duration: Option<String>,
    pub size: Option<String>,
    pub bit_rate: Option<String>,
    pub probe_score: Option<i64>,
    pub tags: Option<FormatTags>,
}

impl Format {
    /// Get the duration parsed into a [`std::time::Duration`].
    #[must_use]
    pub fn try_get_duration(&self) -> Option<Result<time::Duration, num::ParseFloatError>> {
        self.duration
            .as_ref()
            .map(|duration| match duration.parse::<f64>() {
                Ok(num) => Ok(time::Duration::from_secs_f64(num)),
                Err(error) => Err(error),
            })
    }

    /// Get the duration parsed into a [`std::time::Duration`].
    ///
    /// Will return [`None`] if no duration is available, or if parsing fails.
    /// See [`Self::try_get_duration`] for a method that returns an error.
    #[must_use]
    pub fn get_duration(&self) -> Option<time::Duration> {
        self.try_get_duration()?.ok()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "__internal_deny_unknown_fields", serde(deny_unknown_fields))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[serde(rename_all = "camelCase")]
pub struct FormatTags {
    #[serde(rename = "WMFSDKNeeded")]
    pub wmf_sdk_needed: Option<String>,
    #[serde(rename = "WMFSDKVersion")]
    pub wmf_sdk_version: Option<String>,
    #[serde(rename = "DeviceConformanceTemplate")]
    pub device_conformance_template: Option<String>,
    #[serde(rename = "IsVBR")]
    pub is_vbr: Option<String>,
    pub major_brand: Option<String>,
    pub minor_version: Option<String>,
    pub compatible_brands: Option<String>,
    pub creation_time: Option<String>,
    pub encoder: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
