use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::State;

struct Shared {
    ffmpeg: Mutex<Option<Child>>,
}

impl Drop for Shared {
    fn drop(&mut self) {
        if let Ok(slot) = self.ffmpeg.get_mut() {
            if let Some(mut child) = slot.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

fn find_ffmpeg() -> String {
    std::env::var("LITESTREAM_FFMPEG").unwrap_or_else(|_| "ffmpeg".to_string())
}

fn valid_target(target: &str) -> bool {
    (target.starts_with("rtmp://") || target.starts_with("rtmps://"))
        && target.len() <= 2048
        && !target.chars().any(char::is_whitespace)
}

#[tauri::command]
fn start_rtmp(target: String, fps: u32, bitrate_kbps: u32, state: State<'_, Shared>) -> Result<(), String> {
    if !valid_target(&target) {
        return Err("Enter a valid RTMP or RTMPS URL and stream key".into());
    }
    if !(15..=60).contains(&fps) {
        return Err("Frame rate must be between 15 and 60 FPS".into());
    }
    if !(500..=12000).contains(&bitrate_kbps) {
        return Err("Bitrate must be between 500 and 12000 kbps".into());
    }
    let mut slot = state.ffmpeg.lock().map_err(|_| "RTMP state is unavailable")?;
    if slot.is_some() {
        return Err("An RTMP process is already running".into());
    }
    let gop = (fps * 2).to_string();
    let fps_arg = fps.to_string();
    let bitrate_arg = format!("{}k", bitrate_kbps);
    let buffer_arg = format!("{}k", bitrate_kbps * 2);
    let mut command = Command::new(find_ffmpeg());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let child = command
        .args([
            "-hide_banner", "-loglevel", "error", "-nostdin", "-fflags", "+genpts",
            "-thread_queue_size", "512", "-f", "webm", "-i", "pipe:0",
            "-map", "0:v:0", "-map", "0:a?", "-c:v", "libx264", "-preset", "veryfast",
            "-tune", "zerolatency", "-pix_fmt", "yuv420p", "-r", fps_arg.as_str(),
            "-b:v", bitrate_arg.as_str(), "-maxrate", bitrate_arg.as_str(),
            "-bufsize", buffer_arg.as_str(), "-g", gop.as_str(),
            "-c:a", "aac", "-b:a", "128k", "-ar", "48000", "-f", "flv", target.as_str(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not launch FFmpeg (install FFmpeg or set LITESTREAM_FFMPEG): {error}"))?;
    *slot = Some(child);
    Ok(())
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    fn value(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    if bytes.len() > 24 * 1024 * 1024 || bytes.len() % 4 != 0 {
        return Err("Invalid or oversized media chunk".into());
    }
    let mut output = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks_exact(4) {
        let a = value(chunk[0]).ok_or("Invalid base64 media chunk")? as u32;
        let b = value(chunk[1]).ok_or("Invalid base64 media chunk")? as u32;
        let c = if chunk[2] == b'=' { 0 } else { value(chunk[2]).ok_or("Invalid base64 media chunk")? as u32 };
        let d = if chunk[3] == b'=' { 0 } else { value(chunk[3]).ok_or("Invalid base64 media chunk")? as u32 };
        let word = (a << 18) | (b << 12) | (c << 6) | d;
        output.push((word >> 16) as u8);
        if chunk[2] != b'=' { output.push((word >> 8) as u8); }
        if chunk[3] != b'=' { output.push(word as u8); }
    }
    Ok(output)
}

#[tauri::command]
fn write_rtmp_chunk(chunk_base64: String, state: State<'_, Shared>) -> Result<(), String> {
    let data = decode_base64(&chunk_base64)?;
    let mut slot = state.ffmpeg.lock().map_err(|_| "RTMP state is unavailable")?;
    let child = slot.as_mut().ok_or("RTMP is not running")?;
    let stdin: &mut ChildStdin = child.stdin.as_mut().ok_or("FFmpeg input pipe is closed")?;
    stdin.write_all(&data).map_err(|error| format!("Could not write to FFmpeg: {error}"))
}

#[tauri::command]
fn stop_rtmp(state: State<'_, Shared>) -> Result<(), String> {
    let mut slot = state.ffmpeg.lock().map_err(|_| "RTMP state is unavailable")?;
    let Some(mut child) = slot.take() else { return Ok(()); };
    drop(child.stdin.take());
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            _ => { let _ = child.kill(); let _ = child.wait(); return Ok(()); }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Shared { ffmpeg: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![start_rtmp, write_rtmp_chunk, stop_rtmp])
        .run(tauri::generate_context!())
        .expect("error while running LiteStream");
}
