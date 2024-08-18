use std::error::Error;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub async fn make_square_video(
    video_path: &Path,
    side_length: u32,
) -> Result<PathBuf, Box<dyn Error + Send>> {
    // Define the output path
    let output_path = video_path.with_extension("square.mp4");

    let filter = format!("[0:v]split=2[blur][vid];[blur]scale={}:{}:force_original_aspect_ratio=increase,crop={}:{},boxblur=luma_radius=min(h\\,w)/20:luma_power=1:chroma_radius=min(cw\\,ch)/20:chroma_power=1[bg];[vid]scale={}:{}:force_original_aspect_ratio=decrease[ov];[bg][ov]overlay=(W-w)/2:(H-h)/2", side_length, side_length, side_length, side_length, side_length, side_length);
    // Construct the ffmpeg command
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(video_path.to_str().unwrap())
        .arg("-vf")
        .arg(filter.as_str())
        .arg("-preset")
        .arg("fast")
        .arg(output_path.to_str().unwrap())
        .status()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

    // Check if the command was successful
    if !status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ffmpeg command failed",
        )));
    }

    // Return the path to the converted video
    Ok(output_path)
}
