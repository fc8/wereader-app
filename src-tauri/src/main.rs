#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
  tauri::Builder::default()
    .setup(|_app| {
      // Minimal setup. Restore platform-specific webview logic later if needed.
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}