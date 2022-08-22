#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde_json::json;
use tauri::{AppHandle, Manager};
use tokio::{io, net::UdpSocket, task};

#[tokio::main]
async fn main() {
  tauri::async_runtime::set(tokio::runtime::Handle::current());

  let app = tauri::Builder::default()
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

  app.run(|app_handle, event| match event {
    tauri::RunEvent::Ready => {
      let handle = app_handle.clone();
      task::spawn(async move {
        let result = run_recv_loop(handle).await;
        println!("run_recv_loop: {:?}", result)
      });
    }
    _ => {}
  });
}

async fn run_recv_loop(handle: AppHandle) -> io::Result<()> {
  let socket = UdpSocket::bind("0.0.0.0:25527").await?;

  loop {
    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf).await?;

    let raw_data = String::from_utf8_lossy(&buf[..len]);

    let mut end: bool = false;
    let data = raw_data
      .split("\n")
      .filter(|value| {
        end = end || value.ends_with("=end");
        value.starts_with("frame=")
          || value.starts_with("total_size=")
          || value.starts_with("out_time_ms=")
          || value.starts_with("dup_frames=")
          || value.starts_with("drop_frames=")
      })
      .collect::<Vec<&str>>()
      .join("\n");

    let result = handle.emit_all("ffmpeg_stats", json!({ "data": data, "end": end }));
    if let Err(err) = result {
      println!("failed to send event: {:?}", err);
    }
  }
}
