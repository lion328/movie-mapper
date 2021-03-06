use crate::error::*;

use std::process::Command;
use std::path::Path;
use std::str;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref FFPROBE_DURATION_REGEX: Regex = Regex::new("duration=(.*)").unwrap();
}

pub fn download_youtube_m4a_by_id<T: AsRef<Path>>(id: &str, path: T) -> Result<()> {
    download_youtube_m4a_impl(&format!("https://www.youtube.com/watch?v={}", id), path)
}

pub fn download_youtube_m4a_by_search<T: AsRef<Path>>(search: &str, path: T) -> Result<()> {
    download_youtube_m4a_impl(&format!("ytsearch1:{}", search), path)
}

fn download_youtube_m4a_impl<T: AsRef<Path>>(query: &str, path: T) -> Result<()> {
    let output = Command::new("youtube-dl")
        .args(&[
            "-x",
            "--audio-format",
            "m4a",
            "--audio-quality",
            "9",
            "-o",
        ])
        .arg(path.as_ref().with_extension("%(ext)s"))
        .arg(query)
        .output()?;
    
    std::fs::write(path.as_ref().with_extension("out.log"), &output.stdout)?;
    std::fs::write(path.as_ref().with_extension("err.log"), &output.stderr)?;

    if !output.status.success() {
        return Err(Error::YoutubeDL(str::from_utf8(&output.stderr).unwrap().to_owned()));
    }

    Ok(())
}

pub fn get_audio_length<T: AsRef<Path>>(path: T) -> Result<u64> {
    let output = Command::new("ffprobe")
        .arg("-show_format")
        .arg(path.as_ref().as_os_str())
        .output()?;
    
    if !output.status.success() {
        return Err(Error::YoutubeDL(str::from_utf8(&output.stderr).unwrap().to_owned()));
    }

    let stdout = str::from_utf8(&output.stdout).unwrap();
    let captures = FFPROBE_DURATION_REGEX.captures(stdout)
        .ok_or_else(|| Error::FFMPEG("missing duration from ffprobe".into()))?;

    let duration_s: f64 = captures.get(1).unwrap().as_str().parse().unwrap();
    let duration_ms: u64 = (duration_s * 1000.0) as u64;

    Ok(duration_ms)
}

pub fn make_spectrogram<T: AsRef<Path>>(path: T, (width, height): (usize, usize)) -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(path.as_ref().as_os_str())
        .args(&[
            "-filter_complex",
            &format!("showspectrumpic=legend=disabled:stop=16000:s={}x{}", width, height),
            "-y",
        ])
        .arg(path.as_ref().with_extension("jpg"))
        .output()?;
    
    if !output.status.success() {
        return Err(Error::FFMPEG(str::from_utf8(&output.stderr).unwrap().to_owned()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn youtube_duration() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("test.m4a");

        super::download_youtube_m4a_by_id("jNQXAC9IVRw", &path).unwrap();
        assert_eq!(18960, super::get_audio_length(&path).unwrap());

        tempdir.close().unwrap();
    }
}