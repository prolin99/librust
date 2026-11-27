use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use std::path::Path;
use chrono::{Local, NaiveDate};
use crate::models::*;
use crate::legacy::{read_dbf, format_dbf_date};

pub type DbPool = Arc<Mutex<Connection>>;

/// 目前資料庫結構版本號；未來每當資料表或欄位有異動時遞增，並於 `run_migrations` 加入對應升級步驟
pub const CURRENT_SCHEMA_VERSION: i32 = 4;

/// 檢查指定資料表是否已存在某欄位
fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&sql)?;
    let cols: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(cols.iter().any(|c| c.eq_ignore_ascii_case(column)))
}

/// 若指定資料表缺少某欄位，自動執行 ALTER TABLE 補齊（回傳 true 表示有新增）
fn ensure_column(conn: &Connection, table: &str, column: &str, col_def: &str) -> Result<bool> {
    if !column_exists(conn, table, column)? {
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_def);
        conn.execute(&sql, [])?;
        println!("[↑] 資料庫自動修正：資料表 {} 已補齊缺少欄位 {}", table, column);
        return Ok(true);
    }
    Ok(false)
}

/// 防禦性欄位自檢：確保所有核心資料表皆具備最新版所需欄位，避免舊版資料庫缺少欄位導致查詢失敗
fn ensure_all_expected_columns(conn: &Connection) -> Result<usize> {
    let expected_columns: &[(&str, &str, &str)] = &[
        // roles
        ("roles", "max_books", "INTEGER NOT NULL DEFAULT 5"),
        ("roles", "borrow_days", "INTEGER NOT NULL DEFAULT 14"),
        ("roles", "renew_days", "INTEGER NOT NULL DEFAULT 7"),
        ("roles", "note", "TEXT NOT NULL DEFAULT ''"),
        // readers
        ("readers", "role_id", "TEXT NOT NULL DEFAULT '01'"),
        ("readers", "class_name", "TEXT NOT NULL DEFAULT ''"),
        ("readers", "grade", "TEXT NOT NULL DEFAULT ''"),
        ("readers", "phone", "TEXT NOT NULL DEFAULT ''"),
        ("readers", "email", "TEXT NOT NULL DEFAULT ''"),
        ("readers", "status", "TEXT NOT NULL DEFAULT 'active'"),
        ("readers", "note", "TEXT NOT NULL DEFAULT ''"),
        ("readers", "created_at", "TEXT NOT NULL DEFAULT ''"),
        // books
        ("books", "author", "TEXT NOT NULL DEFAULT ''"),
        ("books", "publisher", "TEXT NOT NULL DEFAULT ''"),
        ("books", "pub_year", "TEXT NOT NULL DEFAULT ''"),
        ("books", "isbn", "TEXT NOT NULL DEFAULT ''"),
        ("books", "category", "TEXT NOT NULL DEFAULT ''"),
        ("books", "location", "TEXT NOT NULL DEFAULT ''"),
        ("books", "price", "REAL NOT NULL DEFAULT 0.0"),
        ("books", "description", "TEXT NOT NULL DEFAULT ''"),
        ("books", "cover_url", "TEXT NOT NULL DEFAULT ''"),
        ("books", "status", "TEXT NOT NULL DEFAULT 'available'"),
        ("books", "borrow_count", "INTEGER NOT NULL DEFAULT 0"),
        ("books", "enrichment_status", "TEXT NOT NULL DEFAULT 'none'"),
        ("books", "note", "TEXT NOT NULL DEFAULT ''"),
        ("books", "created_at", "TEXT NOT NULL DEFAULT ''"),
        // loans
        ("loans", "return_date", "TEXT"),
        ("loans", "status", "TEXT NOT NULL DEFAULT 'active'"),
        ("loans", "renew_count", "INTEGER NOT NULL DEFAULT 0"),
        ("loans", "note", "TEXT NOT NULL DEFAULT ''"),
        // users & sessions
        ("users", "created_at", "TEXT NOT NULL DEFAULT ''"),
        ("sessions", "created_at", "TEXT NOT NULL DEFAULT ''"),
        // categories
        ("categories", "sort_order", "INTEGER NOT NULL DEFAULT 0"),
        ("categories", "note", "TEXT NOT NULL DEFAULT ''"),
    ];

    let mut added = 0usize;
    for &(table, col, def) in expected_columns {
        if ensure_column(conn, table, col, def)? {
            added += 1;
        }
    }
    Ok(added)
}

/// 在進行舊資料庫結構升級前，自動建立一份完整備份檔，確保使用者資料安全無虞
fn backup_database_before_migration(path: &Path, conn: &Connection, from_ver: i32, to_ver: i32) -> Option<std::path::PathBuf> {
    if !path.exists() {
        return None;
    }
    // 先將 WAL 資料寫回主資料庫檔案，確保複製出來的備份檔完整一致
    let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_name = path.file_name()?.to_string_lossy();
    let backup_name = format!("{}.bak_v{}_to_v{}_{}", file_name, from_ver, to_ver, timestamp);
    let backup_path = path.with_file_name(backup_name);
    if std::fs::copy(path, &backup_path).is_ok() {
        Some(backup_path)
    } else {
        None
    }
}

/// 執行版本化資料庫結構自動遷移 (Schema Migrations)
fn run_migrations(conn: &mut Connection, path: &Path, existed_with_data: bool) -> Result<()> {
    let mut current_ver: i32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);

    // 建立遷移紀錄表（紀錄歷次自動升級歷史）
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );"
    )?;

    // 若為既有舊資料庫且版本低於目前版本，升級前先自動備份
    if existed_with_data && current_ver < CURRENT_SCHEMA_VERSION {
        if let Some(bak) = backup_database_before_migration(path, conn, current_ver, CURRENT_SCHEMA_VERSION) {
            println!(
                "[i] 偵測到舊版資料庫 (schema v{} -> v{})，已自動備份至: {}",
                current_ver,
                CURRENT_SCHEMA_VERSION,
                bak.display()
            );
        }
    }

    // 1. 先執行防禦性欄位自檢（補齊任何舊版資料庫缺少的欄位）
    ensure_all_expected_columns(conn)?;

    // 2. 依序執行版本化遷移（每個版本包在交易中，確保原子性）
    if current_ver < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_loans_active ON loans(status, reader_id, book_id);
            CREATE INDEX IF NOT EXISTS idx_books_status ON books(status);
            CREATE INDEX IF NOT EXISTS idx_readers_role ON readers(role_id);
            "
        )?;
        let now = Local::now().to_rfc3339();
        tx.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, description, applied_at) VALUES (1, ?1, ?2)",
            params!["初始化基礎資料表、補齊舊版缺少欄位與核心索引", now],
        )?;
        tx.pragma_update(None, "user_version", 1i32)?;
        tx.commit()?;
        current_ver = 1;
    }

    if current_ver < 2 {
        let tx = conn.transaction()?;
        // v0.9 升級：加入借閱日期與統計索引，並將舊資料可能存在的斜線日期 (YYYY/MM/DD) 正規化為 YYYY-MM-DD
        tx.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_loans_borrow_date ON loans(borrow_date);
            CREATE INDEX IF NOT EXISTS idx_loans_reader_date ON loans(reader_id, borrow_date);
            UPDATE loans SET borrow_date = replace(borrow_date, '/', '-') WHERE borrow_date LIKE '%/%';
            UPDATE loans SET due_date = replace(due_date, '/', '-') WHERE due_date LIKE '%/%';
            UPDATE loans SET return_date = replace(return_date, '/', '-') WHERE return_date IS NOT NULL AND return_date LIKE '%/%';
            "
        )?;
        let now = Local::now().to_rfc3339();
        tx.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, description, applied_at) VALUES (2, ?1, ?2)",
            params!["v0.9 統計索引建立與歷史借閱日期格式正規化", now],
        )?;
        tx.pragma_update(None, "user_version", 2i32)?;
        tx.commit()?;
        current_ver = 2;
    }

    if current_ver < 3 {
        let tx = conn.transaction()?;
        // v0.10 升級：建立在借書籍索引，並自動從同層 data/ 舊版 DBF 補齊空白之讀者備註 (Dbf02.NOTE) 與書籍分類號 (Dbf03.ID2)
        tx.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_loans_active_book ON loans(status, book_id);
            "
        )?;

        if let Some(parent_dir) = path.parent() {
            let data_dir = parent_dir.join("data");

            let dbf02_path = data_dir.join("Dbf02.DBF");
            if dbf02_path.exists() {
                if let Ok(table) = read_dbf(&dbf02_path) {
                    let mut updated_notes = 0usize;
                    for rec in table.records {
                        let id = rec.get("ID").cloned().unwrap_or_default();
                        let note = rec.get("NOTE").cloned().unwrap_or_default();
                        if !id.is_empty() && !note.is_empty() {
                            if let Ok(n) = tx.execute(
                                "UPDATE readers SET note = ?2 WHERE id = ?1 AND (note IS NULL OR note = '')",
                                params![id, note],
                            ) {
                                updated_notes += n;
                            }
                        }
                    }
                    if updated_notes > 0 {
                        println!("[↑] 資料庫自動升級 (v0.10)：已從 Dbf02.DBF 補齊 {} 筆讀者備註", updated_notes);
                    }
                }
            }

            let dbf03_path = data_dir.join("Dbf03.DBF");
            if dbf03_path.exists() {
                if let Ok(table) = read_dbf(&dbf03_path) {
                    let mut updated_cats = 0usize;
                    for rec in table.records {
                        let id = rec.get("ID").cloned().unwrap_or_default();
                        let id2 = rec.get("ID2").cloned().unwrap_or_default();
                        if !id.is_empty() && !id2.is_empty() {
                            if let Ok(n) = tx.execute(
                                "UPDATE books SET category = ?2 WHERE id = ?1 AND (category IS NULL OR category = '')",
                                params![id, id2],
                            ) {
                                updated_cats += n;
                            }
                        }
                    }
                    if updated_cats > 0 {
                        println!("[↑] 資料庫自動升級 (v0.10)：已從 Dbf03.DBF (ID2) 補齊 {} 筆書籍分類號", updated_cats);
                    }
                }
            }
        }

        let now = Local::now().to_rfc3339();
        tx.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, description, applied_at) VALUES (3, ?1, ?2)",
            params!["v0.10 在借書籍索引建立與舊版 DBF 讀者備註/書籍分類號(ID2)自動補齊", now],
        )?;
        tx.pragma_update(None, "user_version", 3i32)?;
        tx.commit()?;
        current_ver = 3;
    }

    if current_ver < 4 {
        let tx = conn.transaction()?;
        // v0.12 升級：建立書籍大分類資料表 categories 並預填中文圖書分類法 0~9 十大類
        tx.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS categories (
                code TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                note TEXT NOT NULL DEFAULT ''
            );
            "
        )?;
        let cat_count: i64 = tx.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0)).unwrap_or(0);
        if cat_count == 0 {
            let defaults: &[(&str, &str, i32)] = &[
                ("0", "總類", 0),
                ("1", "哲學", 1),
                ("2", "宗教", 2),
                ("3", "自然科學", 3),
                ("4", "應用科學", 4),
                ("5", "社會科學", 5),
                ("6", "中國史地", 6),
                ("7", "世界史地", 7),
                ("8", "語文類", 8),
                ("9", "藝術類", 9),
            ];
            for &(code, name, order) in defaults {
                tx.execute(
                    "INSERT OR IGNORE INTO categories (code, name, sort_order, note) VALUES (?1, ?2, ?3, '')",
                    params![code, name, order],
                )?;
            }
        }
        let now = Local::now().to_rfc3339();
        tx.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, description, applied_at) VALUES (4, ?1, ?2)",
            params!["v0.12 建立書籍大分類代碼表 (categories) 並預填 0~9 預設分類", now],
        )?;
        tx.pragma_update(None, "user_version", 4i32)?;
        tx.commit()?;
        current_ver = 4;
    }

    let _ = current_ver;
    Ok(())
}

fn init_core_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS roles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            max_books INTEGER NOT NULL DEFAULT 5,
            borrow_days INTEGER NOT NULL DEFAULT 14,
            renew_days INTEGER NOT NULL DEFAULT 7,
            note TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS readers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            role_id TEXT NOT NULL REFERENCES roles(id) ON UPDATE CASCADE,
            class_name TEXT NOT NULL DEFAULT '',
            grade TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'active',
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS books (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT NOT NULL DEFAULT '',
            publisher TEXT NOT NULL DEFAULT '',
            pub_year TEXT NOT NULL DEFAULT '',
            isbn TEXT NOT NULL DEFAULT '',
            category TEXT NOT NULL DEFAULT '',
            location TEXT NOT NULL DEFAULT '',
            price REAL NOT NULL DEFAULT 0.0,
            description TEXT NOT NULL DEFAULT '',
            cover_url TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'available',
            borrow_count INTEGER NOT NULL DEFAULT 0,
            enrichment_status TEXT NOT NULL DEFAULT 'none',
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS loans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id TEXT NOT NULL REFERENCES books(id) ON UPDATE CASCADE,
            reader_id TEXT NOT NULL REFERENCES readers(id) ON UPDATE CASCADE,
            borrow_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            return_date TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            renew_count INTEGER NOT NULL DEFAULT 0,
            note TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS categories (
            code TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            note TEXT NOT NULL DEFAULT ''
        );
        "
    )
}

pub fn init_db<P: AsRef<Path>>(path: P) -> Result<DbPool> {
    let path_ref = path.as_ref();
    let existed_with_data = path_ref.exists()
        && std::fs::metadata(path_ref).map(|m| m.len() > 0).unwrap_or(false);

    let mut conn = Connection::open(path_ref)?;
    init_core_tables(&conn)?;

    // 自動檢查並升級舊資料庫結構（新增缺少欄位、索引與版本化遷移）
    run_migrations(&mut conn, path_ref, existed_with_data)?;

    // 設定預設身分別 (若不存在)
    let role_count: i64 = conn.query_row("SELECT COUNT(*) FROM roles", [], |r| r.get(0)).unwrap_or(0);
    if role_count == 0 {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO roles (id, name, max_books, borrow_days, renew_days, note) VALUES 
             ('01', '一般讀者/服務對象', 2, 30, 14, '預設身分'),
             ('02', '教職員/工作人員', 15, 30, 14, '教師或人員')",
            [],
        );
    }

    // 設定預設機構資訊 (若不存在)
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0)).unwrap_or(0);
    if count == 0 {
        let _ = conn.execute("INSERT OR IGNORE INTO settings (key, value) VALUES ('library_name', '圖書管理中心')", []);
        let _ = conn.execute("INSERT OR IGNORE INTO settings (key, value) VALUES ('library_phone', '')", []);
        let _ = conn.execute("INSERT OR IGNORE INTO settings (key, value) VALUES ('library_address', '')", []);
        let _ = conn.execute("INSERT OR IGNORE INTO settings (key, value) VALUES ('borrow_rules', '1. 請憑本證借書。\n2. 借出圖書請妥善保管，如有損毀遺失需照價賠償。\n3. 請於規定期限前歸還，若逾期將依身分別規則暫停借書權限。')", []);
    }

    // 設定預設管理員帳號 (若不存在，預設帳號 admin，密碼 admin 以 bcrypt 加密)
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0)).unwrap_or(0);
    if user_count == 0 {
        let default_hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap_or_else(|_| {
            "$2b$12$e8x/kZ6qPzK3/p4fC9e9IeOqH0Y4Y9L9r7FjWqf8hV5x9k4yL".to_string()
        });
        let now = Local::now().to_rfc3339();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
            params!["admin", default_hash, now],
        );
    }

    Ok(Arc::new(Mutex::new(conn)))
}

pub fn import_legacy_data(
    db: &DbPool,
    data_dir: &Path,
    clear_existing: bool,
) -> std::result::Result<String, String> {
    if !data_dir.exists() {
        return Err(format!("找不到舊系統資料目錄：{}", data_dir.display()));
    }

    println!("[⏳] 開始讀取舊版 DBF 檔案並寫入資料庫，資料量較大時請耐心等待...");
    let mut conn = db.lock().unwrap();
    let tx = conn
        .transaction()
        .map_err(|e| format!("啟動移轉交易失敗: {}", e))?;
    let mut summary = Vec::new();

    if clear_existing {
        println!("[-] 正在清除現有借閱紀錄、書籍、讀者與身分別資料...");
        tx.execute("DELETE FROM loans", [])
            .map_err(|e| format!("清除借閱紀錄失敗: {}", e))?;
        tx.execute("DELETE FROM books", [])
            .map_err(|e| format!("清除書籍資料失敗: {}", e))?;
        tx.execute("DELETE FROM readers", [])
            .map_err(|e| format!("清除讀者資料失敗: {}", e))?;
        tx.execute("DELETE FROM roles", [])
            .map_err(|e| format!("清除身分別資料失敗: {}", e))?;
        let _ = tx.execute("DELETE FROM sqlite_sequence WHERE name = 'loans'", []);
        summary.push("已清除原有資料".to_string());
    }

    // 1. 身分別 DBF01.DBF
    let dbf01_path = data_dir.join("DBF01.DBF");
    if dbf01_path.exists() {
        println!("[-] 正在讀取 DBF01.DBF（身分別）...");
        if let Ok(table) = read_dbf(&dbf01_path) {
            let mut count = 0;
            for rec in table.records {
                let id = rec.get("ID").cloned().unwrap_or_default();
                let name = rec.get("NAME").cloned().unwrap_or_default();
                let lend_qty: i32 = rec.get("LEND_QTY").and_then(|v| v.parse().ok()).unwrap_or(5);
                let lend_day: i32 = rec.get("LEND_DAY").and_then(|v| v.parse().ok()).unwrap_or(14);
                let cont_day: i32 = rec.get("CONT_DAY").and_then(|v| v.parse().ok()).unwrap_or(7);
                let note = rec.get("NOTE").cloned().unwrap_or_default();

                if !id.is_empty() && !name.is_empty() {
                    let _ = tx.execute(
                        "INSERT OR REPLACE INTO roles (id, name, max_books, borrow_days, renew_days, note)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![id, name, lend_qty, lend_day, if cont_day > 0 { cont_day } else { 7 }, note],
                    );
                    count += 1;
                }
            }
            summary.push(format!("身分別匯入 {} 筆", count));
        }
    }

    let role_count: i64 = tx.query_row("SELECT COUNT(*) FROM roles", [], |r| r.get(0)).unwrap_or(0);
    if role_count == 0 {
        let _ = tx.execute(
            "INSERT INTO roles (id, name, max_books, borrow_days, renew_days, note) VALUES 
             ('01', '一般讀者/服務對象', 2, 30, 14, '預設身分'),
             ('02', '教職員/工作人員', 15, 30, 14, '教師或人員')",
            [],
        );
        summary.push("建立預設身分別 2 筆".to_string());
    }

    // 2. 讀者資料 Dbf02.DBF
    let dbf02_path = data_dir.join("Dbf02.DBF");
    if dbf02_path.exists() {
        println!("[-] 正在讀取 Dbf02.DBF（讀者資料）...");
        if let Ok(table) = read_dbf(&dbf02_path) {
            let mut count = 0;
            let today = Local::now().format("%Y-%m-%d").to_string();
            for rec in table.records {
                let id = rec.get("ID").cloned().unwrap_or_default();
                let name = rec.get("NAME").cloned().unwrap_or_default();
                let mut role_id = rec.get("WHAT").cloned().unwrap_or_else(|| "01".to_string());
                if role_id.is_empty() {
                    role_id = "01".to_string();
                }
                let class_name = rec.get("CLASS").cloned().unwrap_or_default();
                let grade = rec.get("YEAR").cloned().unwrap_or_default();
                let state = rec.get("STATE").cloned().unwrap_or_else(|| "Y".to_string());
                let note = rec.get("NOTE").cloned().unwrap_or_default();
                let status = if state.to_uppercase() == "Y" { "active" } else { "disabled" };

                if !id.is_empty() && !name.is_empty() {
                    let _ = tx.execute(
                        "INSERT OR REPLACE INTO readers (id, name, role_id, class_name, grade, phone, email, status, note, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, '', '', ?6, ?7, ?8)",
                        params![id, name, role_id, class_name, grade, status, note, today],
                    );
                    count += 1;
                }
            }
            summary.push(format!("讀者資料匯入 {} 筆", count));
        }
    }

    // 3. 書籍資料 Dbf03.DBF
    let dbf03_path = data_dir.join("Dbf03.DBF");
    if dbf03_path.exists() {
        println!("[-] 正在讀取 Dbf03.DBF（書籍資料，請耐心等待）...");
        if let Ok(table) = read_dbf(&dbf03_path) {
            let mut count = 0;
            let today = Local::now().format("%Y-%m-%d").to_string();
            for rec in table.records {
                let id = rec.get("ID").cloned().unwrap_or_default();
                let title = rec.get("NAME").cloned().unwrap_or_default();
                let author = rec.get("AUTHOR").cloned().unwrap_or_default();
                let publisher = rec.get("PUBLICER").cloned().unwrap_or_default();
                let pub_year = rec.get("PUB_YEAR").cloned().unwrap_or_default();
                let isbn = rec.get("ISBN").cloned().unwrap_or_default();
                let category = rec.get("ID2").cloned().unwrap_or_default();
                let location = rec.get("WHERE").cloned().unwrap_or_default();
                let price: f64 = rec.get("PRICE").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let lend_sum: i32 = rec.get("LEND_SUM").and_then(|v| v.parse().ok()).unwrap_or(0);
                let note = rec.get("PROPERTY").cloned().unwrap_or_default();

                if !id.is_empty() && !title.is_empty() {
                    let _ = tx.execute(
                        "INSERT OR REPLACE INTO books (id, title, author, publisher, pub_year, isbn, category, location, price, description, cover_url, status, borrow_count, enrichment_status, note, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '', '', 'available', ?10, 'none', ?11, ?12)",
                        params![id, title, author, publisher, pub_year, isbn, category, location, price, lend_sum, note, today],
                    );
                    count += 1;
                }
            }
            summary.push(format!("書籍資料匯入 {} 筆", count));
        }
    }

    // 4. 目前借出 Liborrow.DBF
    let liborrow_path = data_dir.join("Liborrow.DBF");
    if liborrow_path.exists() {
        println!("[-] 正在讀取 Liborrow.DBF（目前在借記錄）...");
        if let Ok(table) = read_dbf(&liborrow_path) {
            let mut count = 0;
            for rec in table.records {
                let reader_id = rec.get("ID").cloned().unwrap_or_default();
                let book_id = rec.get("ACSNO").cloned().unwrap_or_default();
                let date1 = format_dbf_date(&rec.get("DATE1").cloned().unwrap_or_default());
                let date2 = format_dbf_date(&rec.get("DATE2").cloned().unwrap_or_default());

                if !reader_id.is_empty() && !book_id.is_empty() {
                    let _ = tx.execute(
                        "UPDATE books SET status = 'borrowed' WHERE id = ?1",
                        params![book_id],
                    );

                    let exists: i64 = tx.query_row(
                        "SELECT COUNT(*) FROM loans WHERE book_id = ?1 AND reader_id = ?2 AND status = 'active'",
                        params![book_id, reader_id],
                        |r| r.get(0),
                    ).unwrap_or(0);

                    if exists == 0 {
                        let _ = tx.execute(
                            "INSERT INTO loans (book_id, reader_id, borrow_date, due_date, status, renew_count, note)
                             VALUES (?1, ?2, ?3, ?4, 'active', 0, '從舊系統匯入在借記錄')",
                            params![book_id, reader_id, date1, date2],
                        );
                        count += 1;
                    }
                }
            }
            summary.push(format!("目前在借記錄匯入 {} 筆", count));
        }
    }

    // 5. 歷史借還記錄 Liborrow_his.DBF
    let his_path = data_dir.join("Liborrow_his.DBF");
    if his_path.exists() {
        println!("[-] 正在讀取 Liborrow_his.DBF（歷史借閱記錄，筆數較多請耐心等待）...");
        if let Ok(table) = read_dbf(&his_path) {
            let mut count = 0;
            for rec in table.records {
                let reader_id = rec.get("ID").cloned().unwrap_or_default();
                let book_id = rec.get("ACSNO").cloned().unwrap_or_default();
                let date1 = format_dbf_date(&rec.get("DATE1").cloned().unwrap_or_default());
                let date2 = format_dbf_date(&rec.get("DATE2").cloned().unwrap_or_default());
                let date3 = format_dbf_date(&rec.get("DATE3").cloned().unwrap_or_default());

                if !reader_id.is_empty() && !book_id.is_empty() {
                    let _ = tx.execute(
                        "INSERT INTO loans (book_id, reader_id, borrow_date, due_date, return_date, status, renew_count, note)
                         VALUES (?1, ?2, ?3, ?4, ?5, 'returned', 0, '歷史借閱記錄')",
                        params![book_id, reader_id, date1, date2, if date3.is_empty() { date2.clone() } else { date3 }],
                    );
                    count += 1;
                }
            }
            summary.push(format!("歷史借閱記錄匯入 {} 筆", count));
        }
    }

    tx.commit()
        .map_err(|e| format!("提交移轉交易失敗: {}", e))?;

    Ok(summary.join("，"))
}

// === 角色操作 ===

pub fn get_roles(db: &DbPool) -> Result<Vec<Role>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, max_books, borrow_days, renew_days, note FROM roles ORDER BY id ASC")?;
    let roles = stmt.query_map([], |row| {
        Ok(Role {
            id: row.get(0)?,
            name: row.get(1)?,
            max_books: row.get(2)?,
            borrow_days: row.get(3)?,
            renew_days: row.get(4)?,
            note: row.get(5)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(roles)
}

pub fn save_role(db: &DbPool, role: &Role) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO roles (id, name, max_books, borrow_days, renew_days, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![role.id, role.name, role.max_books, role.borrow_days, role.renew_days, role.note],
    ).map_err(|e| format!("儲存身分別失敗: {}", e))?;
    Ok(())
}

pub fn delete_role(db: &DbPool, role_id: &str) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM readers WHERE role_id = ?1",
        params![role_id],
        |r| r.get(0),
    ).unwrap_or(0);

    if count > 0 {
        return Err(format!("無法刪除：尚有 {} 位讀者使用此身分別！請先修改該讀者身分。", count));
    }

    conn.execute("DELETE FROM roles WHERE id = ?1", params![role_id])
        .map_err(|e| format!("刪除失敗: {}", e))?;
    Ok(())
}

// === 讀者操作 ===

pub fn get_readers(db: &DbPool, query: &str) -> Result<Vec<Reader>> {
    let conn = db.lock().unwrap();
    let trimmed = query.trim();
    let map_row = |row: &rusqlite::Row<'_>| -> Result<Reader> {
        Ok(Reader {
            id: row.get(0)?,
            name: row.get(1)?,
            role_id: row.get(2)?,
            role_name: row.get(3)?,
            max_books: row.get(4)?,
            borrow_days: row.get(5)?,
            class_name: row.get(6)?,
            grade: row.get(7)?,
            phone: row.get(8)?,
            email: row.get(9)?,
            status: row.get(10)?,
            note: row.get(11)?,
            created_at: row.get(12)?,
            active_loans_count: row.get(13)?,
        })
    };

    let readers = if trimmed.is_empty() {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.name, r.role_id, ro.name, ro.max_books, ro.borrow_days,
                    r.class_name, r.grade, r.phone, r.email, r.status, r.note, r.created_at,
                    (SELECT COUNT(*) FROM loans l WHERE l.reader_id = r.id AND l.status = 'active') as active_loans
             FROM readers r
             LEFT JOIN roles ro ON r.role_id = ro.id
             ORDER BY r.id ASC"
        )?;
        let rows: Vec<Reader> = stmt.query_map([], map_row)?.filter_map(|r| r.ok()).collect();
        rows
    } else {
        let q = format!("%{}%", trimmed);
        let mut stmt = conn.prepare(
            "SELECT r.id, r.name, r.role_id, ro.name, ro.max_books, ro.borrow_days,
                    r.class_name, r.grade, r.phone, r.email, r.status, r.note, r.created_at,
                    (SELECT COUNT(*) FROM loans l WHERE l.reader_id = r.id AND l.status = 'active') as active_loans
             FROM readers r
             LEFT JOIN roles ro ON r.role_id = ro.id
             WHERE r.id LIKE ?1 OR r.name LIKE ?1 OR r.class_name LIKE ?1 OR ro.name LIKE ?1 OR r.note LIKE ?1
             ORDER BY r.id ASC"
        )?;
        let rows: Vec<Reader> = stmt.query_map(params![q], map_row)?.filter_map(|r| r.ok()).collect();
        rows
    };

    Ok(readers)
}

pub fn get_reader_by_id(db: &DbPool, id: &str) -> Result<Option<Reader>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT r.id, r.name, r.role_id, ro.name, ro.max_books, ro.borrow_days,
                r.class_name, r.grade, r.phone, r.email, r.status, r.note, r.created_at,
                (SELECT COUNT(*) FROM loans l WHERE l.reader_id = r.id AND l.status = 'active') as active_loans
         FROM readers r
         LEFT JOIN roles ro ON r.role_id = ro.id
         WHERE r.id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Reader {
            id: row.get(0)?,
            name: row.get(1)?,
            role_id: row.get(2)?,
            role_name: row.get(3)?,
            max_books: row.get(4)?,
            borrow_days: row.get(5)?,
            class_name: row.get(6)?,
            grade: row.get(7)?,
            phone: row.get(8)?,
            email: row.get(9)?,
            status: row.get(10)?,
            note: row.get(11)?,
            created_at: row.get(12)?,
            active_loans_count: row.get(13)?,
        }))
    } else {
        Ok(None)
    }
}

/// 建立新讀者（若證號為空則延遲取號，在儲存時原子性派發；若已有證號則嚴格檢查避免覆蓋）
pub fn create_reader(db: &DbPool, mut reader: Reader) -> std::result::Result<Reader, String> {
    let conn = db.lock().unwrap();
    let today = if reader.created_at.is_empty() {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        reader.created_at.clone()
    };
    reader.created_at = today.clone();

    // 延遲取號：若未提供證號，則在儲存當下由後端自動產生
    if reader.id.trim().is_empty() {
        let mut candidate = get_next_reader_id_internal(&conn)?;
        let mut attempts = 0;
        while conn.query_row("SELECT 1 FROM readers WHERE id = ?1", params![candidate], |_| Ok(())).is_ok() {
            attempts += 1;
            if attempts > 1000 {
                return Err("無法自動產生未使用的讀者證號，請手動輸入證號！".to_string());
            }
            if let Ok(num) = candidate.parse::<u64>() {
                let width = candidate.len();
                candidate = format!("{:0>width$}", num + 1, width = width);
            } else {
                break;
            }
        }
        reader.id = candidate;
    } else {
        // 使用者手動指定證號：檢查是否已存在，嚴禁覆蓋已有讀者
        let exists: bool = conn.query_row(
            "SELECT 1 FROM readers WHERE id = ?1",
            params![reader.id],
            |_| Ok(()),
        ).is_ok();
        if exists {
            return Err(format!("新增失敗：讀者證號 [{}] 已經存在，請勿重複新增！", reader.id));
        }
    }

    conn.execute(
        "INSERT INTO readers (id, name, role_id, class_name, grade, phone, email, status, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            reader.id,
            reader.name,
            reader.role_id,
            reader.class_name,
            reader.grade,
            reader.phone,
            reader.email,
            reader.status,
            reader.note,
            today
        ],
    ).map_err(|e| format!("新增讀者資料失敗: {}", e))?;

    Ok(reader)
}

/// 更新既有讀者資料
pub fn update_reader(db: &DbPool, reader: &Reader) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let rows = conn.execute(
        "UPDATE readers SET name = ?2, role_id = ?3, class_name = ?4, grade = ?5, phone = ?6, email = ?7, status = ?8, note = ?9
         WHERE id = ?1",
        params![
            reader.id,
            reader.name,
            reader.role_id,
            reader.class_name,
            reader.grade,
            reader.phone,
            reader.email,
            reader.status,
            reader.note,
        ],
    ).map_err(|e| format!("更新讀者資料失敗: {}", e))?;

    if rows == 0 {
        return Err(format!("找不到證號為 [{}] 的讀者，無法更新！", reader.id));
    }
    Ok(())
}

pub fn delete_reader(db: &DbPool, id: &str) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let active_loans: i64 = conn.query_row(
        "SELECT COUNT(*) FROM loans WHERE reader_id = ?1 AND status = 'active'",
        params![id],
        |r| r.get(0),
    ).unwrap_or(0);

    if active_loans > 0 {
        return Err(format!("無法刪除讀者：該讀者目前尚有 {} 本書未還！請先完成歸還作業。", active_loans));
    }

    conn.execute("DELETE FROM readers WHERE id = ?1", params![id])
        .map_err(|e| format!("刪除讀者失敗: {}", e))?;
    Ok(())
}

/// 從連線中分析所有現有讀者證號，取得下一個建議證號（五位數字補零）
pub fn get_next_reader_id_internal(conn: &Connection) -> std::result::Result<String, String> {
    // 取所有 id，嘗試解析為數字，找最大值後 +1
    let mut stmt = conn.prepare("SELECT id FROM readers")
        .map_err(|e| e.to_string())?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let max_num = ids.iter()
        .filter_map(|id| {
            // 擷取尾端連續數字部分
            let digits: String = id.chars().rev().take_while(|c| c.is_ascii_digit()).collect::<String>()
                .chars().rev().collect();
            digits.parse::<u64>().ok()
        })
        .max()
        .unwrap_or(0);

    let next = max_num + 1;
    // 嘗試保持與現有格式相同的位數（至少 5 位）
    let width = ids.iter()
        .filter_map(|id| {
            let digits: String = id.chars().rev().take_while(|c| c.is_ascii_digit()).collect::<String>()
                .chars().rev().collect();
            if digits.len() >= 3 { Some(digits.len()) } else { None }
        })
        .max()
        .unwrap_or(5);

    Ok(format!("{:0>width$}", next, width = width))
}

/// 取得下一個建議讀者證號（格式：五位數字補零，例如 00106）
pub fn get_next_reader_id(db: &DbPool) -> std::result::Result<String, String> {
    let conn = db.lock().unwrap();
    get_next_reader_id_internal(&conn)
}

/// 從連線中分析所有現有書籍條碼，取得下一個建議條碼（保留前綴字母，數字部分 +1，補足原有位數）
pub fn get_next_book_id_internal(conn: &Connection) -> std::result::Result<String, String> {
    let mut stmt = conn.prepare("SELECT id FROM books")
        .map_err(|e| e.to_string())?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if ids.is_empty() {
        return Ok("B000001".to_string());
    }

    // 分析所有 ID，找到具有最大數字後綴的那一個
    let mut best_prefix = "".to_string();
    let mut best_num: u64 = 0;
    let mut best_num_len: usize = 1;

    for id in &ids {
        let rev: Vec<char> = id.chars().rev().collect();
        let digit_count = rev.iter().take_while(|c| c.is_ascii_digit()).count();
        if digit_count == 0 { continue; }
        let num_str: String = rev[..digit_count].iter().rev().collect();
        if let Ok(n) = num_str.parse::<u64>() {
            if n > best_num {
                best_num = n;
                best_num_len = digit_count;
                best_prefix = id[..id.len() - digit_count].to_string();
            }
        }
    }

    if best_num == 0 {
        return Ok("B000001".to_string());
    }

    let next = best_num + 1;
    // 保持與最大號相同的數字位數
    Ok(format!("{}{:0>width$}", best_prefix, next, width = best_num_len))
}

/// 取得下一個建議書籍條碼（保留前綴字母，數字部分 +1，補足原有位數）
pub fn get_next_book_id(db: &DbPool) -> std::result::Result<String, String> {
    let conn = db.lock().unwrap();
    get_next_book_id_internal(&conn)
}

// === 書籍大分類操作 ===

pub fn get_categories(db: &DbPool) -> Result<Vec<BookCategory>> {
    let conn = db.lock().unwrap();
    get_categories_internal(&conn)
}

fn get_categories_internal(conn: &Connection) -> Result<Vec<BookCategory>> {
    let mut stmt = conn.prepare(
        "SELECT code, name, sort_order, note FROM categories ORDER BY sort_order ASC, code ASC"
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BookCategory {
                code: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                note: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

pub fn upsert_category(db: &DbPool, cat: &BookCategory) -> std::result::Result<BookCategory, String> {
    let code = cat.code.trim().to_uppercase();
    let name = cat.name.trim().to_string();
    if code.is_empty() || name.is_empty() {
        return Err("分類代碼與分類名稱不可為空！".to_string());
    }
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO categories (code, name, sort_order, note) VALUES (?1, ?2, ?3, ?4)",
        params![code, name, cat.sort_order, cat.note.trim()],
    )
    .map_err(|e| format!("儲存大分類失敗: {}", e))?;

    Ok(BookCategory {
        code,
        name,
        sort_order: cat.sort_order,
        note: cat.note.trim().to_string(),
    })
}

pub fn delete_category(db: &DbPool, code: &str) -> std::result::Result<(), String> {
    let normalized = code.trim().to_uppercase();
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM categories WHERE code = ?1", params![normalized])
        .map_err(|e| format!("刪除大分類失敗: {}", e))?;
    Ok(())
}

/// 依據大分類代碼表（採最長前綴優先、不分大小寫）判別書籍分類號所屬大分類代碼
pub fn resolve_category_code<'a>(raw_category: &str, categories: &'a [BookCategory]) -> Option<&'a str> {
    let normalized = raw_category.trim().to_uppercase();
    if normalized.is_empty() {
        return None;
    }
    categories
        .iter()
        .filter(|c| {
            let code = c.code.trim().to_uppercase();
            !code.is_empty() && normalized.starts_with(&code)
        })
        .max_by_key(|c| c.code.trim().len())
        .map(|c| c.code.as_str())
}

// === 書籍操作 ===

pub fn get_books(db: &DbPool, query: &str, status_filter: &str, category_filter: &str) -> Result<Vec<Book>> {
    let conn = db.lock().unwrap();
    let trimmed = query.trim();
    let status_cond = match status_filter {
        "available" => Some("b.status = 'available'"),
        "borrowed" => Some("b.status = 'borrowed'"),
        _ => None,
    };

    let today = Local::now().naive_local().date();
    let map_row = |row: &rusqlite::Row<'_>| -> Result<Book> {
        let loan_id: Option<i64> = row.get(16)?;
        let current_borrower = if let Some(lid) = loan_id {
            let rid: String = row.get(17)?;
            let rname: String = row.get(18)?;
            let bdate: String = row.get(19)?;
            let ddate: String = row.get(20)?;

            let days_remaining = NaiveDate::parse_from_str(&ddate, "%Y-%m-%d")
                .map(|d| (d - today).num_days())
                .unwrap_or(0);

            Some(CurrentBorrowerInfo {
                reader_id: rid,
                reader_name: rname,
                borrow_date: bdate,
                due_date: ddate,
                days_remaining,
                is_overdue: days_remaining < 0,
                loan_id: lid,
            })
        } else {
            None
        };

        Ok(Book {
            id: row.get(0)?,
            title: row.get(1)?,
            author: row.get(2)?,
            publisher: row.get(3)?,
            pub_year: row.get(4)?,
            isbn: row.get(5)?,
            category: row.get(6)?,
            location: row.get(7)?,
            price: row.get(8)?,
            description: row.get(9)?,
            cover_url: row.get(10)?,
            status: row.get(11)?,
            borrow_count: row.get(12)?,
            enrichment_status: row.get(13)?,
            note: row.get(14)?,
            created_at: row.get(15)?,
            current_borrower,
        })
    };

    let books = if trimmed.is_empty() {
        let where_clause = match status_cond {
            Some(c) => format!("WHERE {}", c),
            None => String::new(),
        };
        let sql = format!(
            "SELECT b.id, b.title, b.author, b.publisher, b.pub_year, b.isbn, b.category,
                    b.location, b.price, b.description, b.cover_url, b.status, b.borrow_count,
                    b.enrichment_status, b.note, b.created_at,
                    l.id, l.reader_id, r.name, l.borrow_date, l.due_date
             FROM books b
             LEFT JOIN loans l ON b.id = l.book_id AND l.status = 'active'
             LEFT JOIN readers r ON l.reader_id = r.id
             {}
             ORDER BY b.id ASC",
            where_clause
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows: Vec<Book> = stmt.query_map([], map_row)?.filter_map(|b| b.ok()).collect();
        rows
    } else {
        let q = format!("%{}%", trimmed);
        let status_clause = match status_cond {
            Some(c) => format!(" AND {}", c),
            None => String::new(),
        };
        let sql = format!(
            "SELECT b.id, b.title, b.author, b.publisher, b.pub_year, b.isbn, b.category,
                    b.location, b.price, b.description, b.cover_url, b.status, b.borrow_count,
                    b.enrichment_status, b.note, b.created_at,
                    l.id, l.reader_id, r.name, l.borrow_date, l.due_date
             FROM books b
             LEFT JOIN loans l ON b.id = l.book_id AND l.status = 'active'
             LEFT JOIN readers r ON l.reader_id = r.id
             WHERE (b.id LIKE ?1 OR b.title LIKE ?1 OR b.author LIKE ?1 OR b.isbn LIKE ?1 OR b.category LIKE ?1)
             {}
             ORDER BY b.id ASC",
            status_clause
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows: Vec<Book> = stmt.query_map(params![q], map_row)?.filter_map(|b| b.ok()).collect();
        rows
    };

    let cat_filter = category_filter.trim();
    if cat_filter.is_empty() {
        return Ok(books);
    }

    let categories = get_categories_internal(&conn).unwrap_or_default();
    let filtered: Vec<Book> = match cat_filter {
        "__EMPTY__" => books
            .into_iter()
            .filter(|b| b.category.trim().is_empty())
            .collect(),
        "__OTHER__" => books
            .into_iter()
            .filter(|b| {
                !b.category.trim().is_empty()
                    && resolve_category_code(&b.category, &categories).is_none()
            })
            .collect(),
        target_code => {
            let target_upper = target_code.to_uppercase();
            books
                .into_iter()
                .filter(|b| {
                    resolve_category_code(&b.category, &categories)
                        .map(|c| c.eq_ignore_ascii_case(&target_upper))
                        .unwrap_or(false)
                })
                .collect()
        }
    };

    Ok(filtered)
}

pub fn get_book_by_id(db: &DbPool, id: &str) -> Result<Option<Book>> {
    let conn = db.lock().unwrap();
    let sql = "
        SELECT b.id, b.title, b.author, b.publisher, b.pub_year, b.isbn, b.category,
               b.location, b.price, b.description, b.cover_url, b.status, b.borrow_count,
               b.enrichment_status, b.note, b.created_at,
               l.id, l.reader_id, r.name, l.borrow_date, l.due_date
        FROM books b
        LEFT JOIN loans l ON b.id = l.book_id AND l.status = 'active'
        LEFT JOIN readers r ON l.reader_id = r.id
        WHERE b.id = ?1";

    let mut stmt = conn.prepare(sql)?;
    let today = Local::now().naive_local().date();

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        let loan_id: Option<i64> = row.get(16)?;
        let current_borrower = if let Some(lid) = loan_id {
            let rid: String = row.get(17)?;
            let rname: String = row.get(18)?;
            let bdate: String = row.get(19)?;
            let ddate: String = row.get(20)?;

            let days_remaining = NaiveDate::parse_from_str(&ddate, "%Y-%m-%d")
                .map(|d| (d - today).num_days())
                .unwrap_or(0);

            Some(CurrentBorrowerInfo {
                reader_id: rid,
                reader_name: rname,
                borrow_date: bdate,
                due_date: ddate,
                days_remaining,
                is_overdue: days_remaining < 0,
                loan_id: lid,
            })
        } else {
            None
        };

        Ok(Some(Book {
            id: row.get(0)?,
            title: row.get(1)?,
            author: row.get(2)?,
            publisher: row.get(3)?,
            pub_year: row.get(4)?,
            isbn: row.get(5)?,
            category: row.get(6)?,
            location: row.get(7)?,
            price: row.get(8)?,
            description: row.get(9)?,
            cover_url: row.get(10)?,
            status: row.get(11)?,
            borrow_count: row.get(12)?,
            enrichment_status: row.get(13)?,
            note: row.get(14)?,
            created_at: row.get(15)?,
            current_borrower,
        }))
    } else {
        Ok(None)
    }
}

/// 建立新書籍（若條碼為空則延遲取號，在儲存時原子性派發；若已有條碼則嚴格檢查避免覆蓋）
pub fn create_book(db: &DbPool, mut book: Book) -> std::result::Result<Book, String> {
    let conn = db.lock().unwrap();
    let today = if book.created_at.is_empty() {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        book.created_at.clone()
    };
    book.created_at = today.clone();

    // 延遲取號：若未提供條碼，則在儲存當下由後端自動產生
    if book.id.trim().is_empty() {
        let mut candidate = get_next_book_id_internal(&conn)?;
        let mut attempts = 0;
        while conn.query_row("SELECT 1 FROM books WHERE id = ?1", params![candidate], |_| Ok(())).is_ok() {
            attempts += 1;
            if attempts > 1000 {
                return Err("無法自動產生未使用的書籍條碼，請手動輸入條碼！".to_string());
            }
            let rev: Vec<char> = candidate.chars().rev().collect();
            let digit_count = rev.iter().take_while(|c| c.is_ascii_digit()).count();
            if digit_count > 0 {
                let num_str: String = rev[..digit_count].iter().rev().collect();
                if let Ok(n) = num_str.parse::<u64>() {
                    let prefix = &candidate[..candidate.len() - digit_count];
                    candidate = format!("{}{:0>width$}", prefix, n + 1, width = digit_count);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        book.id = candidate;
    } else {
        // 使用者手動指定條碼：檢查是否已存在，嚴禁覆蓋已有書籍
        let exists: bool = conn.query_row(
            "SELECT 1 FROM books WHERE id = ?1",
            params![book.id],
            |_| Ok(()),
        ).is_ok();
        if exists {
            return Err(format!("新增失敗：書籍條碼 [{}] 已經存在，請勿重複新增！", book.id));
        }
    }

    conn.execute(
        "INSERT INTO books (id, title, author, publisher, pub_year, isbn, category, location, price, description, cover_url, status, borrow_count, enrichment_status, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            book.id,
            book.title,
            book.author,
            book.publisher,
            book.pub_year,
            book.isbn,
            book.category,
            book.location,
            book.price,
            book.description,
            book.cover_url,
            book.status,
            book.borrow_count,
            book.enrichment_status,
            book.note,
            today
        ],
    ).map_err(|e| format!("新增書籍失敗: {}", e))?;

    Ok(book)
}

/// 更新既有書籍資料（僅更新可修改的資訊，不覆蓋借閱狀態與計數）
pub fn update_book(db: &DbPool, book: &Book) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let rows = conn.execute(
        "UPDATE books SET
            title = ?2,
            author = ?3,
            publisher = ?4,
            isbn = ?5,
            category = ?6,
            location = ?7,
            description = ?8,
            cover_url = (CASE WHEN ?9 != '' THEN ?9 ELSE cover_url END),
            note = ?10
         WHERE id = ?1",
        params![
            book.id,
            book.title,
            book.author,
            book.publisher,
            book.isbn,
            book.category,
            book.location,
            book.description,
            book.cover_url,
            book.note,
        ],
    ).map_err(|e| format!("更新書籍失敗: {}", e))?;

    if rows == 0 {
        return Err(format!("找不到書籍條碼 [{}]，無法更新！", book.id));
    }
    Ok(())
}

pub fn batch_save_books(db: &DbPool, books: &[Book]) -> std::result::Result<usize, String> {
    let mut conn = db.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut count = 0;

    for book in books {
        let _ = tx.execute(
            "INSERT OR REPLACE INTO books (id, title, author, publisher, pub_year, isbn, category, location, price, description, cover_url, status, borrow_count, enrichment_status, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![book.id, book.title, book.author, book.publisher, book.pub_year, book.isbn, book.category, book.location, book.price, book.description, book.cover_url, book.status, book.borrow_count, book.enrichment_status, book.note, book.created_at],
        );
        count += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
}

pub fn batch_save_readers(db: &DbPool, readers: &[Reader]) -> std::result::Result<usize, String> {
    let mut conn = db.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut count = 0;

    for reader in readers {
        let _ = tx.execute(
            "INSERT OR REPLACE INTO readers (id, name, role_id, class_name, grade, phone, email, status, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![reader.id, reader.name, reader.role_id, reader.class_name, reader.grade, reader.phone, reader.email, reader.status, reader.note, reader.created_at],
        );
        count += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
}

pub fn delete_book(db: &DbPool, id: &str) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let active_loans: i64 = conn.query_row(
        "SELECT COUNT(*) FROM loans WHERE book_id = ?1 AND status = 'active'",
        params![id],
        |r| r.get(0),
    ).unwrap_or(0);

    if active_loans > 0 {
        return Err("無法刪除：該書目前借出中！請先完成歸還作業。".to_string());
    }

    conn.execute("DELETE FROM books WHERE id = ?1", params![id])
        .map_err(|e| format!("刪除書籍失敗: {}", e))?;
    Ok(())
}

// === 借還書作業核心邏輯 ===

pub fn borrow_book(db: &DbPool, reader_id: &str, book_id: &str) -> std::result::Result<LoanRecord, String> {
    let conn = db.lock().unwrap();

    let reader: Reader = conn.query_row(
        "SELECT r.id, r.name, r.role_id, ro.name, ro.max_books, ro.borrow_days,
                r.class_name, r.grade, r.phone, r.email, r.status, r.note, r.created_at,
                (SELECT COUNT(*) FROM loans l WHERE l.reader_id = r.id AND l.status = 'active') as active_loans
         FROM readers r
         JOIN roles ro ON r.role_id = ro.id
         WHERE r.id = ?1",
        params![reader_id],
        |row| {
            Ok(Reader {
                id: row.get(0)?,
                name: row.get(1)?,
                role_id: row.get(2)?,
                role_name: row.get(3)?,
                max_books: row.get(4)?,
                borrow_days: row.get(5)?,
                class_name: row.get(6)?,
                grade: row.get(7)?,
                phone: row.get(8)?,
                email: row.get(9)?,
                status: row.get(10)?,
                note: row.get(11)?,
                created_at: row.get(12)?,
                active_loans_count: row.get(13)?,
            })
        },
    ).map_err(|_| format!("找不到讀者證號 [{}] 或讀者身分別設定異常！", reader_id))?;

    if reader.status != "active" {
        return Err(format!("讀者 [{}] 帳號狀態為停用，無法借書！", reader.name));
    }

    let today = Local::now().naive_local().date();
    let today_str = today.format("%Y-%m-%d").to_string();

    // 檢查是否有逾期未歸還圖書
    let overdue_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM loans WHERE reader_id = ?1 AND status = 'active' AND due_date < ?2",
        params![reader_id, today_str],
        |r| r.get(0),
    ).unwrap_or(0);

    if overdue_count > 0 {
        return Err(format!(
            "借閱失敗：讀者 [{}] 尚有 {} 本逾期圖書未歸還！在還清逾期圖書前暫停借閱新書。",
            reader.name, overdue_count
        ));
    }

    let active_count = reader.active_loans_count.unwrap_or(0);
    let max_books = reader.max_books.unwrap_or(5);

    if active_count >= max_books {
        return Err(format!(
            "借閱失敗：讀者 [{}] 身分為「{}」，可借上限為 {} 本，目前已借 {} 本，已達借閱上限！",
            reader.name,
            reader.role_name.unwrap_or_default(),
            max_books,
            active_count
        ));
    }

    let (book_title, book_status): (String, String) = conn.query_row(
        "SELECT title, status FROM books WHERE id = ?1",
        params![book_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|_| format!("找不到書籍條碼/登錄號 [{}]！", book_id))?;

    if book_status != "available" {
        if book_status == "borrowed" {
            let borrower_info: String = conn.query_row(
                "SELECT r.name || ' (證號:' || r.id || ')' FROM loans l JOIN readers r ON l.reader_id = r.id WHERE l.book_id = ?1 AND l.status = 'active'",
                params![book_id],
                |r| r.get(0),
            ).unwrap_or_else(|_| "其他讀者".to_string());
            return Err(format!("借閱失敗：該書《{}》已被借出！目前借閱人為：{}", book_title, borrower_info));
        } else {
            return Err(format!("借閱失敗：該書《{}》狀態為「{}」，目前無法借閱！", book_title, book_status));
        }
    }

    let borrow_days = reader.borrow_days.unwrap_or(14);
    let due_date = today + chrono::Duration::days(borrow_days as i64);

    let borrow_date_str = today_str;
    let due_date_str = due_date.format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT INTO loans (book_id, reader_id, borrow_date, due_date, status, renew_count, note)
         VALUES (?1, ?2, ?3, ?4, 'active', 0, '')",
        params![book_id, reader_id, borrow_date_str, due_date_str],
    ).map_err(|e| format!("建立借閱記錄失敗: {}", e))?;

    let loan_id = conn.last_insert_rowid();

    conn.execute(
        "UPDATE books SET status = 'borrowed', borrow_count = borrow_count + 1 WHERE id = ?1",
        params![book_id],
    ).map_err(|e| format!("更新書籍狀態失敗: {}", e))?;

    Ok(LoanRecord {
        id: loan_id,
        book_id: book_id.to_string(),
        book_title: Some(book_title),
        reader_id: reader_id.to_string(),
        reader_name: Some(reader.name),
        role_name: reader.role_name,
        borrow_date: borrow_date_str,
        due_date: due_date_str,
        return_date: None,
        status: "active".to_string(),
        renew_count: 0,
        note: "".to_string(),
        days_remaining: Some(borrow_days as i64),
        is_overdue: false,
        overdue_days: Some(0),
    })
}

pub fn return_book(db: &DbPool, book_id_opt: Option<&str>, loan_id_opt: Option<i64>) -> std::result::Result<LoanRecord, String> {
    let conn = db.lock().unwrap();
    let today = Local::now().naive_local().date();
    let today_str = today.format("%Y-%m-%d").to_string();

    let loan_query = if let Some(_lid) = loan_id_opt {
        "SELECT l.id, l.book_id, b.title, l.reader_id, r.name, ro.name, l.borrow_date, l.due_date, l.renew_count
         FROM loans l
         JOIN books b ON l.book_id = b.id
         JOIN readers r ON l.reader_id = r.id
         JOIN roles ro ON r.role_id = ro.id
         WHERE l.id = ?1 AND l.status = 'active'".to_string()
    } else if let Some(_bid) = book_id_opt {
        "SELECT l.id, l.book_id, b.title, l.reader_id, r.name, ro.name, l.borrow_date, l.due_date, l.renew_count
         FROM loans l
         JOIN books b ON l.book_id = b.id
         JOIN readers r ON l.reader_id = r.id
         JOIN roles ro ON r.role_id = ro.id
         WHERE l.book_id = ?1 AND l.status = 'active'
         ORDER BY l.id DESC LIMIT 1".to_string()
    } else {
        return Err("請提供書籍條碼或借閱記錄編號！".to_string());
    };

    let param = if let Some(lid) = loan_id_opt {
        lid.to_string()
    } else {
        book_id_opt.unwrap().to_string()
    };

    let (loan_id, book_id, book_title, reader_id, reader_name, role_name, borrow_date, due_date, renew_count): 
        (i64, String, String, String, String, String, String, String, i32) = conn.query_row(
        &loan_query,
        params![param],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
    ).map_err(|_| "查無此借出中的記錄，該書可能已歸還或不在借閱狀態！".to_string())?;

    let diff_days = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
        .map(|d| (today - d).num_days())
        .unwrap_or(0);
    let is_overdue = diff_days > 0;
    let overdue_days = if is_overdue { diff_days } else { 0 };
    let return_note = if is_overdue {
        format!("逾期歸還 {} 天", overdue_days)
    } else {
        "正常歸還".to_string()
    };

    conn.execute(
        "UPDATE loans SET return_date = ?1, status = 'returned', note = ?2 WHERE id = ?3",
        params![today_str, return_note, loan_id],
    ).map_err(|e| format!("更新借閱記錄失敗: {}", e))?;

    conn.execute(
        "UPDATE books SET status = 'available' WHERE id = ?1",
        params![book_id],
    ).map_err(|e| format!("更新書籍在館狀態失敗: {}", e))?;

    Ok(LoanRecord {
        id: loan_id,
        book_id,
        book_title: Some(book_title),
        reader_id,
        reader_name: Some(reader_name),
        role_name: Some(role_name),
        borrow_date,
        due_date,
        return_date: Some(today_str),
        status: "returned".to_string(),
        renew_count,
        note: return_note,
        days_remaining: None,
        is_overdue,
        overdue_days: Some(overdue_days),
    })
}

pub fn renew_book(db: &DbPool, loan_id: i64) -> std::result::Result<LoanRecord, String> {
    let conn = db.lock().unwrap();
    let today = Local::now().naive_local().date();

    let (book_id, book_title, reader_id, reader_name, role_name, borrow_date, due_date, renew_count, renew_days):
        (String, String, String, String, String, String, String, i32, i32) = conn.query_row(
        "SELECT l.book_id, b.title, l.reader_id, r.name, ro.name, l.borrow_date, l.due_date, l.renew_count, ro.renew_days
         FROM loans l
         JOIN books b ON l.book_id = b.id
         JOIN readers r ON l.reader_id = r.id
         JOIN roles ro ON r.role_id = ro.id
         WHERE l.id = ?1 AND l.status = 'active'",
        params![loan_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
    ).map_err(|_| "查無有效在借記錄！".to_string())?;

    let current_due = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
        .map_err(|_| "日期格式錯誤".to_string())?;

    if current_due < today {
        return Err("該書籍已逾期，無法線上續借，請至櫃台歸還！".to_string());
    }

    let new_due = current_due + chrono::Duration::days(renew_days as i64);
    let new_due_str = new_due.format("%Y-%m-%d").to_string();
    let new_renew_count = renew_count + 1;

    conn.execute(
        "UPDATE loans SET due_date = ?1, renew_count = ?2 WHERE id = ?3",
        params![new_due_str, new_renew_count, loan_id],
    ).map_err(|e| format!("更新續借資料失敗: {}", e))?;

    let days_remaining = (new_due - today).num_days();

    Ok(LoanRecord {
        id: loan_id,
        book_id,
        book_title: Some(book_title),
        reader_id,
        reader_name: Some(reader_name),
        role_name: Some(role_name),
        borrow_date,
        due_date: new_due_str,
        return_date: None,
        status: "active".to_string(),
        renew_count: new_renew_count,
        note: format!("已續借 {} 次", new_renew_count),
        days_remaining: Some(days_remaining),
        is_overdue: false,
        overdue_days: Some(0),
    })
}

pub fn get_active_loans(db: &DbPool, reader_id: Option<&str>) -> Result<Vec<LoanRecord>> {
    let conn = db.lock().unwrap();
    let today = Local::now().naive_local().date();

    let sql = match reader_id {
        Some(_) => "SELECT l.id, l.book_id, b.title, l.reader_id, r.name, ro.name, l.borrow_date, l.due_date, l.renew_count, l.note
                    FROM loans l
                    JOIN books b ON l.book_id = b.id
                    JOIN readers r ON l.reader_id = r.id
                    JOIN roles ro ON r.role_id = ro.id
                    WHERE l.status = 'active' AND l.reader_id = ?1
                    ORDER BY l.due_date ASC",
        None => "SELECT l.id, l.book_id, b.title, l.reader_id, r.name, ro.name, l.borrow_date, l.due_date, l.renew_count, l.note
                 FROM loans l
                 JOIN books b ON l.book_id = b.id
                 JOIN readers r ON l.reader_id = r.id
                 JOIN roles ro ON r.role_id = ro.id
                 WHERE l.status = 'active'
                 ORDER BY l.due_date ASC",
    };

    let map_fn = |row: &rusqlite::Row| {
        let due_date_str: String = row.get(7)?;
        let days_remaining = NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d")
            .map(|d| (d - today).num_days())
            .unwrap_or(0);

        Ok(LoanRecord {
            id: row.get(0)?,
            book_id: row.get(1)?,
            book_title: Some(row.get(2)?),
            reader_id: row.get(3)?,
            reader_name: Some(row.get(4)?),
            role_name: Some(row.get(5)?),
            borrow_date: row.get(6)?,
            due_date: due_date_str,
            return_date: None,
            status: "active".to_string(),
            renew_count: row.get(8)?,
            note: row.get(9)?,
            days_remaining: Some(days_remaining),
            is_overdue: days_remaining < 0,
            overdue_days: Some(if days_remaining < 0 { -days_remaining } else { 0 }),
        })
    };

    let mut stmt = conn.prepare(sql)?;
    let records: Vec<LoanRecord> = match reader_id {
        Some(rid) => stmt.query_map(params![rid], map_fn)?.filter_map(|r| r.ok()).collect(),
        None => stmt.query_map([], map_fn)?.filter_map(|r| r.ok()).collect(),
    };

    Ok(records)
}

pub fn get_loan_history(db: &DbPool, query: &str, limit: i64) -> Result<Vec<LoanRecord>> {
    let conn = db.lock().unwrap();
    let q = format!("%{}%", query.trim());

    let mut stmt = conn.prepare(
        "SELECT l.id, l.book_id, b.title, l.reader_id, r.name, ro.name, l.borrow_date, l.due_date, l.return_date, l.status, l.renew_count, l.note
         FROM loans l
         JOIN books b ON l.book_id = b.id
         JOIN readers r ON l.reader_id = r.id
         JOIN roles ro ON r.role_id = ro.id
         WHERE l.book_id LIKE ?1 OR b.title LIKE ?1 OR l.reader_id LIKE ?1 OR r.name LIKE ?1
         ORDER BY l.id DESC LIMIT ?2"
    )?;

    let records = stmt.query_map(params![q, limit], |row| {
        let due_date_str: String = row.get(7)?;
        let return_date_opt: Option<String> = row.get(8)?;
        let (is_overdue, overdue_days) = if let Some(ref ret) = return_date_opt {
            let d_due = NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d").ok();
            let d_ret = NaiveDate::parse_from_str(ret, "%Y-%m-%d").ok();
            if let (Some(due), Some(ret_d)) = (d_due, d_ret) {
                let diff = (ret_d - due).num_days();
                (diff > 0, Some(diff.max(0)))
            } else {
                (false, Some(0))
            }
        } else {
            (false, None)
        };

        Ok(LoanRecord {
            id: row.get(0)?,
            book_id: row.get(1)?,
            book_title: Some(row.get(2)?),
            reader_id: row.get(3)?,
            reader_name: Some(row.get(4)?),
            role_name: Some(row.get(5)?),
            borrow_date: row.get(6)?,
            due_date: due_date_str,
            return_date: return_date_opt,
            status: row.get(9)?,
            renew_count: row.get(10)?,
            note: row.get(11)?,
            days_remaining: None,
            is_overdue,
            overdue_days,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(records)
}

pub fn get_stats(db: &DbPool) -> Result<StatsSummary> {
    let conn = db.lock().unwrap();
    let today_str = Local::now().format("%Y-%m-%d").to_string();

    let total_books: i64 = conn.query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0)).unwrap_or(0);
    let available_books: i64 = conn.query_row("SELECT COUNT(*) FROM books WHERE status = 'available'", [], |r| r.get(0)).unwrap_or(0);
    let borrowed_books: i64 = conn.query_row("SELECT COUNT(*) FROM books WHERE status = 'borrowed'", [], |r| r.get(0)).unwrap_or(0);
    let total_readers: i64 = conn.query_row("SELECT COUNT(*) FROM readers", [], |r| r.get(0)).unwrap_or(0);
    let active_loans: i64 = conn.query_row("SELECT COUNT(*) FROM loans WHERE status = 'active'", [], |r| r.get(0)).unwrap_or(0);
    let overdue_loans: i64 = conn.query_row(
        "SELECT COUNT(*) FROM loans WHERE status = 'active' AND due_date < ?1",
        params![today_str],
        |r| r.get(0),
    ).unwrap_or(0);
    let total_history_loans: i64 = conn.query_row("SELECT COUNT(*) FROM loans", [], |r| r.get(0)).unwrap_or(0);

    // 分類統計（依大分類代碼表前綴歸類，未填為「未分類」、未定義代碼為「其他分類」）
    let categories = get_categories_internal(&conn).unwrap_or_default();
    let cat_name_map: std::collections::HashMap<&str, &str> = categories
        .iter()
        .map(|c| (c.code.as_str(), c.name.as_str()))
        .collect();
    let mut cat_stmt = conn.prepare("SELECT category, COUNT(*) FROM books GROUP BY category")?;
    let mut aggregated: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let raw_groups: Vec<(String, i64)> = cat_stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    for (raw_cat, cnt) in raw_groups {
        let label = if raw_cat.trim().is_empty() {
            "未分類".to_string()
        } else if let Some(code) = resolve_category_code(&raw_cat, &categories) {
            let name = cat_name_map.get(code).copied().unwrap_or("");
            format!("{} {}", code, name)
        } else {
            "其他分類".to_string()
        };
        *aggregated.entry(label).or_insert(0) += cnt;
    }

    let mut category_distribution: Vec<CategoryStat> = aggregated
        .into_iter()
        .map(|(category, count)| CategoryStat { category, count })
        .collect();
    category_distribution.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.category.cmp(&b.category)));

    // 熱門借閱排行 Top 8
    let mut top_stmt = conn.prepare("SELECT id, title, borrow_count FROM books ORDER BY borrow_count DESC LIMIT 8")?;
    let top_borrowed_books = top_stmt.query_map([], |r| {
        Ok(BookStat {
            id: r.get(0)?,
            title: r.get(1)?,
            borrow_count: r.get(2)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    // 最近歸還有逾期之歷史紀錄 Top 10
    let mut overdue_ret_stmt = conn.prepare(
        "SELECT l.id, l.reader_id, r.name, l.book_id, b.title, l.borrow_date, l.due_date, l.return_date
         FROM loans l
         JOIN books b ON l.book_id = b.id
         JOIN readers r ON l.reader_id = r.id
         WHERE l.status = 'returned' AND l.return_date IS NOT NULL AND l.return_date > l.due_date
         ORDER BY l.return_date DESC, l.id DESC
         LIMIT 10"
    )?;

    let recent_overdue_returns = overdue_ret_stmt.query_map([], |row| {
        let due_date_str: String = row.get(6)?;
        let return_date_str: String = row.get(7)?;

        let d_due = NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d").ok();
        let d_ret = NaiveDate::parse_from_str(&return_date_str, "%Y-%m-%d").ok();
        let overdue_days = match (d_due, d_ret) {
            (Some(due), Some(ret)) => (ret - due).num_days().max(1),
            _ => 1,
        };

        Ok(OverdueReturnRecord {
            loan_id: row.get(0)?,
            reader_id: row.get(1)?,
            reader_name: row.get(2)?,
            book_id: row.get(3)?,
            book_title: row.get(4)?,
            borrow_date: row.get(5)?,
            due_date: due_date_str,
            return_date: return_date_str,
            overdue_days,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(StatsSummary {
        total_books,
        available_books,
        borrowed_books,
        total_readers,
        active_loans,
        overdue_loans,
        total_history_loans,
        category_distribution,
        top_borrowed_books,
        recent_overdue_returns,
    })
}

/// 依時間範圍統計月份借閱人次與冊數，以及個人借閱排行與書籍被借排行
pub fn get_range_stats_report(
    db: &DbPool,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: i64,
) -> Result<RangeStatsReport> {
    let conn = db.lock().unwrap();
    let start = start_date.map(|s| s.trim()).filter(|s| !s.is_empty()).unwrap_or("");
    let end = end_date.map(|s| s.trim()).filter(|s| !s.is_empty()).unwrap_or("");
    let effective_limit = if limit <= 0 { 100_000 } else { limit };

    // 1. 月份借閱人次（同讀者同日計 1 人次）與借閱冊數
    let mut month_stmt = conn.prepare(
        "SELECT substr(l.borrow_date, 1, 7) AS ym,
                COUNT(DISTINCT l.reader_id || '|' || substr(l.borrow_date, 1, 10)) AS visits,
                COUNT(*) AS books
         FROM loans l
         WHERE length(l.borrow_date) >= 7
           AND (?1 = '' OR substr(l.borrow_date, 1, 10) >= ?1)
           AND (?2 = '' OR substr(l.borrow_date, 1, 10) <= ?2)
         GROUP BY ym
         ORDER BY ym ASC"
    )?;

    let raw_monthly: Vec<MonthlyBorrowStat> = month_stmt.query_map(params![start, end], |r| {
        Ok(MonthlyBorrowStat {
            month: r.get(0)?,
            borrow_visits: r.get(1)?,
            borrow_books: r.get(2)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    // 若使用者有指定起訖日期且跨度在 60 個月以內，補齊期間內無借閱的月份（數值為 0），使趨勢圖連續易讀
    let monthly_stats = fill_missing_months(&raw_monthly, start, end);

    let total_visits: i64 = monthly_stats.iter().map(|m| m.borrow_visits).sum();
    let total_books: i64 = monthly_stats.iter().map(|m| m.borrow_books).sum();

    // 2. 時間範圍內個人借閱冊數排行
    let mut reader_stmt = conn.prepare(
        "SELECT l.reader_id,
                COALESCE(r.name, l.reader_id) AS reader_name,
                COALESCE(ro.name, '') AS role_name,
                COALESCE(r.class_name, '') AS class_name,
                COUNT(DISTINCT substr(l.borrow_date, 1, 10)) AS visits,
                COUNT(*) AS books
         FROM loans l
         LEFT JOIN readers r ON l.reader_id = r.id
         LEFT JOIN roles ro ON r.role_id = ro.id
         WHERE length(l.borrow_date) >= 7
           AND (?1 = '' OR substr(l.borrow_date, 1, 10) >= ?1)
           AND (?2 = '' OR substr(l.borrow_date, 1, 10) <= ?2)
         GROUP BY l.reader_id
         ORDER BY books DESC, visits DESC, l.reader_id ASC
         LIMIT ?3"
    )?;

    let reader_rankings: Vec<ReaderBorrowRank> = reader_stmt.query_map(params![start, end, effective_limit], |r| {
        Ok(ReaderBorrowRank {
            reader_id: r.get(0)?,
            reader_name: r.get(1)?,
            role_name: r.get(2)?,
            class_name: r.get(3)?,
            borrow_visits: r.get(4)?,
            borrow_books: r.get(5)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    // 3. 時間範圍內書籍被借次數排行
    let mut book_stmt = conn.prepare(
        "SELECT l.book_id,
                COALESCE(b.title, l.book_id) AS title,
                COALESCE(b.author, '') AS author,
                COALESCE(b.category, '') AS category,
                COUNT(*) AS cnt
         FROM loans l
         LEFT JOIN books b ON l.book_id = b.id
         WHERE length(l.borrow_date) >= 7
           AND (?1 = '' OR substr(l.borrow_date, 1, 10) >= ?1)
           AND (?2 = '' OR substr(l.borrow_date, 1, 10) <= ?2)
         GROUP BY l.book_id
         ORDER BY cnt DESC, l.book_id ASC
         LIMIT ?3"
    )?;

    let book_rankings: Vec<BookBorrowRank> = book_stmt.query_map(params![start, end, effective_limit], |r| {
        Ok(BookBorrowRank {
            book_id: r.get(0)?,
            title: r.get(1)?,
            author: r.get(2)?,
            category: r.get(3)?,
            borrow_count: r.get(4)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(RangeStatsReport {
        start_date: start.to_string(),
        end_date: end.to_string(),
        limit,
        total_visits,
        total_books,
        monthly_stats,
        reader_rankings,
        book_rankings,
    })
}

fn fill_missing_months(raw: &[MonthlyBorrowStat], start: &str, end: &str) -> Vec<MonthlyBorrowStat> {
    if start.len() < 7 || end.len() < 7 {
        return raw.to_vec();
    }
    let parse_ym = |s: &str| -> Option<(i32, u32)> {
        let y: i32 = s.get(0..4)?.parse().ok()?;
        let m: u32 = s.get(5..7)?.parse().ok()?;
        if (1..=12).contains(&m) { Some((y, m)) } else { None }
    };

    let (Some((sy, sm)), Some((ey, em))) = (parse_ym(start), parse_ym(end)) else {
        return raw.to_vec();
    };

    let start_idx = sy * 12 + (sm as i32 - 1);
    let end_idx = ey * 12 + (em as i32 - 1);
    if end_idx < start_idx || (end_idx - start_idx) > 60 {
        return raw.to_vec();
    }

    let mut map = std::collections::HashMap::new();
    for item in raw {
        map.insert(item.month.as_str(), item);
    }

    let mut result = Vec::with_capacity((end_idx - start_idx + 1) as usize);
    for idx in start_idx..=end_idx {
        let y = idx.div_euclid(12);
        let m = idx.rem_euclid(12) + 1;
        let ym = format!("{:04}-{:02}", y, m);
        if let Some(existing) = map.get(ym.as_str()) {
            result.push((*existing).clone());
        } else {
            result.push(MonthlyBorrowStat {
                month: ym,
                borrow_visits: 0,
                borrow_books: 0,
            });
        }
    }
    result
}

// === 設定項目 ===

pub fn get_setting(db: &DbPool, key: &str, default_val: &str) -> String {
    let conn = db.lock().unwrap();
    conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |r| r.get(0))
        .unwrap_or_else(|_| default_val.to_string())
}

pub fn save_setting(db: &DbPool, key: &str, value: &str) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)", params![key, value])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// === 資料庫備份與還原 ===

pub const BACKUP_MAX_KEEP: usize = 10;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupFileInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: String,
    pub is_pre_restore: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupResult {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub pruned_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestoreResult {
    pub pre_restore_backup: String,
    pub restored_from: String,
    pub migrated_from_version: i32,
    pub current_schema_version: i32,
}

/// 產生單一完整的 SQLite 備份檔（透過 VACUUM INTO，避免 WAL 遺漏交易）
pub fn create_database_backup(
    db: &DbPool,
    backup_dir: &Path,
    prefix: &str,
    enforce_retention: bool,
) -> std::result::Result<BackupResult, String> {
    std::fs::create_dir_all(backup_dir)
        .map_err(|e| format!("無法建立備份目錄 {}: {}", backup_dir.display(), e))?;

    let now = Local::now();
    let base_name = format!("{}_{}.db", prefix, now.format("%Y%m%d_%H%M%S"));
    let mut target_path = backup_dir.join(&base_name);
    if target_path.exists() {
        let unique_name = format!("{}_{}_{}.db", prefix, now.format("%Y%m%d_%H%M%S"), now.timestamp_subsec_millis());
        target_path = backup_dir.join(unique_name);
    }

    let filename = target_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(base_name);

    {
        let conn = db.lock().unwrap();
        let target_str = target_path.to_string_lossy().to_string();
        conn.execute("VACUUM INTO ?1", params![target_str])
            .map_err(|e| format!("建立備份檔失敗: {}", e))?;
        if enforce_retention {
            let iso_now = now.to_rfc3339();
            let _ = conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('last_backup_at', ?1)",
                params![iso_now],
            );
        }
    }

    let size_bytes = std::fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0);

    // 自動保留最新 BACKUP_MAX_KEEP (10) 份常規備份，超過則刪除最舊者
    let mut pruned_count = 0usize;
    if enforce_retention {
        let mut regular_backups: Vec<(String, std::path::PathBuf)> = std::fs::read_dir(backup_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("library_backup_") && fname.ends_with(".db") {
                    Some((fname, entry.path()))
                } else {
                    None
                }
            })
            .collect();

        // 檔名含 YYYYMMDD_HHMMSS，降冪排序即由新到舊
        regular_backups.sort_by(|a, b| b.0.cmp(&a.0));
        if regular_backups.len() > BACKUP_MAX_KEEP {
            for (_, old_path) in regular_backups.into_iter().skip(BACKUP_MAX_KEEP) {
                if std::fs::remove_file(&old_path).is_ok() {
                    pruned_count += 1;
                }
            }
        }
    }

    Ok(BackupResult {
        filename,
        path: target_path.to_string_lossy().to_string(),
        size_bytes,
        created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        pruned_count,
    })
}

/// 列出備份目錄中的所有 .db 備份檔（由新到舊排序）
pub fn list_backup_files(backup_dir: &Path) -> Vec<BackupFileInfo> {
    let Ok(entries) = std::fs::read_dir(backup_dir) else {
        return vec![];
    };

    let mut list: Vec<(std::time::SystemTime, BackupFileInfo)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let filename = entry.file_name().to_string_lossy().to_string();
            let lower = filename.to_ascii_lowercase();
            if !lower.ends_with(".db") && !lower.contains(".db.bak_") {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
            let dt: chrono::DateTime<Local> = mtime.into();
            let is_pre_restore = filename.starts_with("library_before_restore_");
            Some((
                mtime,
                BackupFileInfo {
                    filename,
                    path: path.to_string_lossy().to_string(),
                    size_bytes: meta.len(),
                    modified_at: dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                    is_pre_restore,
                },
            ))
        })
        .collect();

    list.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.filename.cmp(&a.1.filename)));
    list.into_iter().map(|(_, info)| info).collect()
}

/// 驗證指定檔案是否為完整合法的圖書系統 SQLite 備份檔，並回傳其 schema user_version
pub fn validate_sqlite_backup_file(path: &Path) -> std::result::Result<i32, String> {
    if !path.exists() || !path.is_file() {
        return Err("找不到指定的備份檔案".to_string());
    }
    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY;
    let conn = Connection::open_with_flags(path, flags)
        .map_err(|e| format!("無法開啟備份檔（非有效的 SQLite 資料庫）: {}", e))?;

    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|e| format!("資料庫完整性檢查失敗: {}", e))?;
    if !integrity.eq_ignore_ascii_case("ok") {
        return Err(format!("備份資料庫已損毀：{}", integrity));
    }

    for required_table in &["roles", "readers", "books", "loans"] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                params![required_table],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            return Err(format!("此檔案缺少核心資料表 `{}`，非本系統有效備份檔！", required_table));
        }
    }

    let ver: i32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);
    Ok(ver)
}

/// 安全還原舊備份資料庫：
/// 1. 驗證備份檔完整性
/// 2. 還原前自動建立目前資料庫快照（library_before_restore_*.db）
/// 3. 保留目前最新的管理員帳號密碼（users）與登入工作階段（sessions），以及備份相關設定
/// 4. 還原後自動執行 init_core_tables + run_migrations，確保舊版本資料表格式自動升級至最新版
pub fn restore_database_from_file(
    db: &DbPool,
    db_path: &Path,
    source_backup: &Path,
    backup_dir: &Path,
) -> std::result::Result<RestoreResult, String> {
    // 1. 驗證來源備份檔
    let source_ver = validate_sqlite_backup_file(source_backup)?;

    // 2. 還原前先建立目前資料庫的安全快照
    let pre_snapshot = create_database_backup(db, backup_dir, "library_before_restore", false)?;

    let restored_from = source_backup
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| source_backup.to_string_lossy().to_string());

    // 3. 在互斥鎖保護下安全切換連線並覆寫主資料庫
    let mut conn_guard = db.lock().unwrap();

    // 3a. 讀取目前最新的管理員帳號密碼與登入工作階段，以及備份相關設定
    let preserved_users: Vec<(String, String, String)> = {
        let mut stmt = conn_guard
            .prepare("SELECT username, password_hash, created_at FROM users")
            .map_err(|e| format!("讀取目前帳號資料失敗: {}", e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| format!("讀取目前帳號資料失敗: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        rows
    };

    let preserved_sessions: Vec<(String, String, String)> = {
        let mut stmt = conn_guard
            .prepare("SELECT token, username, created_at FROM sessions")
            .map_err(|e| format!("讀取目前工作階段失敗: {}", e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| format!("讀取目前工作階段失敗: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        rows
    };

    let mut preserved_settings: Vec<(String, String)> = Vec::new();
    for key in &["backup_dir", "backup_reminder_cycle", "last_backup_at"] {
        if let Ok(val) = conn_guard.query_row::<String, _, _>(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        ) {
            preserved_settings.push(((*key).to_string(), val));
        }
    }

    // 3b. 將 WAL 刷寫回主檔，並暫時切換為記憶體連線以釋放 Windows 檔案鎖定
    let _ = conn_guard.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    let mem_conn = Connection::open_in_memory()
        .map_err(|e| format!("無法建立暫存連線: {}", e))?;
    let old_conn = std::mem::replace(&mut *conn_guard, mem_conn);
    drop(old_conn);

    // 移除舊的 WAL / SHM 檔案，避免與還原後的主檔混淆
    let mut wal_os = db_path.as_os_str().to_os_string();
    wal_os.push("-wal");
    let mut shm_os = db_path.as_os_str().to_os_string();
    shm_os.push("-shm");
    let _ = std::fs::remove_file(std::path::PathBuf::from(wal_os));
    let _ = std::fs::remove_file(std::path::PathBuf::from(shm_os));

    // 3c. 複製備份檔覆蓋 library.db（若失敗則嘗試重新開啟原資料庫）
    if let Err(e) = std::fs::copy(source_backup, db_path) {
        if let Ok(reopened) = Connection::open(db_path) {
            *conn_guard = reopened;
        }
        return Err(format!("複製備份檔覆蓋資料庫失敗: {}", e));
    }

    // 3d. 重新開啟還原後的資料庫並執行結構自檢與版本化自動升級（確保舊版備份格式相容）
    let mut new_conn = Connection::open(db_path)
        .map_err(|e| format!("重新開啟還原後資料庫失敗: {}", e))?;
    init_core_tables(&new_conn)
        .map_err(|e| format!("初始化還原後資料表結構失敗: {}", e))?;
    run_migrations(&mut new_conn, db_path, false)
        .map_err(|e| format!("自動升級還原後資料庫格式失敗: {}", e))?;

    // 3e. 寫回保留的最新管理員帳號密碼、登入工作階段與備份設定
    {
        let tx = new_conn
            .transaction()
            .map_err(|e| format!("啟動帳號保留交易失敗: {}", e))?;
        if !preserved_users.is_empty() {
            tx.execute("DELETE FROM users", [])
                .map_err(|e| format!("更新帳號資料表失敗: {}", e))?;
            for (u, hash, created_at) in &preserved_users {
                tx.execute(
                    "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
                    params![u, hash, created_at],
                )
                .map_err(|e| format!("還原保留帳號失敗: {}", e))?;
            }
        }
        if !preserved_sessions.is_empty() {
            tx.execute("DELETE FROM sessions", [])
                .map_err(|e| format!("更新工作階段失敗: {}", e))?;
            for (tok, u, created_at) in &preserved_sessions {
                tx.execute(
                    "INSERT INTO sessions (token, username, created_at) VALUES (?1, ?2, ?3)",
                    params![tok, u, created_at],
                )
                .map_err(|e| format!("還原保留工作階段失敗: {}", e))?;
            }
        }
        for (k, v) in &preserved_settings {
            let _ = tx.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![k, v],
            );
        }
        tx.commit()
            .map_err(|e| format!("提交帳號保留交易失敗: {}", e))?;
    }

    *conn_guard = new_conn;

    Ok(RestoreResult {
        pre_restore_backup: pre_snapshot.filename,
        restored_from,
        migrated_from_version: source_ver,
        current_schema_version: CURRENT_SCHEMA_VERSION,
    })
}

// === 帳號密碼認證與工作階段 (安全 bcrypt 加密) ===

pub fn verify_user_login(db: &DbPool, username: &str, password: &str) -> std::result::Result<bool, String> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT password_hash FROM users WHERE username = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![username]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let hash: String = row.get(0).map_err(|e| e.to_string())?;
        let is_valid = bcrypt::verify(password, &hash).unwrap_or(false);
        Ok(is_valid)
    } else {
        Ok(false)
    }
}

pub fn update_user_password(db: &DbPool, username: &str, current_pwd: &str, new_pwd: &str) -> std::result::Result<(), String> {
    if new_pwd.trim().is_empty() {
        return Err("新密碼不可為空！".to_string());
    }
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT password_hash FROM users WHERE username = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![username]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let hash: String = row.get(0).map_err(|e| e.to_string())?;
        if !bcrypt::verify(current_pwd, &hash).unwrap_or(false) {
            return Err("目前密碼不正確！".to_string());
        }
    } else {
        return Err("找不到該使用者帳號！".to_string());
    }

    let new_hash = bcrypt::hash(new_pwd, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("密碼加密雜湊失敗: {}", e))?;
    conn.execute(
        "UPDATE users SET password_hash = ?1 WHERE username = ?2",
        params![new_hash, username],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_session(db: &DbPool, token: &str, username: &str) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let now = Local::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO sessions (token, username, created_at) VALUES (?1, ?2, ?3)",
        params![token, username, now],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn validate_session(db: &DbPool, token: &str) -> Option<String> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT username FROM sessions WHERE token = ?1",
        params![token],
        |r| r.get(0),
    ).ok()
}

pub fn delete_session(db: &DbPool, token: &str) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM sessions WHERE token = ?1", params![token])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_fails_when_reader_has_overdue_loans() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_lib_{}.db", fastrand::u64(..)));
        let pool = init_db(&db_path).unwrap();

        // 建立測試讀者
        let reader = Reader {
            id: "RTEST01".to_string(),
            name: "測試讀者".to_string(),
            role_id: "01".to_string(),
            role_name: None,
            max_books: Some(5),
            borrow_days: Some(14),
            class_name: "".to_string(),
            grade: "".to_string(),
            phone: "".to_string(),
            email: "".to_string(),
            status: "active".to_string(),
            note: "".to_string(),
            created_at: "2026-11-14".to_string(),
            active_loans_count: None,
        };
        create_reader(&pool, reader).unwrap();

        // 建立測試書籍
        let b1 = Book {
            id: "BTEST01".to_string(),
            isbn: "".to_string(),
            title: "測試書一".to_string(),
            author: "".to_string(),
            publisher: "".to_string(),
            pub_year: "".to_string(),
            price: 100.0,
            category: "857".to_string(),
            location: "A01".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            description: "".to_string(),
            cover_url: "".to_string(),
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "2026-11-14".to_string(),
            current_borrower: None,
        };
        create_book(&pool, b1).unwrap();

        let b2 = Book {
            id: "BTEST02".to_string(),
            isbn: "".to_string(),
            title: "測試書二".to_string(),
            author: "".to_string(),
            publisher: "".to_string(),
            pub_year: "".to_string(),
            price: 150.0,
            category: "857".to_string(),
            location: "A01".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            description: "".to_string(),
            cover_url: "".to_string(),
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "2026-11-14".to_string(),
            current_borrower: None,
        };
        create_book(&pool, b2).unwrap();

        // 插入一筆逾期借閱 (到期日設為 2020-01-01)
        {
            let conn = pool.lock().unwrap();
            conn.execute(
                "INSERT INTO loans (book_id, reader_id, borrow_date, due_date, status, renew_count, note)
                 VALUES ('BTEST01', 'RTEST01', '2019-12-01', '2020-01-01', 'active', 0, '')",
                [],
            ).unwrap();
            conn.execute("UPDATE books SET status = 'borrowed' WHERE id = 'BTEST01'", []).unwrap();
        }

        // 讀者借第二本書，應該被拒絕，提示逾期
        let res = borrow_book(&pool, "RTEST01", "BTEST02");
        assert!(res.is_err(), "借新書應該失敗");
        let err_msg = res.err().unwrap();
        assert!(err_msg.contains("逾期"), "錯誤訊息應包含逾期提示: {}", err_msg);

        // 測試 get_active_loans 依讀者篩選
        let loans = get_active_loans(&pool, Some("RTEST01")).unwrap();
        assert_eq!(loans.len(), 1);
        assert_eq!(loans[0].book_id, "BTEST01");
        assert!(loans[0].is_overdue);

        // 歸還逾期圖書，測試計算逾期天數
        let ret_record = return_book(&pool, Some("BTEST01"), None).unwrap();
        assert!(ret_record.is_overdue);
        assert!(ret_record.overdue_days.unwrap() > 0);
        assert!(ret_record.note.contains("逾期歸還"));

        // 測試統計中之最近歸還逾期紀錄
        let stats = get_stats(&pool).unwrap();
        assert_eq!(stats.recent_overdue_returns.len(), 1);
        assert_eq!(stats.recent_overdue_returns[0].book_id, "BTEST01");
        assert_eq!(stats.recent_overdue_returns[0].reader_id, "RTEST01");
        assert!(stats.recent_overdue_returns[0].overdue_days > 0);

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_auth_default_and_password_change() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_auth_{}.db", fastrand::u64(..)));
        let pool = init_db(&db_path).unwrap();

        // 1. 預設帳密 admin / admin 登入成功
        assert!(verify_user_login(&pool, "admin", "admin").unwrap());
        // 錯誤密碼失敗
        assert!(!verify_user_login(&pool, "admin", "wrong_pass").unwrap());
        // 錯誤帳號失敗
        assert!(!verify_user_login(&pool, "nonexistent", "admin").unwrap());

        // 2. 驗證資料庫中密碼已加密，絕非明碼
        {
            let conn = pool.lock().unwrap();
            let hash: String = conn.query_row("SELECT password_hash FROM users WHERE username = 'admin'", [], |r| r.get(0)).unwrap();
            assert!(hash.starts_with("$2b$") || hash.starts_with("$2a$"), "密碼必須為 bcrypt 加密格式: {}", hash);
            assert_ne!(hash, "admin", "密碼不可為明碼！");
        }

        // 3. 變更密碼：舊密碼錯誤應失敗
        let err = update_user_password(&pool, "admin", "wrong_old", "new_admin_pass").unwrap_err();
        assert!(err.contains("目前密碼不正確"));

        // 舊密碼正確時更新成功
        assert!(update_user_password(&pool, "admin", "admin", "new_admin_pass").is_ok());

        // 4. 新密碼登入成功，舊密碼失效
        assert!(verify_user_login(&pool, "admin", "new_admin_pass").unwrap());
        assert!(!verify_user_login(&pool, "admin", "admin").unwrap());

        // 5. 測試 Session 建立與驗證
        let token = "test_token_123456";
        create_session(&pool, token, "admin").unwrap();
        assert_eq!(validate_session(&pool, token), Some("admin".to_string()));
        delete_session(&pool, token).unwrap();
        assert_eq!(validate_session(&pool, token), None);

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_delayed_id_allocation_and_duplicate_prevention() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_delayed_id_{}.db", fastrand::u64(..)));
        let pool = init_db(&db_path).unwrap();

        // 1. 測試書籍延遲取號（留空時系統自動派發）
        let book1 = Book {
            id: "".to_string(), // 空條碼
            title: "第一本書".to_string(),
            author: "作者A".to_string(),
            publisher: "出版社A".to_string(),
            pub_year: "".to_string(),
            isbn: "9781234567890".to_string(),
            category: "800".to_string(),
            location: "A01".to_string(),
            price: 200.0,
            description: "".to_string(),
            cover_url: "".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            current_borrower: None,
        };
        let saved_book1 = create_book(&pool, book1).unwrap();
        assert_eq!(saved_book1.id, "B000001");

        // 再次建立空條碼書籍，應自動遞增為 B000002
        let book2 = Book {
            id: "".to_string(),
            title: "第二本書".to_string(),
            author: "作者B".to_string(),
            publisher: "出版社B".to_string(),
            pub_year: "".to_string(),
            isbn: "".to_string(),
            category: "800".to_string(),
            location: "A01".to_string(),
            price: 250.0,
            description: "".to_string(),
            cover_url: "".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            current_borrower: None,
        };
        let saved_book2 = create_book(&pool, book2).unwrap();
        assert_eq!(saved_book2.id, "B000002");

        // 2. 測試手動輸入已存在條碼，必須報錯拒絕，不可覆蓋
        let dup_book = Book {
            id: "B000001".to_string(),
            title: "意圖覆蓋的書名".to_string(),
            author: "".to_string(),
            publisher: "".to_string(),
            pub_year: "".to_string(),
            isbn: "".to_string(),
            category: "".to_string(),
            location: "".to_string(),
            price: 0.0,
            description: "".to_string(),
            cover_url: "".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            current_borrower: None,
        };
        let dup_res = create_book(&pool, dup_book);
        assert!(dup_res.is_err());
        assert!(dup_res.unwrap_err().contains("已經存在"));

        // 驗證原 B000001 書籍書名依然是「第一本書」
        let check_b1 = get_book_by_id(&pool, "B000001").unwrap().unwrap();
        assert_eq!(check_b1.title, "第一本書");

        // 3. 測試讀者延遲取號
        let reader1 = Reader {
            id: "".to_string(), // 空證號
            name: "第一位讀者".to_string(),
            role_id: "01".to_string(),
            role_name: None,
            max_books: None,
            borrow_days: None,
            class_name: "資工一".to_string(),
            grade: "1".to_string(),
            phone: "".to_string(),
            email: "".to_string(),
            status: "active".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            active_loans_count: None,
        };
        let saved_r1 = create_reader(&pool, reader1).unwrap();
        assert_eq!(saved_r1.id, "00001");

        // 第二位讀者延遲取號
        let reader2 = Reader {
            id: "".to_string(),
            name: "第二位讀者".to_string(),
            role_id: "01".to_string(),
            role_name: None,
            max_books: None,
            borrow_days: None,
            class_name: "資工一".to_string(),
            grade: "1".to_string(),
            phone: "".to_string(),
            email: "".to_string(),
            status: "active".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            active_loans_count: None,
        };
        let saved_r2 = create_reader(&pool, reader2).unwrap();
        assert_eq!(saved_r2.id, "00002");

        // 手動重複新增已存在讀者證號，必須拒絕
        let dup_reader = Reader {
            id: "00001".to_string(),
            name: "意圖覆蓋的讀者".to_string(),
            role_id: "01".to_string(),
            role_name: None,
            max_books: None,
            borrow_days: None,
            class_name: "".to_string(),
            grade: "".to_string(),
            phone: "".to_string(),
            email: "".to_string(),
            status: "active".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            active_loans_count: None,
        };
        let dup_r_res = create_reader(&pool, dup_reader);
        assert!(dup_r_res.is_err());
        assert!(dup_r_res.unwrap_err().contains("已經存在"));

        // 4. 測試正常更新
        let mut r1_edit = check_b1;
        r1_edit.title = "第一本書(新版)".to_string();
        update_book(&pool, &r1_edit).unwrap();
        let check_updated = get_book_by_id(&pool, "B000001").unwrap().unwrap();
        assert_eq!(check_updated.title, "第一本書(新版)");

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_auto_migration_from_legacy_db() {
        let temp_dir = std::env::temp_dir().join(format!("test_migrate_{}", fastrand::u64(..)));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("library.db");

        // 1. 模擬非常舊版的資料庫：缺少 users/sessions/settings 資料表，也缺少多個後續新增欄位，且日期含斜線
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "
                CREATE TABLE roles (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL
                );
                CREATE TABLE readers (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    role_id TEXT NOT NULL
                );
                CREATE TABLE books (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL
                );
                CREATE TABLE loans (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    book_id TEXT NOT NULL,
                    reader_id TEXT NOT NULL,
                    borrow_date TEXT NOT NULL,
                    due_date TEXT NOT NULL
                );
                INSERT INTO roles (id, name) VALUES ('01', '舊版讀者');
                INSERT INTO readers (id, name, role_id) VALUES ('R001', '張三', '01');
                INSERT INTO books (id, title) VALUES ('B001', '舊版測試書');
                INSERT INTO loans (book_id, reader_id, borrow_date, due_date)
                    VALUES ('B001', 'R001', '2025/05/01', '2025/05/15');
                "
            ).unwrap();
        }

        // 2. 以最新版 init_db 開啟舊資料庫，應自動備份、補齊欄位/資料表/索引、正規化日期
        let pool = init_db(&db_path).unwrap();

        // 驗證升級前已建立自動備份檔
        let backups: Vec<_> = std::fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak_v0_to_v"))
            .collect();
        assert_eq!(backups.len(), 1, "升級舊資料庫前應自動建立 1 份備份檔");

        // 驗證 PRAGMA user_version 與 schema_migrations
        {
            let conn = pool.lock().unwrap();
            let ver: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
            assert_eq!(ver, CURRENT_SCHEMA_VERSION);
            let mig_count: i64 = conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0)).unwrap();
            assert_eq!(mig_count, CURRENT_SCHEMA_VERSION as i64);
        }

        // 驗證舊資料完整保留，且所有新版查詢函式皆能正常讀取
        let reader = get_reader_by_id(&pool, "R001").unwrap().unwrap();
        assert_eq!(reader.name, "張三");
        assert_eq!(reader.status, "active");

        let book = get_book_by_id(&pool, "B001").unwrap().unwrap();
        assert_eq!(book.title, "舊版測試書");
        assert_eq!(book.enrichment_status, "none");

        // 驗證斜線日期已正規化為 YYYY-MM-DD，且統計報表正常計算
        let report = get_range_stats_report(&pool, Some("2025-05-01"), Some("2025-05-31"), 10).unwrap();
        assert_eq!(report.total_books, 1);
        assert_eq!(report.total_visits, 1);
        assert_eq!(report.monthly_stats[0].month, "2025-05");

        // 3. 再次以 init_db 開啟已升級之資料庫，應具冪等性且不會重複產生備份檔
        drop(pool);
        let _pool2 = init_db(&db_path).unwrap();
        let backups_after: Vec<_> = std::fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak_v"))
            .collect();
        assert_eq!(backups_after.len(), 1, "已升級資料庫重新開啟時不應重複備份");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_backup_retention_and_restore_preserves_password_and_migrates() {
        let temp_dir = std::env::temp_dir().join(format!("test_backup_restore_{}", fastrand::u64(..)));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("library.db");
        let backup_dir = temp_dir.join("backups");

        let pool = init_db(&db_path).unwrap();

        // 1. 建立初始書籍 B001，並建立第一份備份
        let b1 = Book {
            id: "B001".to_string(),
            title: "原始書籍".to_string(),
            author: "".to_string(),
            publisher: "".to_string(),
            pub_year: "".to_string(),
            isbn: "".to_string(),
            category: "100".to_string(),
            location: "".to_string(),
            price: 100.0,
            description: "".to_string(),
            cover_url: "".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            current_borrower: None,
        };
        create_book(&pool, b1).unwrap();
        let first_backup = create_database_backup(&pool, &backup_dir, "library_backup", true).unwrap();
        assert!(std::path::Path::new(&first_backup.path).exists());

        // 2. 測試自動保留 10 份：先放入 11 個舊檔名模擬，再建立新備份，應只保留 10 份
        for i in 1..=11 {
            let fake_path = backup_dir.join(format!("library_backup_20250101_0000{:02}.db", i));
            std::fs::copy(&first_backup.path, &fake_path).unwrap();
        }
        let pruned_res = create_database_backup(&pool, &backup_dir, "library_backup", true).unwrap();
        assert!(pruned_res.pruned_count >= 2);
        let regular_count = list_backup_files(&backup_dir)
            .into_iter()
            .filter(|f| f.filename.starts_with("library_backup_"))
            .count();
        assert_eq!(regular_count, BACKUP_MAX_KEEP);

        // 3. 修改管理員密碼為新密碼，並新增第二本書 B002
        update_user_password(&pool, "admin", "admin", "new_secret_123").unwrap();
        let session_tok = "test_session_tok_123";
        create_session(&pool, session_tok, "admin").unwrap();
        let b2 = Book {
            id: "B002".to_string(),
            title: "後來新增的書".to_string(),
            author: "".to_string(),
            publisher: "".to_string(),
            pub_year: "".to_string(),
            isbn: "".to_string(),
            category: "200".to_string(),
            location: "".to_string(),
            price: 200.0,
            description: "".to_string(),
            cover_url: "".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            current_borrower: None,
        };
        create_book(&pool, b2).unwrap();
        assert!(get_book_by_id(&pool, "B002").unwrap().is_some());

        // 4. 還原回第一份備份（當時只有 B001、且密碼還是舊的 admin）
        let restore_res = restore_database_from_file(
            &pool,
            &db_path,
            std::path::Path::new(&first_backup.path),
            &backup_dir,
        )
        .unwrap();
        assert!(restore_res.pre_restore_backup.starts_with("library_before_restore_"));

        // 驗證資料已還原（B001 存在，B002 不存在）
        assert!(get_book_by_id(&pool, "B001").unwrap().is_some());
        assert!(get_book_by_id(&pool, "B002").unwrap().is_none());

        // 驗證管理員密碼與登入工作階段皆保留最新版本！
        assert!(verify_user_login(&pool, "admin", "new_secret_123").unwrap());
        assert!(!verify_user_login(&pool, "admin", "admin").unwrap());
        assert_eq!(validate_session(&pool, session_tok).as_deref(), Some("admin"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_book_categories_prefix_filter_and_custom_codes() {
        let temp_dir = std::env::temp_dir().join(format!("test_categories_{}", fastrand::u64(..)));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("library.db");
        let pool = init_db(&db_path).unwrap();

        // 預設應有 0~9 共 10 個大分類
        let cats = get_categories(&pool).unwrap();
        assert_eq!(cats.len(), 10);
        assert_eq!(cats[3].code, "3");
        assert_eq!(cats[3].name, "自然科學");

        // 新增自訂大分類 A 外國
        upsert_category(
            &pool,
            &BookCategory {
                code: "a".to_string(),
                name: "外國".to_string(),
                sort_order: 10,
                note: "外文書".to_string(),
            },
        )
        .unwrap();

        let make_book = |id: &str, cat: &str| Book {
            id: id.to_string(),
            title: format!("書 {}", id),
            author: "".to_string(),
            publisher: "".to_string(),
            pub_year: "".to_string(),
            isbn: "".to_string(),
            category: cat.to_string(),
            location: "".to_string(),
            price: 0.0,
            description: "".to_string(),
            cover_url: "".to_string(),
            status: "available".to_string(),
            borrow_count: 0,
            enrichment_status: "none".to_string(),
            note: "".to_string(),
            created_at: "".to_string(),
            current_borrower: None,
        };
        create_book(&pool, make_book("B1", "300.")).unwrap();
        create_book(&pool, make_book("B2", "310")).unwrap();
        create_book(&pool, make_book("B3", "a105")).unwrap();
        create_book(&pool, make_book("B4", "")).unwrap();
        create_book(&pool, make_book("B5", "Z99")).unwrap();

        let sci = get_books(&pool, "", "", "3").unwrap();
        assert_eq!(sci.len(), 2);
        let foreign = get_books(&pool, "", "", "A").unwrap();
        assert_eq!(foreign.len(), 1);
        assert_eq!(foreign[0].id, "B3");
        let empty = get_books(&pool, "", "", "__EMPTY__").unwrap();
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].id, "B4");
        let other = get_books(&pool, "", "", "__OTHER__").unwrap();
        assert_eq!(other.len(), 1);
        assert_eq!(other[0].id, "B5");

        // 驗證統計儀表板的藏書分類分佈也依大分類歸類彙總
        let stats = get_stats(&pool).unwrap();
        assert!(stats.category_distribution.iter().any(|c| c.category == "3 自然科學" && c.count == 2));
        assert!(stats.category_distribution.iter().any(|c| c.category == "A 外國" && c.count == 1));
        assert!(stats.category_distribution.iter().any(|c| c.category == "未分類" && c.count == 1));
        assert!(stats.category_distribution.iter().any(|c| c.category == "其他分類" && c.count == 1));

        delete_category(&pool, "A").unwrap();
        assert_eq!(get_categories(&pool).unwrap().len(), 10);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
