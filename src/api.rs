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
use chrono::Local;
use crate::db::{*, get_next_reader_id, get_next_book_id};
use crate::models::*;
use crate::importer::*;
use crate::enricher::EnricherManager;

pub const APP_VERSION: &str = "0.10";
pub const GITHUB_RELEASES_URL: &str = "https://github.com/prolin99/librust/releases";
const GITHUB_LATEST_RELEASE_API: &str = "https://api.github.com/repos/prolin99/librust/releases/latest";
const GITHUB_RELEASES_LIST_API: &str = "https://api.github.com/repos/prolin99/librust/releases?per_page=1";

static UPDATE_CACHE: std::sync::OnceLock<tokio::sync::RwLock<Option<(std::time::Instant, UpdateCheckResult)>>> = std::sync::OnceLock::new();

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
        .route("/api/stats/range", get(get_stats_range))
        .route("/api/settings", get(get_system_settings).post(save_system_settings))
        .route("/api/system/import-legacy", post(trigger_legacy_import))
        .route("/api/system/shutdown", post(handle_system_shutdown))
        .route("/api/system/ping", get(handle_system_ping))
        .route("/api/system/check-update", get(handle_check_update))
        .route("/api/system/apply-update", post(handle_apply_update))
        // 圖示與網站 Favicon
        .route("/favicon.ico", get(serve_favicon))
        .route("/app.png", get(serve_app_png))
        // 匯入與範本
        .route("/api/import/books", post(handle_import_books))
        .route("/api/import/readers", post(handle_import_readers))
        .route("/api/templates/books.csv", get(download_books_template))
        .route("/api/templates/readers.csv", get(download_readers_template))
        // 資料匯出 (Excel / CSV)
        .route("/api/export/readers.csv", get(export_readers_csv))
        .route("/api/export/readers", get(export_readers_csv))
        .route("/api/export/books.csv", get(export_books_csv))
        .route("/api/export/books", get(export_books_csv))
        .route("/api/export/active-loans.csv", get(export_active_loans_csv))
        .route("/api/export/active-loans", get(export_active_loans_csv))
        .route("/api/export/stats/monthly.csv", get(export_stats_monthly_csv))
        .route("/api/export/stats/readers.csv", get(export_stats_readers_csv))
        .route("/api/export/stats/books.csv", get(export_stats_books_csv))
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

#[derive(Debug, Deserialize)]
pub struct RangeStatsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
}

async fn get_stats_summary(State(state): State<AppState>) -> Json<ApiResponse<StatsSummary>> {
    match get_stats(&state.db) {
        Ok(stats) => Json(ApiResponse::ok("統計獲取成功", stats)),
        Err(e) => Json(ApiResponse::err(format!("統計獲取失敗: {}", e))),
    }
}

async fn get_stats_range(
    State(state): State<AppState>,
    Query(query): Query<RangeStatsQuery>,
) -> Json<ApiResponse<RangeStatsReport>> {
    let limit = query.limit.unwrap_or(20);
    match get_range_stats_report(
        &state.db,
        query.start_date.as_deref(),
        query.end_date.as_deref(),
        limit,
    ) {
        Ok(report) => Json(ApiResponse::ok("區間統計獲取成功", report)),
        Err(e) => Json(ApiResponse::err(format!("區間統計獲取失敗: {}", e))),
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct LegacyImportRequest {
    #[serde(default)]
    pub clear_existing: bool,
}

async fn trigger_legacy_import(
    State(state): State<AppState>,
    payload: Option<Json<LegacyImportRequest>>,
) -> Json<ApiResponse<String>> {
    let clear_existing = payload.map(|Json(p)| p.clear_existing).unwrap_or(false);
    let db = state.db.clone();
    let data_dir = state.data_dir.clone();
    let res = tokio::task::spawn_blocking(move || import_legacy_data(&db, &data_dir, clear_existing)).await;
    match res {
        Ok(Ok(msg)) => Json(ApiResponse::ok(format!("舊系統資料匯入成功：{}", msg), msg)),
        Ok(Err(e)) => Json(ApiResponse::err(format!("匯入失敗: {}", e))),
        Err(e) => Json(ApiResponse::err(format!("匯入工作執行異常: {}", e))),
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
        version: APP_VERSION,
    })
}

/// 將版本字串解析為數字陣列，例如 "v0.9.1" -> [0, 9, 1]、"0.10" -> [0, 10]
pub fn parse_version_numbers(v: &str) -> Vec<u32> {
    let trimmed = v.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let main_part = trimmed.split(|c| c == '-' || c == '+').next().unwrap_or(trimmed);
    main_part
        .split('.')
        .filter_map(|part| {
            let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digits.is_empty() { None } else { digits.parse::<u32>().ok() }
        })
        .collect()
}

/// 判斷 latest 版本是否比 current 新
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let cur = parse_version_numbers(current);
    let lat = parse_version_numbers(latest);
    if cur.is_empty() || lat.is_empty() {
        return false;
    }
    let max_len = cur.len().max(lat.len());
    for i in 0..max_len {
        let c = *cur.get(i).unwrap_or(&0);
        let l = *lat.get(i).unwrap_or(&0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

#[derive(Debug, Clone, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseItem {
    tag_name: String,
    html_url: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<GithubReleaseAsset>,
}

/// 從 GitHub Release 附件中挑選 Windows 執行檔（優先選擇檔名含 librust 的 .exe，其次取第一個 .exe）
fn select_exe_asset(assets: &[GithubReleaseAsset]) -> Option<GithubReleaseAsset> {
    let exe_assets: Vec<&GithubReleaseAsset> = assets
        .iter()
        .filter(|a| {
            let lower = a.name.to_ascii_lowercase();
            lower.ends_with(".exe") && !a.browser_download_url.trim().is_empty()
        })
        .collect();

    if let Some(preferred) = exe_assets
        .iter()
        .find(|a| a.name.to_ascii_lowercase().contains("librust"))
    {
        return Some((*preferred).clone());
    }
    exe_assets.first().map(|a| (*a).clone())
}

async fn fetch_latest_github_release() -> Option<(String, String, Option<GithubReleaseAsset>)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent(format!("LibRust/{}", APP_VERSION))
        .build()
        .ok()?;

    // 1. 先查詢 /releases/latest（正式穩定版）
    if let Ok(resp) = client
        .get(GITHUB_LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(item) = resp.json::<GithubReleaseItem>().await {
                if !item.tag_name.trim().is_empty() {
                    let url = item.html_url.unwrap_or_else(|| GITHUB_RELEASES_URL.to_string());
                    let asset = select_exe_asset(&item.assets);
                    return Some((item.tag_name, url, asset));
                }
            }
        }
    }

    // 2. 若尚無 latest 正式版（例如僅有 pre-release），退而檢查最新一筆非草稿 release
    if let Ok(resp) = client
        .get(GITHUB_RELEASES_LIST_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(items) = resp.json::<Vec<GithubReleaseItem>>().await {
                if let Some(item) = items.into_iter().find(|r| !r.draft && !r.tag_name.trim().is_empty()) {
                    let url = item.html_url.unwrap_or_else(|| GITHUB_RELEASES_URL.to_string());
                    let asset = select_exe_asset(&item.assets);
                    return Some((item.tag_name, url, asset));
                }
            }
        }
    }

    None
}

#[derive(Debug, Deserialize, Default)]
pub struct CheckUpdateQuery {
    #[serde(default)]
    pub force: Option<String>,
}

async fn handle_check_update(
    Query(query): Query<CheckUpdateQuery>,
) -> Json<ApiResponse<UpdateCheckResult>> {
    let force_refresh = matches!(query.force.as_deref(), Some("1") | Some("true"));
    let cache = UPDATE_CACHE.get_or_init(|| tokio::sync::RwLock::new(None));
    if !force_refresh {
        let guard = cache.read().await;
        if let Some((checked_at, ref cached)) = *guard {
            if checked_at.elapsed() < std::time::Duration::from_secs(1800) {
                return Json(ApiResponse::ok("版本檢查完成", cached.clone()));
            }
        }
    }

    let result = match fetch_latest_github_release().await {
        Some((tag, url, asset)) => {
            let has_update = is_newer_version(APP_VERSION, &tag);
            let res = UpdateCheckResult {
                current_version: APP_VERSION.to_string(),
                latest_version: Some(tag),
                has_update,
                release_url: url,
                download_url: asset.as_ref().map(|a| a.browser_download_url.clone()),
                asset_name: asset.as_ref().map(|a| a.name.clone()),
                asset_size: asset.as_ref().map(|a| a.size),
            };
            let mut guard = cache.write().await;
            *guard = Some((std::time::Instant::now(), res.clone()));
            res
        }
        None => UpdateCheckResult {
            current_version: APP_VERSION.to_string(),
            latest_version: None,
            has_update: false,
            release_url: GITHUB_RELEASES_URL.to_string(),
            download_url: None,
            asset_name: None,
            asset_size: None,
        },
    };

    Json(ApiResponse::ok("版本檢查完成", result))
}

async fn handle_apply_update(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Json<ApiResponse<String>> {
    // 驗證管理員登入狀態
    let is_authed = extract_token(&headers)
        .and_then(|tok| validate_session(&state.db, &tok))
        .is_some();
    if !is_authed {
        return Json(ApiResponse::err("請先登入管理員帳號後再執行系統更新！"));
    }

    // 1. 即時查詢 GitHub 最新 Release
    let (tag, _release_url, asset_opt) = match fetch_latest_github_release().await {
        Some(v) => v,
        None => return Json(ApiResponse::err("無法連線至 GitHub Releases 取得最新版本資訊，請確認網路連線。")),
    };

    if !is_newer_version(APP_VERSION, &tag) {
        return Json(ApiResponse::err(format!(
            "目前已是最新版本 (v{})，無須更新。",
            APP_VERSION
        )));
    }

    let asset = match asset_opt {
        Some(a) => a,
        None => {
            return Json(ApiResponse::err(format!(
                "最新版本 {} 尚未附上可自動更新的 Windows 執行檔（.exe），請前往 GitHub Releases 手動下載。",
                tag
            )));
        }
    };

    // 安全檢查：僅允許從 GitHub 官方網域下載
    if !asset.browser_download_url.starts_with("https://github.com/")
        && !asset.browser_download_url.starts_with("https://objects.githubusercontent.com/")
    {
        return Json(ApiResponse::err("不支援的下載來源網址，已中止更新。"));
    }

    // 2. 下載新版執行檔（最長等待 180 秒）
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .user_agent(format!("LibRust/{}", APP_VERSION))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse::err(format!("建立下載連線失敗: {}", e))),
    };

    let resp = match client.get(&asset.browser_download_url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => return Json(ApiResponse::err(format!("下載新版執行檔失敗 (HTTP {})", r.status()))),
        Err(e) => return Json(ApiResponse::err(format!("下載新版執行檔失敗: {}", e))),
    };

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => return Json(ApiResponse::err(format!("讀取下載檔案內容失敗: {}", e))),
    };

    // 3. 驗證完整性與 Windows PE 標頭 ("MZ")
    if bytes.len() < 100 * 1024 || &bytes[0..2] != b"MZ" {
        return Json(ApiResponse::err("下載的檔案不是有效的 Windows 執行檔，已取消更新。"));
    }
    if asset.size > 0 && (bytes.len() as u64) != asset.size {
        return Json(ApiResponse::err(format!(
            "下載檔案大小不符（預期 {} bytes，實際 {} bytes），請稍後重試。",
            asset.size,
            bytes.len()
        )));
    }

    // 4. 透過 current_exe() 取得本機實際執行檔路徑（保留中文檔名如 圖書管理系統.exe）
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return Json(ApiResponse::err(format!("無法取得目前執行檔路徑: {}", e))),
    };

    let mut new_path_os = current_exe.as_os_str().to_os_string();
    new_path_os.push(".new");
    let new_path = std::path::PathBuf::from(new_path_os);

    let mut old_path_os = current_exe.as_os_str().to_os_string();
    old_path_os.push(".old");
    let old_path = std::path::PathBuf::from(old_path_os);

    // 寫入 .new 暫存檔
    if let Err(e) = std::fs::write(&new_path, &bytes) {
        let _ = std::fs::remove_file(&new_path);
        return Json(ApiResponse::err(format!(
            "無法寫入更新暫存檔（請確認目錄具備寫入權限，勿放置於 Program Files）: {}",
            e
        )));
    }

    // 5. 安全替換：目前執行檔 -> .old，.new -> 目前執行檔
    let _ = std::fs::remove_file(&old_path);
    if let Err(e) = std::fs::rename(&current_exe, &old_path) {
        let _ = std::fs::remove_file(&new_path);
        return Json(ApiResponse::err(format!("無法備份舊版執行檔: {}", e)));
    }

    if let Err(e) = std::fs::rename(&new_path, &current_exe) {
        // 若替換失敗，立刻回復舊版執行檔
        let _ = std::fs::rename(&old_path, &current_exe);
        let _ = std::fs::remove_file(&new_path);
        return Json(ApiResponse::err(format!("無法套用新版執行檔（已自動回復舊版）: {}", e)));
    }

    // 6. 將 SQLite WAL 刷寫回主檔並啟動新版程式後安全退出
    {
        if let Ok(conn) = state.db.lock() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
    }

    let port_file = state.data_dir.parent().map(|p| p.join("library.port"));
    let exe_to_launch = current_exe.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        if let Some(pf) = port_file {
            let _ = std::fs::remove_file(pf);
        }
        let _ = std::process::Command::new(&exe_to_launch)
            .arg("--post-update")
            .spawn();
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
        std::process::exit(0);
    });

    Json(ApiResponse::ok(
        format!("已成功下載並更新至 {}（{}），系統正在重新啟動...", tag, asset.name),
        tag,
    ))
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

// === 資料匯出 (Excel 相容 CSV) ===

fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            encoded.push(b as char);
        } else {
            encoded.push_str(&format!("%{:02X}", b));
        }
    }
    encoded
}

async fn export_readers_csv(State(state): State<AppState>) -> impl IntoResponse {
    let readers = match get_readers(&state.db, "") {
        Ok(r) => r,
        Err(_) => vec![],
    };

    let mut buf = Vec::new();
    buf.extend_from_slice("\u{FEFF}".as_bytes()); // UTF-8 BOM，以利 Excel 自動識別編碼
    {
        let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
        let _ = wtr.write_record(&[
            "證號", "姓名", "身分別代碼", "身分別", "班級", "年級", "電話", "Email", "狀態", "建立日期", "目前在借冊數", "備註",
        ]);
        for r in readers {
            let status_str = if r.status == "active" { "正常" } else { "停用" };
            let loans_count = r.active_loans_count.unwrap_or(0).to_string();
            let _ = wtr.write_record(&[
                &r.id,
                &r.name,
                &r.role_id,
                r.role_name.as_deref().unwrap_or(""),
                &r.class_name,
                &r.grade,
                &r.phone,
                &r.email,
                status_str,
                &r.created_at,
                &loans_count,
                &r.note,
            ]);
        }
        let _ = wtr.flush();
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename_ascii = format!("readers_{}.csv", timestamp);
    let filename_utf8 = format!("讀者資料_{}.csv", timestamp);
    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        filename_ascii,
        url_encode(&filename_utf8)
    );

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/csv; charset=utf-8"));
    if let Ok(val) = header::HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, val);
    }
    (headers, buf)
}

async fn export_books_csv(State(state): State<AppState>) -> impl IntoResponse {
    let books = match get_books(&state.db, "", "") {
        Ok(b) => b,
        Err(_) => vec![],
    };

    let mut buf = Vec::new();
    buf.extend_from_slice("\u{FEFF}".as_bytes()); // UTF-8 BOM
    {
        let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
        let _ = wtr.write_record(&[
            "條碼", "書名", "作者", "出版社", "出版年", "ISBN", "分類", "存放位置", "定價", "館藏狀態", "借閱次數", "目前借閱人證號", "目前借閱人姓名", "借出日期", "應還日期", "建檔日期", "備註",
        ]);
        for b in books {
            let status_str = match b.status.as_str() {
                "available" => "可借閱",
                "borrowed" => "出借中",
                "lost" => "遺失",
                "maintenance" => "館內維護",
                s => s,
            };
            let (borrower_id, borrower_name, borrow_date, due_date) = if let Some(ref cb) = b.current_borrower {
                (cb.reader_id.as_str(), cb.reader_name.as_str(), cb.borrow_date.as_str(), cb.due_date.as_str())
            } else {
                ("", "", "", "")
            };
            let price_str = if b.price.fract() == 0.0 {
                format!("{:.0}", b.price)
            } else {
                format!("{:.2}", b.price)
            };
            let borrow_count_str = b.borrow_count.to_string();
            let _ = wtr.write_record(&[
                &b.id,
                &b.title,
                &b.author,
                &b.publisher,
                &b.pub_year,
                &b.isbn,
                &b.category,
                &b.location,
                &price_str,
                status_str,
                &borrow_count_str,
                borrower_id,
                borrower_name,
                borrow_date,
                due_date,
                &b.created_at,
                &b.note,
            ]);
        }
        let _ = wtr.flush();
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename_ascii = format!("books_{}.csv", timestamp);
    let filename_utf8 = format!("書籍資料_{}.csv", timestamp);
    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        filename_ascii,
        url_encode(&filename_utf8)
    );

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/csv; charset=utf-8"));
    if let Ok(val) = header::HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, val);
    }
    (headers, buf)
}

#[derive(Debug, Deserialize)]
pub struct ExportActiveLoansQuery {
    pub status: Option<String>,
    pub q: Option<String>,
}

async fn export_active_loans_csv(
    State(state): State<AppState>,
    Query(query): Query<ExportActiveLoansQuery>,
) -> impl IntoResponse {
    let loans = match get_active_loans(&state.db, None) {
        Ok(l) => l,
        Err(_) => vec![],
    };

    let status_filter = query.status.as_deref().unwrap_or("").trim();
    let q_filter = query.q.as_deref().unwrap_or("").trim().to_lowercase();

    let filtered_loans: Vec<LoanRecord> = loans.into_iter().filter(|item| {
        if status_filter == "overdue" && !item.is_overdue {
            return false;
        }
        if status_filter == "normal" && item.is_overdue {
            return false;
        }
        if !q_filter.is_empty() {
            let match_title = item.book_title.as_deref().unwrap_or("").to_lowercase().contains(&q_filter);
            let match_book_id = item.book_id.to_lowercase().contains(&q_filter);
            let match_reader_name = item.reader_name.as_deref().unwrap_or("").to_lowercase().contains(&q_filter);
            let match_reader_id = item.reader_id.to_lowercase().contains(&q_filter);
            if !match_title && !match_book_id && !match_reader_name && !match_reader_id {
                return false;
            }
        }
        true
    }).collect();

    let mut buf = Vec::new();
    buf.extend_from_slice("\u{FEFF}".as_bytes()); // UTF-8 BOM
    {
        let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
        let _ = wtr.write_record(&[
            "借閱編號", "書籍條碼", "書名", "讀者證號", "借閱人姓名", "身分別", "借閱日期", "應還日期", "借閱狀態", "剩餘或逾期天數", "續借次數", "備註",
        ]);
        for l in filtered_loans {
            let status_str = if l.is_overdue { "⚠️ 逾期催還" } else { "正常借閱中" };
            let days_str = if l.is_overdue {
                format!("逾期 {} 天", l.overdue_days.unwrap_or(0))
            } else {
                format!("剩餘 {} 天", l.days_remaining.unwrap_or(0))
            };
            let loan_id_str = l.id.to_string();
            let renew_count_str = l.renew_count.to_string();

            let _ = wtr.write_record(&[
                &loan_id_str,
                &l.book_id,
                l.book_title.as_deref().unwrap_or(""),
                &l.reader_id,
                l.reader_name.as_deref().unwrap_or(""),
                l.role_name.as_deref().unwrap_or(""),
                &l.borrow_date,
                &l.due_date,
                status_str,
                &days_str,
                &renew_count_str,
                &l.note,
            ]);
        }
        let _ = wtr.flush();
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let (filename_ascii, filename_utf8) = if status_filter == "overdue" {
        (format!("overdue_loans_{}.csv", timestamp), format!("催還名單_{}.csv", timestamp))
    } else {
        (format!("active_loans_{}.csv", timestamp), format!("在借圖書清單_{}.csv", timestamp))
    };

    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        filename_ascii,
        url_encode(&filename_utf8)
    );

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/csv; charset=utf-8"));
    if let Ok(val) = header::HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, val);
    }
    (headers, buf)
}

fn build_csv_response(buf: Vec<u8>, filename_ascii: &str, filename_utf8: &str) -> (header::HeaderMap, Vec<u8>) {
    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        filename_ascii,
        url_encode(filename_utf8)
    );
    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/csv; charset=utf-8"));
    if let Ok(val) = header::HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, val);
    }
    (headers, buf)
}

async fn export_stats_monthly_csv(
    State(state): State<AppState>,
    Query(query): Query<RangeStatsQuery>,
) -> impl IntoResponse {
    let report = match get_range_stats_report(
        &state.db,
        query.start_date.as_deref(),
        query.end_date.as_deref(),
        0,
    ) {
        Ok(r) => r,
        Err(_) => RangeStatsReport {
            start_date: "".to_string(),
            end_date: "".to_string(),
            limit: 0,
            total_visits: 0,
            total_books: 0,
            monthly_stats: vec![],
            reader_rankings: vec![],
            book_rankings: vec![],
        },
    };

    let mut buf = Vec::new();
    buf.extend_from_slice("\u{FEFF}".as_bytes());
    {
        let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
        let _ = wtr.write_record(&["月份", "借閱人次", "借閱冊數"]);
        for m in &report.monthly_stats {
            let _ = wtr.write_record(&[
                m.month.as_str(),
                &m.borrow_visits.to_string(),
                &m.borrow_books.to_string(),
            ]);
        }
        let _ = wtr.write_record(&[
            "合計",
            &report.total_visits.to_string(),
            &report.total_books.to_string(),
        ]);
        let _ = wtr.flush();
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    build_csv_response(
        buf,
        &format!("monthly_borrow_stats_{}.csv", timestamp),
        &format!("月份借閱統計_{}.csv", timestamp),
    )
}

async fn export_stats_readers_csv(
    State(state): State<AppState>,
    Query(query): Query<RangeStatsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    let report = match get_range_stats_report(
        &state.db,
        query.start_date.as_deref(),
        query.end_date.as_deref(),
        limit,
    ) {
        Ok(r) => r,
        Err(_) => RangeStatsReport {
            start_date: "".to_string(),
            end_date: "".to_string(),
            limit,
            total_visits: 0,
            total_books: 0,
            monthly_stats: vec![],
            reader_rankings: vec![],
            book_rankings: vec![],
        },
    };

    let range_label = format!(
        "{} ~ {}",
        if report.start_date.is_empty() { "不限" } else { &report.start_date },
        if report.end_date.is_empty() { "不限" } else { &report.end_date }
    );

    let mut buf = Vec::new();
    buf.extend_from_slice("\u{FEFF}".as_bytes());
    {
        let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
        let _ = wtr.write_record(&["排名", "讀者證號", "姓名", "身分別", "班級/部門", "借閱人次", "借閱冊數", "統計期間"]);
        for (idx, r) in report.reader_rankings.iter().enumerate() {
            let _ = wtr.write_record(&[
                &(idx + 1).to_string(),
                r.reader_id.as_str(),
                r.reader_name.as_str(),
                r.role_name.as_str(),
                r.class_name.as_str(),
                &r.borrow_visits.to_string(),
                &r.borrow_books.to_string(),
                &range_label,
            ]);
        }
        let _ = wtr.flush();
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    build_csv_response(
        buf,
        &format!("reader_borrow_ranking_{}.csv", timestamp),
        &format!("個人借閱冊數統計_{}.csv", timestamp),
    )
}

async fn export_stats_books_csv(
    State(state): State<AppState>,
    Query(query): Query<RangeStatsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    let report = match get_range_stats_report(
        &state.db,
        query.start_date.as_deref(),
        query.end_date.as_deref(),
        limit,
    ) {
        Ok(r) => r,
        Err(_) => RangeStatsReport {
            start_date: "".to_string(),
            end_date: "".to_string(),
            limit,
            total_visits: 0,
            total_books: 0,
            monthly_stats: vec![],
            reader_rankings: vec![],
            book_rankings: vec![],
        },
    };

    let range_label = format!(
        "{} ~ {}",
        if report.start_date.is_empty() { "不限" } else { &report.start_date },
        if report.end_date.is_empty() { "不限" } else { &report.end_date }
    );

    let mut buf = Vec::new();
    buf.extend_from_slice("\u{FEFF}".as_bytes());
    {
        let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
        let _ = wtr.write_record(&["排名", "書籍條碼", "書名", "作者", "分類", "被借次數", "統計期間"]);
        for (idx, b) in report.book_rankings.iter().enumerate() {
            let _ = wtr.write_record(&[
                &(idx + 1).to_string(),
                b.book_id.as_str(),
                b.title.as_str(),
                b.author.as_str(),
                b.category.as_str(),
                &b.borrow_count.to_string(),
                &range_label,
            ]);
        }
        let _ = wtr.flush();
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    build_csv_response(
        buf,
        &format!("book_borrow_ranking_{}.csv", timestamp),
        &format!("書籍被借次數統計_{}.csv", timestamp),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn test_ping_version_is_0_10() {
        let resp = handle_system_ping().await;
        assert_eq!(resp.0.version, "0.10");
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.9", "0.10"));
        assert!(is_newer_version("0.9", "v0.9.1"));
        assert!(is_newer_version("0.9", "1.0"));
        assert!(is_newer_version("0.9", "V1.0.0"));
        assert!(!is_newer_version("0.9", "0.9"));
        assert!(!is_newer_version("0.9", "v0.9.0"));
        assert!(!is_newer_version("0.9", "0.8"));
        assert!(!is_newer_version("0.9", "v0.8.9"));
        assert!(!is_newer_version("0.9", ""));
    }

    #[test]
    fn test_select_exe_asset() {
        let assets = vec![
            GithubReleaseAsset {
                name: "LibRust.zip".to_string(),
                browser_download_url: "https://github.com/prolin99/librust/releases/download/v0.11/LibRust.zip".to_string(),
                size: 1000,
            },
            GithubReleaseAsset {
                name: "helper.exe".to_string(),
                browser_download_url: "https://github.com/prolin99/librust/releases/download/v0.11/helper.exe".to_string(),
                size: 2000,
            },
            GithubReleaseAsset {
                name: "LibRust.exe".to_string(),
                browser_download_url: "https://github.com/prolin99/librust/releases/download/v0.11/LibRust.exe".to_string(),
                size: 9000000,
            },
        ];
        let picked = select_exe_asset(&assets).unwrap();
        assert_eq!(picked.name, "LibRust.exe");
        assert_eq!(picked.size, 9000000);
    }

    #[tokio::test]
    async fn test_range_stats_and_csv_exports() {
        let temp_dir = std::env::temp_dir().join(format!("test_range_stats_{}", fastrand::u64(..)));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");
        let covers_dir = temp_dir.join("covers");
        let data_dir = temp_dir.join("data");
        std::fs::create_dir_all(&covers_dir).unwrap();

        let db = init_db(&db_path).unwrap();

        // 建立測試讀者與書籍
        for (rid, rname) in [("R01", "王小明"), ("R02", "李小華")] {
            let _ = create_reader(&db, Reader {
                id: rid.to_string(),
                name: rname.to_string(),
                role_id: "01".to_string(),
                role_name: None,
                max_books: Some(10),
                borrow_days: Some(14),
                class_name: "三年忠班".to_string(),
                grade: "3".to_string(),
                phone: "".to_string(),
                email: "".to_string(),
                status: "active".to_string(),
                note: "".to_string(),
                created_at: "2026-01-01".to_string(),
                active_loans_count: None,
            });
        }
        for (bid, title) in [("B01", "Rust 程式設計"), ("B02", "演算法圖鑑")] {
            let _ = create_book(&db, Book {
                id: bid.to_string(),
                title: title.to_string(),
                author: "作者".to_string(),
                publisher: "出版社".to_string(),
                pub_year: "2026".to_string(),
                isbn: "".to_string(),
                category: "電腦".to_string(),
                location: "A01".to_string(),
                price: 300.0,
                description: "".to_string(),
                cover_url: "".to_string(),
                status: "available".to_string(),
                borrow_count: 0,
                enrichment_status: "none".to_string(),
                note: "".to_string(),
                created_at: "2026-01-01".to_string(),
                current_borrower: None,
            });
        }

        // 插入跨月份借閱紀錄：
        // 2026-01: R01 同日借 B01, B02 (1 人次, 2 冊)；R02 借 B01 (1 人次, 1 冊) -> 合計 2 人次, 3 冊
        // 2026-03: R01 借 B01 (1 人次, 1 冊) -> 2026-02 應補 0
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO loans (book_id, reader_id, borrow_date, due_date, return_date, status, renew_count, note) VALUES
                 ('B01', 'R01', '2026-01-10', '2026-01-24', '2026-01-20', 'returned', 0, ''),
                 ('B02', 'R01', '2026-01-10', '2026-01-24', '2026-01-20', 'returned', 0, ''),
                 ('B01', 'R02', '2026-01-15', '2026-01-29', '2026-01-25', 'returned', 0, ''),
                 ('B01', 'R01', '2026-03-05', '2026-03-19', '2026-03-15', 'returned', 0, '')",
                [],
            ).unwrap();
        }

        let report = get_range_stats_report(&db, Some("2026-01-01"), Some("2026-03-31"), 1).unwrap();
        assert_eq!(report.monthly_stats.len(), 3);
        assert_eq!(report.monthly_stats[0].month, "2026-01");
        assert_eq!(report.monthly_stats[0].borrow_visits, 2);
        assert_eq!(report.monthly_stats[0].borrow_books, 3);
        assert_eq!(report.monthly_stats[1].month, "2026-02");
        assert_eq!(report.monthly_stats[1].borrow_visits, 0);
        assert_eq!(report.monthly_stats[1].borrow_books, 0);
        assert_eq!(report.monthly_stats[2].month, "2026-03");
        assert_eq!(report.monthly_stats[2].borrow_visits, 1);
        assert_eq!(report.monthly_stats[2].borrow_books, 1);
        assert_eq!(report.total_visits, 3);
        assert_eq!(report.total_books, 4);

        // limit = 1 時，個人排行只回傳第一名 R01 (2 人次, 3 冊)
        assert_eq!(report.reader_rankings.len(), 1);
        assert_eq!(report.reader_rankings[0].reader_id, "R01");
        assert_eq!(report.reader_rankings[0].borrow_visits, 2);
        assert_eq!(report.reader_rankings[0].borrow_books, 3);

        // limit = 1 時，書籍排行只回傳第一名 B01 (3 次)
        assert_eq!(report.book_rankings.len(), 1);
        assert_eq!(report.book_rankings[0].book_id, "B01");
        assert_eq!(report.book_rankings[0].borrow_count, 3);

        // 測試 3 個 CSV 匯出端點
        let state = AppState {
            db: db.clone(),
            data_dir,
            covers_dir: covers_dir.clone(),
            enricher: Arc::new(EnricherManager::new(covers_dir)),
        };

        let q = RangeStatsQuery {
            start_date: Some("2026-01-01".to_string()),
            end_date: Some("2026-03-31".to_string()),
            limit: Some(10),
        };
        let r1 = export_stats_monthly_csv(State(state.clone()), Query(q)).await.into_response();
        let b1 = to_bytes(r1.into_body(), usize::MAX).await.unwrap();
        let s1 = String::from_utf8_lossy(&b1);
        assert!(s1.starts_with('\u{FEFF}'));
        assert!(s1.contains("月份,借閱人次,借閱冊數"));
        assert!(s1.contains("2026-01,2,3"));

        let q2 = RangeStatsQuery {
            start_date: Some("2026-01-01".to_string()),
            end_date: Some("2026-03-31".to_string()),
            limit: Some(10),
        };
        let r2 = export_stats_readers_csv(State(state.clone()), Query(q2)).await.into_response();
        let b2 = to_bytes(r2.into_body(), usize::MAX).await.unwrap();
        let s2 = String::from_utf8_lossy(&b2);
        assert!(s2.contains("讀者證號,姓名"));
        assert!(s2.contains("R01,王小明"));

        let q3 = RangeStatsQuery {
            start_date: Some("2026-01-01".to_string()),
            end_date: Some("2026-03-31".to_string()),
            limit: Some(10),
        };
        let r3 = export_stats_books_csv(State(state), Query(q3)).await.into_response();
        let b3 = to_bytes(r3.into_body(), usize::MAX).await.unwrap();
        let s3 = String::from_utf8_lossy(&b3);
        assert!(s3.contains("書籍條碼,書名"));
        assert!(s3.contains("B01,Rust 程式設計"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_export_endpoints() {
        let temp_dir = std::env::temp_dir().join(format!("test_export_{}", fastrand::u64(..)));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");
        let covers_dir = temp_dir.join("covers");
        let data_dir = temp_dir.join("data");
        std::fs::create_dir_all(&covers_dir).unwrap();

        let db = init_db(&db_path).unwrap();

        // 建立測試讀者
        let test_reader = Reader {
            id: "R888".to_string(),
            name: "測試讀者888".to_string(),
            role_id: "01".to_string(),
            role_name: None,
            max_books: Some(5),
            borrow_days: Some(14),
            class_name: "三年一班".to_string(),
            grade: "3".to_string(),
            phone: "0911222333".to_string(),
            email: "r888@example.com".to_string(),
            status: "active".to_string(),
            note: "測試用備註".to_string(),
            created_at: "2026-11-22".to_string(),
            active_loans_count: Some(0),
        };
        let _ = create_reader(&db, test_reader);

        // 建立測試書籍
        let test_book = Book {
            id: "B888".to_string(),
            title: "測試書籍888".to_string(),
            author: "測試作者".to_string(),
            publisher: "測試出版社".to_string(),
            pub_year: "2026".to_string(),
            isbn: "9789570000000".to_string(),
            category: "電腦科學".to_string(),
            location: "A01".to_string(),
            price: 500.0,
            description: "".to_string(),
            cover_url: "".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            enrichment_status: "none".to_string(),
            note: "測試書籍備註".to_string(),
            created_at: "2026-11-22".to_string(),
            current_borrower: None,
        };
        let _ = create_book(&db, test_book);

        // 執行借閱以產生在借紀錄
        let _ = borrow_book(&db, "R888", "B888");

        let enricher = Arc::new(EnricherManager::new(covers_dir.clone()));

        let state = AppState {
            db: db.clone(),
            data_dir: data_dir.clone(),
            covers_dir: covers_dir.clone(),
            enricher,
        };

        // 1. 測試讀者匯出
        let resp = export_readers_csv(State(state.clone())).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.starts_with('\u{FEFF}'));
        assert!(content.contains("證號,姓名,身分別代碼"));
        assert!(content.contains("R888,測試讀者888"));

        // 2. 測試書籍匯出
        let resp = export_books_csv(State(state.clone())).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.starts_with('\u{FEFF}'));
        assert!(content.contains("條碼,書名,作者"));
        assert!(content.contains("B888,測試書籍888"));

        // 3. 測試在借與催還匯出 (全部)
        let resp = export_active_loans_csv(
            State(state.clone()),
            Query(ExportActiveLoansQuery { status: None, q: None }),
        ).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.starts_with('\u{FEFF}'));
        assert!(content.contains("借閱編號,書籍條碼,書名"));
        assert!(content.contains("B888,測試書籍888,R888,測試讀者888"));

        // 4. 測試在借與催還匯出 (篩選條件 - 無符合關鍵字)
        let resp_empty = export_active_loans_csv(
            State(state.clone()),
            Query(ExportActiveLoansQuery { status: None, q: Some("不存在的關鍵字".to_string()) }),
        ).await.into_response();
        let bytes_empty = to_bytes(resp_empty.into_body(), usize::MAX).await.unwrap();
        let content_empty = String::from_utf8_lossy(&bytes_empty);
        assert!(content_empty.starts_with('\u{FEFF}'));
        assert!(content_empty.contains("借閱編號,書籍條碼,書名"));
        assert!(!content_empty.contains("B888"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
