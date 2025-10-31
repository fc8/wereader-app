#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 非调试模式下在 Windows 隐藏控制台窗口

use tauri::{ Manager, PhysicalSize }; // 引入 Manager trait（用于窗口管理）和 PhysicalSize（表示窗口尺寸）
use serde::{ Deserialize, Serialize }; // 引入 serde 的序列化/反序列化 trait
use std::fs; // 引入文件系统操作模块

// 为 SimpleWindowState 自动派生 serde 的序列化和反序列化能力
#[derive(Serialize, Deserialize)]
struct SimpleWindowState {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
}

impl Default for SimpleWindowState {
    fn default() -> Self {
        SimpleWindowState {
            width: 800,
            height: 1200,
            x: -1,
            y: -1,
        }
    }
}

impl SimpleWindowState {
    // 将当前状态保存到应用配置目录下的 window_state.json
    fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("获取配置目录失败: {}", e))?;

        fs
            ::create_dir_all(&config_dir) // 确保配置目录存在，必要时递归创建
            .map_err(|e| format!("创建配置目录失败: {}", e))?;

        let config_path = config_dir.join("window_state.json");
        let json = serde_json
            ::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(&config_path, json).map_err(|e| format!("写入配置文件失败: {}", e))?;

        println!("✅ 窗口大小已保存: {}x{}", self.width, self.height);
        Ok(())
    }

    // 从配置文件加载窗口状态，失败时返回默认值
    fn load(app_handle: &tauri::AppHandle) -> Self {
        let config_path = match app_handle.path().app_config_dir() {
            Ok(dir) => dir.join("window_state.json"), // 成功则拼接文件名
            Err(e) => {
                println!("❌ 获取配置目录失败: {}", e);
                return SimpleWindowState::default();
            }
        };

        println!("📁 尝试加载配置文件: {:?}", config_path);

        match fs::read_to_string(&config_path) {
            // 尝试读取文件内容为字符串
            Ok(content) => {
                match serde_json::from_str::<SimpleWindowState>(&content) {
                    Ok(state) => {
                        println!("✅ 成功加载窗口大小: {}x{}", state.width, state.height);
                        state // 返回解析得到的状态
                    }
                    Err(e) => {
                        println!("❌ 解析配置文件失败: {}", e);
                        SimpleWindowState::default() // 返回默认值
                    }
                }
            }
            Err(e) => {
                println!("❌ 读取配置文件失败: {}", e);
                SimpleWindowState::default()
            }
        }
    }
}

fn main() {
    tauri::Builder
        ::default()
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();

            // 加载保存的窗口大小
            let saved_size = SimpleWindowState::load(&app.handle()); // 从配置加载窗口大小
            println!("📏 设置窗口大小: {}x{}", saved_size.width, saved_size.height); // 打印要设置的大小

            if
                let Err(e) = main_window.set_size(
                    PhysicalSize::new(saved_size.width, saved_size.height)
                )
            {
                // 设置窗口尺寸
                println!("❌ 设置窗口大小失败: {}", e); // 如果失败打印错误
            }
            if saved_size.x >= 0 && saved_size.y >= 0 {
                if
                    let Err(_e) = main_window.set_position(
                        tauri::Position::Physical(tauri::PhysicalPosition {
                            x: saved_size.x,
                            y: saved_size.y,
                        })
                    )
                {
                    if let Err(e) = main_window.center() {
                        // 尝试将窗口居中
                        println!("❌ 居中窗口失败: {}", e); // 居中失败打印错误
                    }
                }
            } else {
                let _ = main_window.center();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // 监听窗口事件
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // 当窗口被请求关闭时触发
                let app_handle = window.app_handle(); // 获取应用句柄

                // 保存当前窗口大小
                if let Ok(size) = window.inner_size() {
                    // 获取窗口内部大小
                    let position = window.inner_position().unwrap();
                    let state = SimpleWindowState {
                        width: size.width, // 取当前宽度
                        height: size.height, // 取当前高度
                        x: position.x,
                        y: position.y,
                    };

                    if let Err(e) = state.save(&app_handle) {
                        // 保存状态到配置文件
                        println!("❌ 保存窗口大小失败: {}", e); // 保存失败打印错误
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
