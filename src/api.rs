use axum::{
    extract::{Path, Query, State, Multipart},
    response::{Json, IntoResponse},
    http::{header, StatusCode},
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use crate::db::{*, get_next_reader_id, get_next_book_id};
use crate::models::*;
use crate::importer::*;
use crate::enricher::EnricherManager;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub data_dir: PathBuf,
    pub covers_dir: PathBuf,
    pub enricher: Arc<EnricherManager>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct IsbnLookupRequest {
    pub isbn: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSettings {
    pub library_name: String,
    pub library_phone: String,
    pub library_address: String,
    #[serde(default)]
    pub borrow_rules: String,
    pub barcode_width_mm: f64,
    pub barcode_height_mm: f64,
    pub show_spine_tag: bool,
    pub card_margin_mm: f64,
    pub requests_per_minute: u32,
    pub jitter_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub username: String,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // 登入認證
        .route("/api/auth/login", post(handle_login))
        .route("/api/auth/logout", post(handle_logout))
        .route("/api/auth/status", get(handle_auth_status))
        .route("/api/auth/change-password", post(handle_change_password))
        // 角色身分別
        .route("/api/roles", get(list_roles).post(create_or_update_role))
        .route("/api/roles/:id", delete(remove_role))
        // 讀者
        .route("/api/readers", get(list_readers).post(create_reader_handler))
        .route("/api/readers/next-id", get(get_next_reader_id_handler))
        .route("/api/readers/:id", get(get_reader).put(update_reader_handler).delete(remove_reader))
        // 書籍
        .route("/api/books", get(list_books).post(create_book_handler))
        .route("/api/books/next-id", get(get_next_book_id_handler))
        .route("/api/books/:id", get(get_book).put(update_book_handler).delete(remove_book))
        .route("/api/books/lookup-isbn", post(lookup_isbn))
        // 系統與翻譯
        .route("/api/system/translate", post(handle_translate))
        // 借還流通
        .route("/api/circulation/borrow", post(handle_borrow))
        .route("/api/circulation/return", post(handle_return))
        .route("/api/circulation/renew", post(handle_renew))
        .route("/api/circulation/active", get(list_active_loans))
        .route("/api/circulation/history", get(list_loan_history))
        // 統計與系統
        .route("/api/stats", get(get_stats_summary))
        .route("/api/settings", get(get_system_settings).post(save_system_settings))
        .route("/api/system/import-legacy", post(trigger_legacy_import))
        .route("/api/system/shutdown", post(handle_system_shutdown))
        .route("/api/system/ping", get(handle_system_ping))
        // 圖示與網站 Favicon
        .route("/favicon.ico", get(serve_favicon))
        .route("/app.png", get(serve_app_png))
        // 匯入與範本
        .route("/api/import/books", post(handle_import_books))
        .route("/api/import/readers", post(handle_import_readers))
        .route("/api/templates/books.csv", get(download_books_template))
        .route("/api/templates/readers.csv", get(download_readers_template))
        // 封面圖片靜態路由
        .route("/covers/:filename", get(serve_cover_image))
        .with_state(state)
}

// === 角色 ===

async fn list_roles(State(state): State<AppState>) -> Json<ApiResponse<Vec<Role>>> {
    match get_roles(&state.db) {
        Ok(roles) => Json(ApiResponse::ok("查詢成功", roles)),
        Err(e) => Json(ApiResponse::err(format!("查詢身分別失敗: {}", e))),
    }
}

async fn create_or_update_role(
    State(state): State<AppState>,
    Json(role): Json<Role>,
) -> Json<ApiResponse<Role>> {
    if role.id.trim().is_empty() || role.name.trim().is_empty() {
        return Json(ApiResponse::err("身分代號與名稱不可為空！"));
    }
    match save_role(&state.db, &role) {
        Ok(_) => Json(ApiResponse::ok("儲存身分別成功", role)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn remove_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    match delete_role(&state.db, &id) {
        Ok(_) => Json(ApiResponse::ok("身分別已刪除", id)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

// === 讀者 ===

async fn list_readers(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<Vec<Reader>>> {
    let q = query.q.unwrap_or_default();
    match get_readers(&state.db, &q) {
        Ok(readers) => Json(ApiResponse::ok("查詢成功", readers)),
        Err(e) => Json(ApiResponse::err(format!("查詢讀者失敗: {}", e))),
    }
}

async fn get_reader(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Option<Reader>>> {
    match get_reader_by_id(&state.db, &id) {
        Ok(Some(r)) => Json(ApiResponse::ok("查詢成功", Some(r))),
        Ok(None) => Json(ApiResponse::err(format!("查無讀者證號 [{}]", id))),
        Err(e) => Json(ApiResponse::err(format!("讀取失敗: {}", e))),
    }
}

async fn create_reader_handler(
    State(state): State<AppState>,
    Json(reader): Json<Reader>,
) -> Json<ApiResponse<Reader>> {
    if reader.name.trim().is_empty() {
        return Json(ApiResponse::err("讀者姓名不可為空！"));
    }
    match create_reader(&state.db, reader) {
        Ok(saved) => Json(ApiResponse::ok("新增讀者成功", saved)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn update_reader_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut reader): Json<Reader>,
) -> Json<ApiResponse<Reader>> {
    reader.id = id;
    if reader.id.trim().is_empty() || reader.name.trim().is_empty() {
        return Json(ApiResponse::err("讀者證號與姓名不可為空！"));
    }
    match update_reader(&state.db, &reader) {
        Ok(_) => Json(ApiResponse::ok("更新讀者成功", reader)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn remove_reader(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    match delete_reader(&state.db, &id) {
        Ok(_) => Json(ApiResponse::ok("讀者已刪除", id)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn get_next_reader_id_handler(
    State(state): State<AppState>,
) -> Json<ApiResponse<String>> {
    match get_next_reader_id(&state.db) {
        Ok(next_id) => Json(ApiResponse::ok("取得建議證號成功", next_id)),
        Err(e) => Json(ApiResponse::err(format!("取得建議讀者證號失敗: {}", e))),
    }
}

// === 書籍 ===

async fn list_books(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<Vec<Book>>> {
    let q = query.q.unwrap_or_default();
    let status = query.status.unwrap_or_default();
    match get_books(&state.db, &q, &status) {
        Ok(books) => Json(ApiResponse::ok("查詢成功", books)),
        Err(e) => Json(ApiResponse::err(format!("查詢書籍失敗: {}", e))),
    }
}

async fn get_next_book_id_handler(
    State(state): State<AppState>,
) -> Json<ApiResponse<String>> {
    match get_next_book_id(&state.db) {
        Ok(next_id) => Json(ApiResponse::ok("取得建議條碼成功", next_id)),
        Err(e) => Json(ApiResponse::err(format!("取得建議書籍條碼失敗: {}", e))),
    }
}

async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Option<Book>>> {
    match get_book_by_id(&state.db, &id) {
        Ok(Some(b)) => Json(ApiResponse::ok("查詢成功", Some(b))),
        Ok(None) => Json(ApiResponse::err(format!("查無書籍條碼 [{}]", id))),
        Err(e) => Json(ApiResponse::err(format!("讀取失敗: {}", e))),
    }
}

async fn create_book_handler(
    State(state): State<AppState>,
    Json(book): Json<Book>,
) -> Json<ApiResponse<Book>> {
    if book.title.trim().is_empty() {
        return Json(ApiResponse::err("書名不可為空！"));
    }
    match create_book(&state.db, book) {
        Ok(saved) => Json(ApiResponse::ok("新增書籍成功", saved)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn update_book_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut book): Json<Book>,
) -> Json<ApiResponse<Book>> {
    book.id = id;
    if book.id.trim().is_empty() || book.title.trim().is_empty() {
        return Json(ApiResponse::err("書籍條碼與書名不可為空！"));
    }
    match update_book(&state.db, &book) {
        Ok(_) => Json(ApiResponse::ok("更新書籍成功", book)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn remove_book(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    match delete_book(&state.db, &id) {
        Ok(_) => Json(ApiResponse::ok("書籍已刪除", id)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

// === ISBN 網路查詢與翻譯 ===

#[derive(Serialize)]
pub struct LookupResult {
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub pub_year: String,
    pub description: String,
    pub cover_url: String,
    pub source: String,
    pub is_chinese: bool,
}

#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct TranslateResponse {
    pub original: String,
    pub translated: String,
}

async fn handle_translate(
    State(state): State<AppState>,
    Json(payload): Json<TranslateRequest>,
) -> Json<ApiResponse<TranslateResponse>> {
    let clean = payload.text.trim();
    if clean.is_empty() {
        return Json(ApiResponse::ok("內容為空", TranslateResponse {
            original: payload.text,
            translated: "".to_string(),
        }));
    }

    // 關鍵判斷：如果書名/文字是漢語拼音音譯（如 "Man hua San guo zhi"），走拼音轉繁體中文
    if EnricherManager::looks_like_chinese_pinyin(clean) {
        if let Ok(zh_text) = state.enricher.pinyin_to_traditional_chinese(clean).await {
            if !zh_text.trim().is_empty() {
                return Json(ApiResponse::ok("拼音音譯還原中文成功", TranslateResponse {
                    original: payload.text,
                    translated: zh_text,
                }));
            }
        }
    }

    // 一般外語（英語、日語等）走常規機器翻譯
    match state.enricher.translate_to_traditional_chinese(clean).await {
        Ok(translated) => Json(ApiResponse::ok("翻譯成功", TranslateResponse {
            original: payload.text,
            translated,
        })),
        Err(e) => Json(ApiResponse::err(format!("翻譯失敗: {}", e))),
    }
}

async fn lookup_isbn(
    State(state): State<AppState>,
    Json(payload): Json<IsbnLookupRequest>,
) -> Json<ApiResponse<Option<LookupResult>>> {
    match state.enricher.fetch_book_by_isbn(&payload.isbn).await {
        Ok(Some(info)) => {
            // 若有圖片，下載並存至 covers 目錄
            let mut local_cover_url = "".to_string();
            if let Some(bytes) = info.cover_image_bytes {
                let filename = format!("{}.jpg", payload.isbn.replace('-', ""));
                let save_path = state.covers_dir.join(&filename);
                if std::fs::write(&save_path, &bytes).is_ok() {
                    local_cover_url = format!("/covers/{}", filename);
                }
            } else if let Some(u) = info.cover_url {
                local_cover_url = u;
            }

            let msg = format!("已由【{}】成功獲取書目資訊！", info.source);
            Json(ApiResponse::ok(msg, Some(LookupResult {
                title: info.title.unwrap_or_default(),
                author: info.author.unwrap_or_default(),
                publisher: info.publisher.unwrap_or_default(),
                pub_year: info.pub_year.unwrap_or_default(),
                description: info.description.unwrap_or_default(),
                cover_url: local_cover_url,
                source: info.source,
                is_chinese: info.is_chinese,
            })))
        }
        Ok(None) => Json(ApiResponse::err("查無此 ISBN 對應之網路書目資料")),
        Err(e) => Json(ApiResponse::err(format!("網路連線或查詢失敗: {}", e))),
    }
}

// === 借還流通 ===

async fn handle_borrow(
    State(state): State<AppState>,
    Json(payload): Json<BorrowRequest>,
) -> Json<ApiResponse<LoanRecord>> {
    let rid = payload.reader_id.trim();
    let bid = payload.book_id.trim();

    if rid.is_empty() || bid.is_empty() {
        return Json(ApiResponse::err("請輸入讀者證號與書籍條碼！"));
    }

    match borrow_book(&state.db, rid, bid) {
        Ok(loan) => Json(ApiResponse::ok("借閱成功！", loan)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn handle_return(
    State(state): State<AppState>,
    Json(payload): Json<ReturnRequest>,
) -> Json<ApiResponse<LoanRecord>> {
    match return_book(&state.db, payload.book_id.as_deref(), payload.loan_id) {
        Ok(loan) => Json(ApiResponse::ok("歸還手續完成！", loan)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

async fn handle_renew(
    State(state): State<AppState>,
    Json(payload): Json<RenewRequest>,
) -> Json<ApiResponse<LoanRecord>> {
    match renew_book(&state.db, payload.loan_id) {
        Ok(loan) => Json(ApiResponse::ok("續借成功！", loan)),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

#[derive(Deserialize)]
pub struct ActiveLoansQuery {
    pub reader_id: Option<String>,
}

async fn list_active_loans(
    State(state): State<AppState>,
    Query(query): Query<ActiveLoansQuery>,
) -> Json<ApiResponse<Vec<LoanRecord>>> {
    match get_active_loans(&state.db, query.reader_id.as_deref()) {
        Ok(loans) => Json(ApiResponse::ok("查詢成功", loans)),
        Err(e) => Json(ApiResponse::err(format!("查詢在借清單失敗: {}", e))),
    }
}

async fn list_loan_history(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<Vec<LoanRecord>>> {
    let q = query.q.unwrap_or_default();
    let limit = query.limit.unwrap_or(200);
    match get_loan_history(&state.db, &q, limit) {
        Ok(loans) => Json(ApiResponse::ok("查詢成功", loans)),
        Err(e) => Json(ApiResponse::err(format!("查詢歷史借還失敗: {}", e))),
    }
}

// === 統計與匯入 ===

async fn get_stats_summary(State(state): State<AppState>) -> Json<ApiResponse<StatsSummary>> {
    match get_stats(&state.db) {
        Ok(stats) => Json(ApiResponse::ok("統計獲取成功", stats)),
        Err(e) => Json(ApiResponse::err(format!("統計獲取失敗: {}", e))),
    }
}

async fn trigger_legacy_import(State(state): State<AppState>) -> Json<ApiResponse<String>> {
    match import_legacy_data(&state.db, &state.data_dir) {
        Ok(msg) => Json(ApiResponse::ok(format!("舊系統資料匯入成功：{}", msg), msg)),
        Err(e) => Json(ApiResponse::err(format!("匯入失敗: {}", e))),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPingResponse {
    pub status: &'static str,
    pub app: &'static str,
    pub version: &'static str,
}

pub async fn handle_system_ping() -> Json<SystemPingResponse> {
    Json(SystemPingResponse {
        status: "ok",
        app: "librust",
        version: "0.7",
    })
}

async fn handle_system_shutdown(State(state): State<AppState>) -> Json<ApiResponse<()>> {
    let port_file = state.data_dir.parent().map(|p| p.join("library.port"));
    tokio::spawn(async move {
        if let Some(pf) = port_file {
            let _ = std::fs::remove_file(pf);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        std::process::exit(0);
    });
    Json(ApiResponse::ok("系統即將安全關閉", ()))
}

async fn serve_favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        include_bytes!("../app.ico").as_slice(),
    )
}

async fn serve_app_png() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png")],
        include_bytes!("../app.png").as_slice(),
    )
}

// === Excel / CSV 匯入 ===

async fn handle_import_books(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Json<ApiResponse<usize>> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = match field.bytes().await {
                Ok(b) => b,
                Err(e) => return Json(ApiResponse::err(format!("讀取上傳資料失敗: {}", e))),
            };

            let temp_path = std::env::temp_dir().join(format!("book_import_{}", filename));
            if let Err(e) = std::fs::write(&temp_path, &data) {
                return Json(ApiResponse::err(format!("暫存上傳檔案失敗: {}", e)));
            }

            let books = match import_books_from_file(&temp_path) {
                Ok(b) => b,
                Err(e) => {
                    let _ = std::fs::remove_file(&temp_path);
                    return Json(ApiResponse::err(e));
                }
            };

            let _ = std::fs::remove_file(&temp_path);
            let count = books.len();
            if count == 0 {
                return Json(ApiResponse::err("未能從檔案中解析到任何有效書籍資料，請確認標題欄位格式！"));
            }

            match batch_save_books(&state.db, &books) {
                Ok(saved) => return Json(ApiResponse::ok(format!("成功匯入 {} 本書籍！", saved), saved)),
                Err(e) => return Json(ApiResponse::err(format!("寫入資料庫失敗: {}", e))),
            }
        }
    }
    Json(ApiResponse::err("未收到上傳檔案！"))
}

async fn handle_import_readers(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Json<ApiResponse<usize>> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("readers.xlsx").to_string();
            let data = match field.bytes().await {
                Ok(b) => b,
                Err(e) => return Json(ApiResponse::err(format!("讀取上傳資料失敗: {}", e))),
            };

            let temp_path = std::env::temp_dir().join(format!("reader_import_{}", filename));
            if let Err(e) = std::fs::write(&temp_path, &data) {
                return Json(ApiResponse::err(format!("暫存檔案失敗: {}", e)));
            }

            let readers = match import_readers_from_file(&temp_path) {
                Ok(r) => r,
                Err(e) => {
                    let _ = std::fs::remove_file(&temp_path);
                    return Json(ApiResponse::err(e));
                }
            };

            let _ = std::fs::remove_file(&temp_path);
            let count = readers.len();
            if count == 0 {
                return Json(ApiResponse::err("未能從檔案中解析到任何有效讀者資料，請確認標題欄位格式！"));
            }

            match batch_save_readers(&state.db, &readers) {
                Ok(saved) => return Json(ApiResponse::ok(format!("成功匯入 {} 位讀者！", saved), saved)),
                Err(e) => return Json(ApiResponse::err(format!("寫入資料庫失敗: {}", e))),
            }
        }
    }
    Json(ApiResponse::err("未收到上傳檔案！"))
}

async fn download_books_template() -> impl IntoResponse {
    let csv_content = "\u{FEFF}條碼,書名,作者,出版社,出版年,ISBN,分類,存放位置,定價,備註\nB001,哈利波特：神秘的魔法石,J.K.羅琳,皇冠文化,2000,9789573317241,小說,A01,350,範例資料\nB002,原子習慣,詹姆斯·克利爾,方智,2019,9789861755267,心理勵志,B02,330,範例資料\n";
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"books_template.csv\""),
        ],
        csv_content,
    )
}

async fn download_readers_template() -> impl IntoResponse {
    let csv_content = "\u{FEFF}證號,姓名,身分別,班級,年級,電話,Email,備註\n001,陳大明,01,三年忠班,3,0912345678,user1@example.com,一般讀者範例\n002,王小美,02,教務處,,0922334455,teacher@example.com,教職員範例\n";
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"readers_template.csv\""),
        ],
        csv_content,
    )
}

// === 認證與工作階段控制 ===

fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(val) = headers.get("x-auth-token").and_then(|v| v.to_str().ok()) {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }
    if let Some(val) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(tok) = val.strip_prefix("Bearer ") {
            if !tok.trim().is_empty() {
                return Some(tok.trim().to_string());
            }
        }
    }
    if let Some(cookie_str) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        for part in cookie_str.split(';') {
            let part = part.trim();
            if let Some(tok) = part.strip_prefix("auth_token=") {
                return Some(tok.trim().to_string());
            }
        }
    }
    None
}

async fn handle_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match verify_user_login(&state.db, &payload.username, &payload.password) {
        Ok(true) => {
            let token = format!("{:x}{:x}", fastrand::u128(..), fastrand::u128(..));
            let _ = create_session(&state.db, &token, &payload.username);
            let cookie_val = format!("auth_token={}; Path=/; HttpOnly; SameSite=Lax", token);
            (
                StatusCode::OK,
                [(header::SET_COOKIE, cookie_val)],
                Json(ApiResponse::ok("登入成功！", Some(AuthResponse {
                    username: payload.username,
                    token,
                }))),
            ).into_response()
        }
        _ => {
            (
                StatusCode::OK,
                [(header::SET_COOKIE, "".to_string())],
                Json(ApiResponse::<Option<AuthResponse>>::err("帳號或密碼錯誤！")),
            ).into_response()
        }
    }
}

async fn handle_logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    if let Some(token) = extract_token(&headers) {
        let _ = delete_session(&state.db, &token);
    }
    let cookie_val = "auth_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax".to_string();
    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_val)],
        Json(ApiResponse::ok("已登出！", ())),
    )
}

async fn handle_auth_status(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Json<ApiResponse<AuthStatus>> {
    if let Some(token) = extract_token(&headers) {
        if let Some(username) = validate_session(&state.db, &token) {
            return Json(ApiResponse::ok("已登入", AuthStatus {
                logged_in: true,
                username: Some(username),
            }));
        }
    }
    Json(ApiResponse::ok("未登入", AuthStatus {
        logged_in: false,
        username: None,
    }))
}

async fn handle_change_password(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Json<ApiResponse<()>> {
    let username = extract_token(&headers)
        .and_then(|tok| validate_session(&state.db, &tok))
        .unwrap_or_else(|| "admin".to_string());

    match update_user_password(&state.db, &username, &payload.current_password, &payload.new_password) {
        Ok(_) => Json(ApiResponse::ok("密碼更新成功！已安全加密儲存於資料庫。", ())),
        Err(e) => Json(ApiResponse::err(e)),
    }
}

// === 系統設定 ===

async fn get_system_settings(State(state): State<AppState>) -> Json<ApiResponse<SystemSettings>> {
    let library_name = get_setting(&state.db, "library_name", "圖書管理中心");
    let library_phone = get_setting(&state.db, "library_phone", "");
    let library_address = get_setting(&state.db, "library_address", "");
    let borrow_rules = get_setting(&state.db, "borrow_rules", "1. 請憑本證借書。\n2. 借出圖書請妥善保管，如有損毀遺失需照價賠償。\n3. 請於規定期限前歸還。");
    let barcode_width_mm: f64 = get_setting(&state.db, "barcode_width_mm", "48").parse().unwrap_or(48.0);
    let barcode_height_mm: f64 = get_setting(&state.db, "barcode_height_mm", "40").parse().unwrap_or(40.0);
    let show_spine_tag: bool = get_setting(&state.db, "show_spine_tag", "1") == "1";
    let card_margin_mm: f64 = get_setting(&state.db, "card_margin_mm", "5").parse().unwrap_or(5.0);
    let requests_per_minute: u32 = get_setting(&state.db, "requests_per_minute", "20").parse().unwrap_or(20);
    let jitter_ms: u64 = get_setting(&state.db, "jitter_ms", "1000").parse().unwrap_or(1000);

    Json(ApiResponse::ok("讀取設定成功", SystemSettings {
        library_name,
        library_phone,
        library_address,
        borrow_rules,
        barcode_width_mm,
        barcode_height_mm,
        show_spine_tag,
        card_margin_mm,
        requests_per_minute,
        jitter_ms,
    }))
}

async fn save_system_settings(
    State(state): State<AppState>,
    Json(settings): Json<SystemSettings>,
) -> Json<ApiResponse<String>> {
    let _ = save_setting(&state.db, "library_name", &settings.library_name);
    let _ = save_setting(&state.db, "library_phone", &settings.library_phone);
    let _ = save_setting(&state.db, "library_address", &settings.library_address);
    let _ = save_setting(&state.db, "borrow_rules", &settings.borrow_rules);
    let _ = save_setting(&state.db, "barcode_width_mm", &settings.barcode_width_mm.to_string());
    let _ = save_setting(&state.db, "barcode_height_mm", &settings.barcode_height_mm.to_string());
    let _ = save_setting(&state.db, "show_spine_tag", if settings.show_spine_tag { "1" } else { "0" });
    let _ = save_setting(&state.db, "card_margin_mm", &settings.card_margin_mm.to_string());
    let _ = save_setting(&state.db, "requests_per_minute", &settings.requests_per_minute.to_string());
    let _ = save_setting(&state.db, "jitter_ms", &settings.jitter_ms.to_string());

    Json(ApiResponse::ok("設定儲存成功！", "成功".to_string()))
}

// === 封面靜態讀取 ===

async fn serve_cover_image(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let clean_name = filename.replace("..", "").replace('/', "").replace('\\', "");
    let file_path = state.covers_dir.join(&clean_name);

    if file_path.exists() {
        if let Ok(bytes) = std::fs::read(&file_path) {
            let mime = if clean_name.ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                bytes,
            ).into_response();
        }
    }

    // 若圖片不存在，回傳 404
    (StatusCode::NOT_FOUND, "Cover not found").into_response()
}
