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

    // 1. 設定工作路徑
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let db_path = current_dir.join("library.db");
    let data_dir = current_dir.join("data");
    let covers_dir = current_dir.join("covers");

    std::fs::create_dir_all(&covers_dir).ok();

    // 2. 初始化 SQLite 資料庫
    let is_new_db = !db_path.exists();
    println!("[-] 正在載入本機資料庫: {}", db_path.display());
    let db = init_db(&db_path)?;

    // 3. 若為新建立的資料庫且 data/ 目錄存在，自動匯入舊系統 DBF 資料
    if is_new_db && data_dir.exists() {
        println!("[-] 偵測到初次啟動與舊系統 data 目錄，自動進行舊資料庫無痛移轉...");
        match import_legacy_data(&db, &data_dir) {
            Ok(msg) => println!("[✔] 舊系統資料移轉完成: {}", msg),
            Err(e) => eprintln!("[!] 移轉過程發生警示: {}", e),
        }
    } else {
        println!("[✔] 本機資料庫載入就緒！");
    }

    // 4. 初始化網路書目補齊管理器
    let enricher = Arc::new(EnricherManager::new(covers_dir.clone()));

    // 5. 建立應用狀態與路由
    let state = AppState {
        db: db.clone(),
        data_dir,
        covers_dir,
        enricher,
    };

    let app = Router::new()
        .route("/", get(|| async { Html(HTML_PAGE) }))
        .merge(create_router(state))
        .layer(CorsLayer::permissive());

    // 6. 尋找可用連接埠並啟動伺服器
    let mut port = 3000;
    let listener = loop {
        let addr = format!("127.0.0.1:{}", port);
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => break l,
            Err(_) => {
                port += 1;
                if port > 3010 {
                    panic!("找不到可用的連接埠 (3000-3010)！");
                }
            }
        }
    };

    let url = format!("http://127.0.0.1:{}", port);
    println!("\n[✔] 圖書管理系統已啟動成功！");
    println!("👉 請於瀏覽器開啟: {}", url);
    println!("👉 自動為您開啟預設瀏覽器中...");
    println!("--------------------------------------------------------");
    println!("提示：此系統為單機獨立運作，所有書籍與借還資料均儲存在本地 library.db");
    println!("若欲關閉系統，直接關閉此終端視窗或按 Ctrl+C 即可。\n");

    // 自動開啟預設瀏覽器
    let _ = open::that(&url);

    // 啟動 HTTP 伺服器
    axum::serve(listener, app).await?;

    Ok(())
}
