use std::fs::File;
use std::io::Read;
use std::path::Path;
use encoding_rs::BIG5;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DbfField {
    pub name: String,
    pub field_type: char,
    pub length: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DbfTable {
    pub fields: Vec<DbfField>,
    pub records: Vec<std::collections::HashMap<String, String>>,
}

pub fn read_dbf<P: AsRef<Path>>(path: P) -> Result<DbfTable, String> {
    let mut file = File::open(path.as_ref()).map_err(|e| format!("無法開啟檔案: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| format!("讀取檔案失敗: {}", e))?;

    if buffer.len() < 32 {
        return Err("DBF 標頭長度不足".to_string());
    }

    let num_records = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;
    let header_len = u16::from_le_bytes([buffer[8], buffer[9]]) as usize;
    let record_len = u16::from_le_bytes([buffer[10], buffer[11]]) as usize;

    let mut fields = Vec::new();
    let mut offset = 32;

    while offset < header_len && offset + 32 <= buffer.len() && buffer[offset] != 0x0D {
        let field_bytes = &buffer[offset..offset + 11];
        let name_end = field_bytes.iter().position(|&b| b == 0).unwrap_or(11);
        let name = String::from_utf8_lossy(&field_bytes[..name_end])
            .trim()
            .to_uppercase();

        let field_type = buffer[offset + 11] as char;
        let length = buffer[offset + 16] as usize;

        fields.push(DbfField {
            name,
            field_type,
            length,
        });

        offset += 32;
    }

    let mut records = Vec::new();
    let mut pos = header_len;

    for _ in 0..num_records {
        if pos + record_len > buffer.len() {
            break;
        }

        let rec_slice = &buffer[pos..pos + record_len];
        pos += record_len;

        let del_flag = rec_slice[0] as char;
        if del_flag == '*' {
            // 已標記刪除記錄，略過
            continue;
        }

        let mut row = std::collections::HashMap::new();
        let mut field_pos = 1;

        for field in &fields {
            if field_pos + field.length <= rec_slice.len() {
                let slice = &rec_slice[field_pos..field_pos + field.length];
                let (decoded, _, _) = BIG5.decode(slice);
                row.insert(field.name.clone(), decoded.trim().to_string());
            }
            field_pos += field.length;
        }

        records.push(row);
    }

    Ok(DbfTable { fields, records })
}

pub fn format_dbf_date(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return "".to_string();
    }
    // 可能是 20260702
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        return format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]);
    }
    // 可能是 2023/3/8 或 2023/03/08
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            let y = parts[0];
            let m = parts[1].parse::<u32>().unwrap_or(1);
            let d = parts[2].parse::<u32>().unwrap_or(1);
            return format!("{:04}-{:02}-{:02}", y, m, d);
        }
    }
    s.to_string()
}
