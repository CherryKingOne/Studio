use std::path::Path;

use crate::error::AppError;

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
