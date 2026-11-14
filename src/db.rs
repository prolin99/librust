use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use std::path::Path;
use chrono::{Local, NaiveDate};
use crate::models::*;
use crate::legacy::{read_dbf, format_dbf_date};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db<P: AsRef<Path>>(path: P) -> Result<DbPool> {
    let conn = Connection::open(path)?;
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

        CREATE INDEX IF NOT EXISTS idx_loans_active ON loans(status, reader_id, book_id);
        CREATE INDEX IF NOT EXISTS idx_books_status ON books(status);
        CREATE INDEX IF NOT EXISTS idx_readers_role ON readers(role_id);
        "
    )?;

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

pub fn import_legacy_data(db: &DbPool, data_dir: &Path) -> std::result::Result<String, String> {
    let conn = db.lock().unwrap();
    let mut summary = Vec::new();

    // 1. 身分別 DBF01.DBF
    let dbf01_path = data_dir.join("DBF01.DBF");
    if dbf01_path.exists() {
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
                    let _ = conn.execute(
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

    let role_count: i64 = conn.query_row("SELECT COUNT(*) FROM roles", [], |r| r.get(0)).unwrap_or(0);
    if role_count == 0 {
        let _ = conn.execute(
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
                    let _ = conn.execute(
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
                let category = rec.get("WHAT").cloned().unwrap_or_default();
                let location = rec.get("WHERE").cloned().unwrap_or_default();
                let price: f64 = rec.get("PRICE").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let lend_sum: i32 = rec.get("LEND_SUM").and_then(|v| v.parse().ok()).unwrap_or(0);
                let note = rec.get("PROPERTY").cloned().unwrap_or_default();

                if !id.is_empty() && !title.is_empty() {
                    let _ = conn.execute(
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
        if let Ok(table) = read_dbf(&liborrow_path) {
            let mut count = 0;
            for rec in table.records {
                let reader_id = rec.get("ID").cloned().unwrap_or_default();
                let book_id = rec.get("ACSNO").cloned().unwrap_or_default();
                let date1 = format_dbf_date(&rec.get("DATE1").cloned().unwrap_or_default());
                let date2 = format_dbf_date(&rec.get("DATE2").cloned().unwrap_or_default());

                if !reader_id.is_empty() && !book_id.is_empty() {
                    let _ = conn.execute(
                        "UPDATE books SET status = 'borrowed' WHERE id = ?1",
                        params![book_id],
                    );

                    let exists: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM loans WHERE book_id = ?1 AND reader_id = ?2 AND status = 'active'",
                        params![book_id, reader_id],
                        |r| r.get(0),
                    ).unwrap_or(0);

                    if exists == 0 {
                        let _ = conn.execute(
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
        if let Ok(table) = read_dbf(&his_path) {
            let mut count = 0;
            for rec in table.records {
                let reader_id = rec.get("ID").cloned().unwrap_or_default();
                let book_id = rec.get("ACSNO").cloned().unwrap_or_default();
                let date1 = format_dbf_date(&rec.get("DATE1").cloned().unwrap_or_default());
                let date2 = format_dbf_date(&rec.get("DATE2").cloned().unwrap_or_default());
                let date3 = format_dbf_date(&rec.get("DATE3").cloned().unwrap_or_default());

                if !reader_id.is_empty() && !book_id.is_empty() {
                    let _ = conn.execute(
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
    let q = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        "SELECT r.id, r.name, r.role_id, ro.name, ro.max_books, ro.borrow_days,
                r.class_name, r.grade, r.phone, r.email, r.status, r.note, r.created_at,
                (SELECT COUNT(*) FROM loans l WHERE l.reader_id = r.id AND l.status = 'active') as active_loans
         FROM readers r
         LEFT JOIN roles ro ON r.role_id = ro.id
         WHERE r.id LIKE ?1 OR r.name LIKE ?1 OR r.class_name LIKE ?1 OR ro.name LIKE ?1
         ORDER BY r.id ASC LIMIT 300"
    )?;

    let readers = stmt.query_map(params![q], |row| {
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
    })?.filter_map(|r| r.ok()).collect();

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

pub fn save_reader(db: &DbPool, reader: &Reader) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let today = if reader.created_at.is_empty() {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        reader.created_at.clone()
    };

    conn.execute(
        "INSERT OR REPLACE INTO readers (id, name, role_id, class_name, grade, phone, email, status, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![reader.id, reader.name, reader.role_id, reader.class_name, reader.grade, reader.phone, reader.email, reader.status, reader.note, today],
    ).map_err(|e| format!("儲存讀者資料失敗: {}", e))?;
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

/// 取得下一個建議讀者證號（格式：五位數字補零，例如 00106）
pub fn get_next_reader_id(db: &DbPool) -> std::result::Result<String, String> {
    let conn = db.lock().unwrap();
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

/// 取得下一個建議書籍條碼（保留前綴字母，數字部分 +1，補足原有位數）
pub fn get_next_book_id(db: &DbPool) -> std::result::Result<String, String> {
    let conn = db.lock().unwrap();
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

// === 書籍操作 ===

pub fn get_books(db: &DbPool, query: &str, status_filter: &str) -> Result<Vec<Book>> {
    let conn = db.lock().unwrap();
    let q = format!("%{}%", query.trim());
    let status_clause = match status_filter {
        "available" => " AND b.status = 'available'",
        "borrowed" => " AND b.status = 'borrowed'",
        _ => "",
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
         ORDER BY b.id ASC LIMIT 300",
        status_clause
    );

    let mut stmt = conn.prepare(&sql)?;
    let today = Local::now().naive_local().date();

    let books = stmt.query_map(params![q], |row| {
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
    })?.filter_map(|b| b.ok()).collect();

    Ok(books)
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

pub fn save_book(db: &DbPool, book: &Book) -> std::result::Result<(), String> {
    let conn = db.lock().unwrap();
    let today = if book.created_at.is_empty() {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        book.created_at.clone()
    };

    conn.execute(
        "INSERT OR REPLACE INTO books (id, title, author, publisher, pub_year, isbn, category, location, price, description, cover_url, status, borrow_count, enrichment_status, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![book.id, book.title, book.author, book.publisher, book.pub_year, book.isbn, book.category, book.location, book.price, book.description, book.cover_url, book.status, book.borrow_count, book.enrichment_status, book.note, today],
    ).map_err(|e| format!("儲存書籍失敗: {}", e))?;
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

    // 分類統計
    let mut cat_stmt = conn.prepare("SELECT COALESCE(NULLIF(category, ''), '未分類') as cat, COUNT(*) FROM books GROUP BY cat ORDER BY COUNT(*) DESC LIMIT 8")?;
    let category_distribution = cat_stmt.query_map([], |r| {
        Ok(CategoryStat {
            category: r.get(0)?,
            count: r.get(1)?,
        })
    })?.filter_map(|r| r.ok()).collect();

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
        save_reader(&pool, &reader).unwrap();

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
        save_book(&pool, &b1).unwrap();

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
        save_book(&pool, &b2).unwrap();

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
}
