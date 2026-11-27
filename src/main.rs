#![windows_subsystem = "windows"]

mod models;
mod legacy;
mod db;
mod importer;
mod enricher;
mod api;
mod frontend;

use std::path::PathBuf;
use std::sync::Arc;
use axum::{
    response::Html,
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use crate::api::{create_router, AppState};
use crate::db::{init_db, import_legacy_data};
use crate::enricher::EnricherManager;
use crate::frontend::HTML_PAGE;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================================");
    println!("     📚 現代化單機版圖書管理借還系統 (Rust 核心)");
    println!("========================================================");

    let is_post_update = std::env::args().any(|a| a == "--post-update");

    // 1. 設定工作路徑並清理上次自動更新殘留的 .old / .new 暫存檔
    let current_dir = if let Ok(exe_path) = std::env::current_exe() {
        let mut old_path = exe_path.as_os_str().to_os_string();
        old_path.push(".old");
        let old_path = PathBuf::from(old_path);
        let mut new_path = exe_path.as_os_str().to_os_string();
        new_path.push(".new");
        let new_path = PathBuf::from(new_path);

        if is_post_update {
            // 等待舊程式結束並釋放連接埠與檔案鎖定
            for _ in 0..20 {
                if old_path.exists() && std::fs::remove_file(&old_path).is_ok() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        } else {
            let _ = std::fs::remove_file(&old_path);
        }
        let _ = std::fs::remove_file(&new_path);

        if let Some(parent) = exe_path.parent() {
            if parent.join("library.db").exists() || parent.join("data").exists() {
                parent.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_else(|_| parent.to_path_buf())
            }
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    // 2. 檢查是否已在運行中，避免重複執行導致 port 號一直遞增
    if is_post_update {
        for _ in 0..20 {
            if find_running_instance(&current_dir).await.is_none() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    } else if let Some(active_port) = find_running_instance(&current_dir).await {
        let url = format!("http://localhost:{}", active_port);
        println!("\n[✔] 偵測到圖書管理系統已在運行中 (連接埠: {})！", active_port);
        println!("👉 自動為您開啟系統畫面: {}", url);
        println!("--------------------------------------------------------");
        let _ = open::that(&url);
        return Ok(());
    }

    let db_path = current_dir.join("library.db");
    let data_dir = current_dir.join("data");
    let covers_dir = current_dir.join("covers");

    std::fs::create_dir_all(&covers_dir).ok();

    // 3. 初始化 SQLite 資料庫
    let is_new_db = !db_path.exists();
    println!("[-] 正在載入本機資料庫: {}", db_path.display());
    let db = init_db(&db_path)?;

    // 4. 若為新建立的資料庫且 data/ 目錄存在，自動匯入舊系統 DBF 資料
    if is_new_db && data_dir.exists() {
        println!("[-] 偵測到初次啟動與舊系統 data 目錄，正在建立資料庫並移轉舊版 DBF 資料...");
        println!("[⏳] 讀取舊版 DBF 檔案（含書籍、讀者與歷史借閱紀錄）可能需要較長時間，請耐心等待，切勿關閉視窗！");
        match import_legacy_data(&db, &data_dir, false) {
            Ok(msg) => println!("[✔] 舊系統資料移轉完成: {}", msg),
            Err(e) => eprintln!("[!] 移轉過程發生警示: {}", e),
        }
    } else {
        println!("[✔] 本機資料庫載入就緒！");
    }

    // 5. 初始化網路書目補齊管理器
    let enricher = Arc::new(EnricherManager::new(covers_dir.clone()));

    // 6. 建立應用狀態與路由
    let state = AppState {
        db: db.clone(),
        db_path,
        data_dir,
        covers_dir,
        enricher,
    };

    let app = Router::new()
        .route("/", get(|| async { Html(HTML_PAGE) }))
        .merge(create_router(state))
        .layer(CorsLayer::permissive());

    // 7. 尋找可用連接埠並啟動伺服器（綁定 0.0.0.0 支援本機與區網 IP 存取）
    let mut port = 3000;
    let listener = loop {
        let addr = format!("0.0.0.0:{}", port);
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => break l,
            Err(_) => {
                // 若綁定失敗，再次確認是否剛好有另一程序啟動了 LibRust
                if let Some(active_port) = find_running_instance(&current_dir).await {
                    let url = format!("http://localhost:{}", active_port);
                    println!("\n[✔] 偵測到圖書管理系統已由另一程序啟動 (連接埠: {})！", active_port);
                    println!("👉 自動為您開啟系統畫面: {}", url);
                    let _ = open::that(&url);
                    return Ok(());
                }
                port += 1;
                if port > 3010 {
                    panic!("找不到可用的連接埠 (3000-3010)！");
                }
            }
        }
    };

    let port_file = current_dir.join("library.port");
    let _ = std::fs::write(&port_file, port.to_string());

    let url = format!("http://localhost:{}", port);
    println!("\n[✔] 圖書管理系統已啟動成功 (v0.12)！");
    println!("👉 本機連線請於瀏覽器開啟: {}", url);
    println!("👉 區網其他裝置請連線: http://<本機IP>:{}", port);
    println!("👉 自動為您開啟預設瀏覽器中...");
    println!("--------------------------------------------------------");
    println!("提示：此系統為單機獨立運作，所有書籍與借還資料均儲存在本地 library.db");
    println!("若欲關閉系統，可在網頁右上角點擊「關閉系統」，或於右下角系統匣圖示按右鍵退出。\n");

    // 自動開啟預設瀏覽器（自動更新重啟時不重複開新分頁）
    if !is_post_update {
        let _ = open::that(&url);
    }

    // 8. 註冊 Windows 系統匣圖示 (System Tray)
    #[cfg(windows)]
    let _tray = match tray_item::TrayItem::new("圖書管理系統", tray_item::IconSource::Resource("app-icon")) {
        Ok(mut t) => {
            let _ = t.add_label("📚 圖書管理借還系統 (v0.12)");
            let url_open = url.clone();
            let _ = t.add_menu_item("開啟系統網頁", move || {
                let _ = open::that(&url_open);
            });
            let port_file_tray = port_file.clone();
            let _ = t.add_menu_item("關閉退出系統", move || {
                let _ = std::fs::remove_file(&port_file_tray);
                std::process::exit(0);
            });
            Some(t)
        }
        Err(e) => {
            eprintln!("[!] 系統匣圖示初始化略過: {}", e);
            None
        }
    };

    // 啟動 HTTP 伺服器
    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_port_is_librust(client: &reqwest::Client, port: u16) -> bool {
    let ping_url = format!("http://127.0.0.1:{}/api/system/ping", port);
    if let Ok(resp) = client.get(&ping_url).send().await {
        if resp.status().is_success() {
            if let Ok(text) = resp.text().await {
                if text.contains("librust") {
                    return true;
                }
            }
        }
    }

    // 相容性檢查 (舊版本可能無 /api/system/ping 端點)
    let auth_url = format!("http://127.0.0.1:{}/api/auth/status", port);
    if let Ok(resp) = client.get(&auth_url).send().await {
        if resp.status().is_success() {
            if let Ok(text) = resp.text().await {
                if text.contains("logged_in") {
                    return true;
                }
            }
        }
    }

    false
}

async fn find_running_instance(base_dir: &std::path::Path) -> Option<u16> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(300))
        .build()
        .ok()?;

    // 1. 優先檢查記錄在 library.port 檔案中的連接埠
    let port_file = base_dir.join("library.port");
    if let Ok(content) = std::fs::read_to_string(&port_file) {
        if let Ok(port) = content.trim().parse::<u16>() {
            if check_port_is_librust(&client, port).await {
                return Some(port);
            }
        }
    }

    // 2. 依序探測預設連接埠範圍 (3000..=3010)
    for port in 3000..=3010 {
        if check_port_is_librust(&client, port).await {
            let _ = std::fs::write(&port_file, port.to_string());
            return Some(port);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;

    #[tokio::test]
    async fn test_ping_and_instance_detection() {
        let app = Router::new().route("/api/system/ping", get(crate::api::handle_system_ping));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        // 稍候伺服器就緒
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(500))
            .build()
            .unwrap();

        // 測試在線 port
        assert!(check_port_is_librust(&client, port).await);

        // 測試離線 port
        assert!(!check_port_is_librust(&client, 59999).await);

        // 測試由 library.port 檔案偵測
        let temp_dir = std::env::temp_dir().join(format!("librust_test_{}", port));
        let _ = std::fs::create_dir_all(&temp_dir);
        let port_file = temp_dir.join("library.port");
        std::fs::write(&port_file, port.to_string()).unwrap();

        let found = find_running_instance(&temp_dir).await;
        assert_eq!(found, Some(port));

        let _ = std::fs::remove_dir_all(&temp_dir);
        server.abort();
    }
}
