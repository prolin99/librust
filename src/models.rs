use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub max_books: i32,
    pub borrow_days: i32,
    pub renew_days: i32,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reader {
    pub id: String,
    pub name: String,
    pub role_id: String,
    pub role_name: Option<String>,
    pub max_books: Option<i32>,
    pub borrow_days: Option<i32>,
    pub class_name: String,
    pub grade: String,
    pub phone: String,
    pub email: String,
    pub status: String, // 'active', 'disabled'
    pub note: String,
    pub created_at: String,
    pub active_loans_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub pub_year: String,
    pub isbn: String,
    pub category: String,
    pub location: String,
    pub price: f64,
    pub description: String,
    pub cover_url: String,
    pub status: String, // 'available', 'borrowed', 'lost', 'maintenance'
    pub borrow_count: i32,
    pub enrichment_status: String, // 'none', 'completed', 'not_found', 'failed'
    pub note: String,
    pub created_at: String,
    pub current_borrower: Option<CurrentBorrowerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentBorrowerInfo {
    pub reader_id: String,
    pub reader_name: String,
    pub borrow_date: String,
    pub due_date: String,
    pub days_remaining: i64,
    pub is_overdue: bool,
    pub loan_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanRecord {
    pub id: i64,
    pub book_id: String,
    pub book_title: Option<String>,
    pub reader_id: String,
    pub reader_name: Option<String>,
    pub role_name: Option<String>,
    pub borrow_date: String,
    pub due_date: String,
    pub return_date: Option<String>,
    pub status: String, // 'active', 'returned', 'overdue'
    pub renew_count: i32,
    pub note: String,
    pub days_remaining: Option<i64>,
    pub is_overdue: bool,
    pub overdue_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverdueReturnRecord {
    pub loan_id: i64,
    pub reader_id: String,
    pub reader_name: String,
    pub book_id: String,
    pub book_title: String,
    pub borrow_date: String,
    pub due_date: String,
    pub return_date: String,
    pub overdue_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummary {
    pub total_books: i64,
    pub available_books: i64,
    pub borrowed_books: i64,
    pub total_readers: i64,
    pub active_loans: i64,
    pub overdue_loans: i64,
    pub total_history_loans: i64,
    pub category_distribution: Vec<CategoryStat>,
    pub top_borrowed_books: Vec<BookStat>,
    pub recent_overdue_returns: Vec<OverdueReturnRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookStat {
    pub id: String,
    pub title: String,
    pub borrow_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyBorrowStat {
    pub month: String,       // YYYY-MM
    pub borrow_visits: i64,  // 借閱人次 (同日同讀者計 1 人次)
    pub borrow_books: i64,   // 借閱冊數
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderBorrowRank {
    pub reader_id: String,
    pub reader_name: String,
    pub role_name: String,
    pub class_name: String,
    pub borrow_visits: i64,  // 借閱人次
    pub borrow_books: i64,   // 借閱冊數
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookBorrowRank {
    pub book_id: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub borrow_count: i64,   // 期間內被借次數
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeStatsReport {
    pub start_date: String,
    pub end_date: String,
    pub limit: i64,
    pub total_visits: i64,
    pub total_books: i64,
    pub monthly_stats: Vec<MonthlyBorrowStat>,
    pub reader_rankings: Vec<ReaderBorrowRank>,
    pub book_rankings: Vec<BookBorrowRank>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub release_url: String,
}

#[derive(Debug, Deserialize)]
pub struct BorrowRequest {
    pub reader_id: String,
    pub book_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ReturnRequest {
    pub book_id: Option<String>,
    pub loan_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RenewRequest {
    pub loan_id: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub max_requests_per_hour: u32,
    pub jitter_ms: u64,
    pub update_cover: bool,
    pub update_desc: bool,
    pub update_meta: bool,
    pub overwrite_existing: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 20,
            max_requests_per_hour: 500,
            jitter_ms: 1000,
            update_cover: true,
            update_desc: true,
            update_meta: true,
            overwrite_existing: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentProgress {
    pub is_running: bool,
    pub total: usize,
    pub processed: usize,
    pub success: usize,
    pub not_found: usize,
    pub failed: usize,
    pub current_book: Option<String>,
    pub last_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn ok(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
}
