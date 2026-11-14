use std::path::Path;
use std::fs::File;
use std::io::Read;
use calamine::{Reader, open_workbook_auto, Data};
use encoding_rs::BIG5;
use crate::models::{Book, Reader as ReaderModel};
use chrono::Local;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ImportResult {
    pub success_count: usize,
    pub skip_count: usize,
    pub errors: Vec<String>,
}

pub fn import_books_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Book>, String> {
    let path_ref = path.as_ref();
    let ext = path_ref.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    if ext == "xlsx" || ext == "xls" || ext == "ods" {
        import_books_from_excel(path_ref)
    } else if ext == "csv" || ext == "txt" {
        import_books_from_csv(path_ref)
    } else {
        Err("不支援的檔案格式，請上傳 .xlsx, .xls 或 .csv 檔案".to_string())
    }
}

pub fn import_readers_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<ReaderModel>, String> {
    let path_ref = path.as_ref();
    let ext = path_ref.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    if ext == "xlsx" || ext == "xls" || ext == "ods" {
        import_readers_from_excel(path_ref)
    } else if ext == "csv" || ext == "txt" {
        import_readers_from_csv(path_ref)
    } else {
        Err("不支援的檔案格式，請上傳 .xlsx, .xls 或 .csv 檔案".to_string())
    }
}

// === Excel 匯入 ===

fn import_books_from_excel(path: &Path) -> Result<Vec<Book>, String> {
    let mut workbook = open_workbook_auto(path).map_err(|e| format!("無法開啟 Excel 檔案: {}", e))?;
    let sheet_names = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err("Excel 內沒有任何工作表".to_string());
    }

    let first_sheet = sheet_names[0].clone();
    let range = workbook.worksheet_range(&first_sheet)
        .map_err(|e| format!("讀取工作表失敗: {}", e))?;

    let mut books = Vec::new();
    let mut headers: Vec<String> = Vec::new();
    let today = Local::now().format("%Y-%m-%d").to_string();

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            headers = row.iter().map(|c| cell_to_string(c).trim().to_string()).collect();
            continue;
        }

        let mut map = std::collections::HashMap::new();
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx < headers.len() {
                map.insert(headers[col_idx].clone(), cell_to_string(cell).trim().to_string());
            }
        }

        // 找尋對應欄位 (相容各種常見標題)
        let id = get_field(&map, &["條碼", "條碼號", "登錄號", "書籍條碼", "ID", "編號"]);
        let title = get_field(&map, &["書名", "書籍名稱", "名稱", "TITLE", "NAME"]);
        let author = get_field(&map, &["作者", "著者", "AUTHOR"]);
        let publisher = get_field(&map, &["出版社", "出版者", "出版商", "PUBLISHER", "PUBLICER"]);
        let pub_year = get_field(&map, &["出版年", "年份", "PUB_YEAR", "YEAR"]);
        let isbn = get_field(&map, &["ISBN", "國際標準書號", "條碼(ISBN)"]);
        let category = get_field(&map, &["分類", "類別", "分類號", "CATEGORY", "WHAT"]);
        let location = get_field(&map, &["館藏地", "地點", "存放位置", "櫃號", "LOCATION", "WHERE"]);
        let price_str = get_field(&map, &["定價", "價格", "金額", "PRICE"]);
        let price: f64 = price_str.parse().unwrap_or(0.0);
        let note = get_field(&map, &["備註", "NOTE", "說明"]);

        if !id.is_empty() && !title.is_empty() {
            books.push(Book {
                id,
                title,
                author,
                publisher,
                pub_year,
                isbn,
                category,
                location,
                price,
                description: "".to_string(),
                cover_url: "".to_string(),
                status: "available".to_string(),
                borrow_count: 0,
                enrichment_status: "none".to_string(),
                note,
                created_at: today.clone(),
                current_borrower: None,
            });
        }
    }

    Ok(books)
}

fn import_readers_from_excel(path: &Path) -> Result<Vec<ReaderModel>, String> {
    let mut workbook = open_workbook_auto(path).map_err(|e| format!("無法開啟 Excel 檔案: {}", e))?;
    let sheet_names = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err("Excel 內沒有工作表".to_string());
    }

    let first_sheet = sheet_names[0].clone();
    let range = workbook.worksheet_range(&first_sheet)
        .map_err(|e| format!("讀取工作表失敗: {}", e))?;

    let mut readers = Vec::new();
    let mut headers: Vec<String> = Vec::new();
    let today = Local::now().format("%Y-%m-%d").to_string();

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx == 0 {
            headers = row.iter().map(|c| cell_to_string(c).trim().to_string()).collect();
            continue;
        }

        let mut map = std::collections::HashMap::new();
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx < headers.len() {
                map.insert(headers[col_idx].clone(), cell_to_string(cell).trim().to_string());
            }
        }

        let id = get_field(&map, &["證號", "讀者證號", "借書證號", "學號", "工號", "ID"]);
        let name = get_field(&map, &["姓名", "讀者姓名", "NAME"]);
        let role_id = get_field(&map, &["身分別", "身分代號", "身分", "ROLE", "WHAT"]);
        let class_name = get_field(&map, &["班級", "科系", "部門", "CLASS"]);
        let grade = get_field(&map, &["年級", "級別", "YEAR", "GRADE"]);
        let phone = get_field(&map, &["電話", "手機", "PHONE"]);
        let email = get_field(&map, &["Email", "信箱", "EMAIL"]);
        let note = get_field(&map, &["備註", "NOTE"]);

        if !id.is_empty() && !name.is_empty() {
            readers.push(ReaderModel {
                id,
                name,
                role_id: if role_id.is_empty() { "01".to_string() } else { role_id },
                role_name: None,
                max_books: None,
                borrow_days: None,
                class_name,
                grade,
                phone,
                email,
                status: "active".to_string(),
                note,
                created_at: today.clone(),
                active_loans_count: Some(0),
            });
        }
    }

    Ok(readers)
}

// === CSV 匯入 (自動相容 UTF-8 與 Big5 / CP950) ===

fn read_file_with_encoding(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("無法開啟檔案: {}", e))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| format!("讀取檔案失敗: {}", e))?;

    // 嘗試以 UTF-8 解碼
    if let Ok(utf8_str) = String::from_utf8(bytes.clone()) {
        return Ok(utf8_str);
    }

    // 若非 UTF-8，以 Big5 / CP950 解碼 (相容舊系統 CSV)
    let (decoded, _, _) = BIG5.decode(&bytes);
    Ok(decoded.to_string())
}

fn import_books_from_csv(path: &Path) -> Result<Vec<Book>, String> {
    let text = read_file_with_encoding(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers: Vec<String> = rdr.headers()
        .map_err(|e| format!("CSV 格式解析錯誤: {}", e))?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    let mut books = Vec::new();
    let today = Local::now().format("%Y-%m-%d").to_string();

    for result in rdr.records() {
        if let Ok(record) = result {
            let mut map = std::collections::HashMap::new();
            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    map.insert(headers[i].clone(), field.trim().to_string());
                }
            }

            let id = get_field(&map, &["條碼", "條碼號", "登錄號", "書籍條碼", "ID", "編號"]);
            let title = get_field(&map, &["書名", "書籍名稱", "名稱", "TITLE", "NAME"]);
            let author = get_field(&map, &["作者", "著者", "AUTHOR"]);
            let publisher = get_field(&map, &["出版社", "出版者", "出版商", "PUBLISHER", "PUBLICER"]);
            let pub_year = get_field(&map, &["出版年", "年份", "PUB_YEAR", "YEAR"]);
            let isbn = get_field(&map, &["ISBN", "國際標準書號", "條碼(ISBN)"]);
            let category = get_field(&map, &["分類", "類別", "分類號", "CATEGORY", "WHAT"]);
            let location = get_field(&map, &["館藏地", "地點", "存放位置", "櫃號", "LOCATION", "WHERE"]);
            let price_str = get_field(&map, &["定價", "價格", "金額", "PRICE"]);
            let price: f64 = price_str.parse().unwrap_or(0.0);
            let note = get_field(&map, &["備註", "NOTE", "說明"]);

            if !id.is_empty() && !title.is_empty() {
                books.push(Book {
                    id,
                    title,
                    author,
                    publisher,
                    pub_year,
                    isbn,
                    category,
                    location,
                    price,
                    description: "".to_string(),
                    cover_url: "".to_string(),
                    status: "available".to_string(),
                    borrow_count: 0,
                    enrichment_status: "none".to_string(),
                    note,
                    created_at: today.clone(),
                    current_borrower: None,
                });
            }
        }
    }

    Ok(books)
}

fn import_readers_from_csv(path: &Path) -> Result<Vec<ReaderModel>, String> {
    let text = read_file_with_encoding(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(text.as_bytes());

    let headers: Vec<String> = rdr.headers()
        .map_err(|e| format!("CSV 格式解析錯誤: {}", e))?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    let mut readers = Vec::new();
    let today = Local::now().format("%Y-%m-%d").to_string();

    for result in rdr.records() {
        if let Ok(record) = result {
            let mut map = std::collections::HashMap::new();
            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    map.insert(headers[i].clone(), field.trim().to_string());
                }
            }

            let id = get_field(&map, &["證號", "讀者證號", "借書證號", "學號", "工號", "ID"]);
            let name = get_field(&map, &["姓名", "讀者姓名", "NAME"]);
            let role_id = get_field(&map, &["身分別", "身分代號", "身分", "ROLE", "WHAT"]);
            let class_name = get_field(&map, &["班級", "科系", "部門", "CLASS"]);
            let grade = get_field(&map, &["年級", "級別", "YEAR", "GRADE"]);
            let phone = get_field(&map, &["電話", "手機", "PHONE"]);
            let email = get_field(&map, &["Email", "信箱", "EMAIL"]);
            let note = get_field(&map, &["備註", "NOTE"]);

            if !id.is_empty() && !name.is_empty() {
                readers.push(ReaderModel {
                    id,
                    name,
                    role_id: if role_id.is_empty() { "01".to_string() } else { role_id },
                    role_name: None,
                    max_books: None,
                    borrow_days: None,
                    class_name,
                    grade,
                    phone,
                    email,
                    status: "active".to_string(),
                    note,
                    created_at: today.clone(),
                    active_loans_count: Some(0),
                });
            }
        }
    }

    Ok(readers)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => "".to_string(),
        Data::String(s) => s.to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::DateTime(d) => format!("{}", d),
        Data::DateTimeIso(s) => s.to_string(),
        Data::DurationIso(s) => s.to_string(),
        Data::Error(e) => format!("{:?}", e),
    }
}

fn get_field(map: &std::collections::HashMap<String, String>, candidates: &[&str]) -> String {
    for &c in candidates {
        if let Some(val) = map.get(c) {
            if !val.trim().is_empty() {
                return val.trim().to_string();
            }
        }
    }
    "".to_string()
}
