use std::path::Path;
use std::time::Duration;

use crate::runtime::error::AppError;

pub struct SilenceSpec<'a> {
    pub ffmpeg_path: &'a str,
    pub output: &'a Path,
    pub duration_ms: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate_kbps: u32,
}

pub struct MergeSpec<'a> {
    pub ffmpeg_path: &'a str,
    pub concat_file: &'a Path,
    pub output: &'a Path,
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate_kbps: u32,
    pub loudness_i: f32,
    pub loudness_tp: f32,
    pub loudness_lra: f32,
    pub title: &'a str,
    pub artist: &'a str,
    pub album: &'a str,
    pub track: u32,
}

pub async fn generate_silence(spec: &SilenceSpec<'_>) -> Result<(), AppError> {
    let seconds = (spec.duration_ms as f64 / 1000.0).max(0.05);
    let layout = if spec.channels == 1 { "mono" } else { "stereo" };
    let mut command = tokio::process::Command::new(spec.ffmpeg_path);
    command
        .arg("-y")
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg(format!("anullsrc=r={}:cl={layout}", spec.sample_rate))
        .arg("-t")
        .arg(format!("{seconds:.3}"))
        .arg("-c:a")
        .arg("libmp3lame")
        .arg("-b:a")
        .arg(format!("{}k", spec.bitrate_kbps))
        .arg(spec.output);
    command.kill_on_drop(true);

    let output = tokio::time::timeout(Duration::from_secs(60), command.output())
        .await
        .map_err(|_| AppError::AudioProcess("生成静音超时".to_string()))?
        .map_err(|error| AppError::AudioProcess(error.to_string()))?;
    if !output.status.success() {
        return Err(AppError::AudioProcess(
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(300)
                .collect(),
        ));
    }
    Ok(())
}

pub async fn merge_with_loudnorm(spec: &MergeSpec<'_>) -> Result<(), AppError> {
    let layout = if spec.channels == 1 { "1" } else { "2" };
    let loudnorm = format!(
        "loudnorm=I={}:TP={}:LRA={}",
        spec.loudness_i, spec.loudness_tp, spec.loudness_lra
    );
    let mut command = tokio::process::Command::new(spec.ffmpeg_path);
    command
        .arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(spec.concat_file)
        .arg("-af")
        .arg(loudnorm)
        .arg("-ar")
        .arg(spec.sample_rate.to_string())
        .arg("-ac")
        .arg(layout)
        .arg("-c:a")
        .arg("libmp3lame")
        .arg("-b:a")
        .arg(format!("{}k", spec.bitrate_kbps))
        .arg("-metadata")
        .arg(format!("title={}", spec.title))
        .arg("-metadata")
        .arg(format!("artist={}", spec.artist))
        .arg("-metadata")
        .arg(format!("album={}", spec.album))
        .arg("-metadata")
        .arg(format!("track={}", spec.track))
        .arg(spec.output);
    command.kill_on_drop(true);

    let output = tokio::time::timeout(Duration::from_secs(600), command.output())
        .await
        .map_err(|_| AppError::AudioProcess("章节拼接超时".to_string()))?
        .map_err(|error| AppError::AudioProcess(error.to_string()))?;
    if !output.status.success() {
        return Err(AppError::AudioProcess(
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(2000)
                .collect(),
        ));
    }
    Ok(())
}

pub async fn probe_duration_ms(
    ffprobe_path: &str,
    tmp_dir: &str,
    bytes: &[u8],
) -> Result<u32, AppError> {
    let dir = Path::new(tmp_dir);
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|error| AppError::AudioProcess(error.to_string()))?;

    let path = dir.join(format!("probe-{}.mp3", uuid::Uuid::now_v7()));
    tokio::fs::write(&path, bytes)
        .await
        .map_err(|error| AppError::AudioProcess(error.to_string()))?;

    let output = tokio::process::Command::new(ffprobe_path)
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(&path)
        .output()
        .await;

    let _ = tokio::fs::remove_file(&path).await;

    let output = output.map_err(|error| AppError::AudioProcess(error.to_string()))?;
    if !output.status.success() {
        return Err(AppError::AudioProcess(
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(300)
                .collect(),
        ));
    }

    let seconds = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|error| AppError::AudioProcess(format!("解析音频时长失败: {error}")))?;
    Ok((seconds * 1000.0).round() as u32)
}
