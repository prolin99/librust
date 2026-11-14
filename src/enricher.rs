use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde_json::Value;
use std::fs;
use crate::models::{RateLimitConfig, EnrichmentProgress};
use crate::db::DbPool;

#[derive(Debug, Clone)]
pub struct FetchedBookInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub pub_year: Option<String>,
    pub description: Option<String>,
    pub cover_image_bytes: Option<Vec<u8>>,
    pub cover_url: Option<String>,
    pub source: String,
    pub is_chinese: bool,
}

fn clean_html_text(raw: &str) -> Option<String> {
    let mut s = raw.to_string();
    while let Some(start) = s.find('<') {
        if let Some(end) = s[start..].find('>') {
            s.replace_range(start..=start + end, "");
        } else {
            break;
        }
    }
    let s = s.replace("&amp;", "&")
             .replace("&lt;", "<")
             .replace("&gt;", ">")
             .replace("&quot;", "\"")
             .replace("&#39;", "'")
             .replace("&apos;", "'")
             .replace("&nbsp;", " ")
             .replace("&#xA;", " ")
             .replace("&#x9;", " ")
             .replace("&#10;", " ")
             .replace("&#9;", " ");

    let trimmed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn sanitize_book_title(title: &str) -> String {
    let t = title.trim();
    if t.starts_with("【電子書】") {
        t.trim_start_matches("【電子書】").trim().to_string()
    } else if t.starts_with("[電子書]") {
        t.trim_start_matches("[電子書]").trim().to_string()
    } else {
        t.to_string()
    }
}

pub struct EnricherManager {
    pub progress: Arc<RwLock<EnrichmentProgress>>,
    pub cancel_flag: Arc<Mutex<bool>>,
    pub client: reqwest::Client,
    pub covers_dir: PathBuf,
}

impl EnricherManager {
    pub fn contains_chinese_text(value: &str) -> bool {
        value.chars().any(|ch| {
            ('\u{4E00}'..='\u{9FFF}').contains(&ch)
                || ('\u{3400}'..='\u{4DBF}').contains(&ch)
                || ('\u{F900}'..='\u{FAFF}').contains(&ch)
        })
    }

    /// 判斷文字是否符合漢語拼音 / 威妥瑪拼音的羅馬音譯特徵
    pub fn looks_like_chinese_pinyin(value: &str) -> bool {
        let clean = value.trim();
        if clean.is_empty() || Self::contains_chinese_text(clean) {
            return false;
        }

        // 常見漢語拼音與羅馬化音節特徵庫
        const PINYIN_SYLLABLES: &[&str] = &[
            "man", "hua", "san", "guo", "zhi", "hong", "lou", "meng", "xi", "you", "ji",
            "zhong", "wen", "de", "shi", "xue", "da", "xiao", "tai", "bei", "chu", "ban",
            "shu", "bian", "qian", "zi", "er", "tong", "shao", "nian", "li", "she", "hui",
            "ren", "min", "jia", "ting", "ke", "ji", "yi", "shu", "yin", "yue", "dian",
            "ying", "shi", "jie", "di", "li", "tian", "kong", "hai", "yang", "zheng", "zhi",
            "jing", "ji", "guan", "li", "fa", "lu", "zhe", "xue", "zong", "jiao", "wen",
            "zhang", "xiao", "shuo", "san", "wen", "shi", "ge", "xi", "qu", "zhuan", "ji",
            "ci", "dian", "shou", "ce", "tu", "shu", "guan", "bao", "gao", "lun", "wen",
            "ping", "lun", "yan", "jiu", "fa", "zhan", "chuang", "zuo", "she", "ji", "zhi",
            "zao", "gong", "cheng", "nong", "ye", "gong", "ye", "shang", "ye", "jin", "rong",
            "mao", "yi", "cai", "wu", "kuai", "ji", "shen", "ti", "jian", "kang", "yi",
            "liao", "wei", "sheng", "ti", "yu", "yun", "dong", "you", "xi", "sheng", "huo",
            "yin", "shi", "lv", "you", "feng", "su", "chuan", "tong", "xian", "dai", "wei",
            "lai", "gu", "dai", "jin", "dai", "dang", "dai", "zhong", "guo", "tai", "wan",
            "xiang", "gang", "ao", "men", "bei", "jing", "shang", "hai", "guang", "zhou",
            "ba", "bai", "ban", "bang", "bao", "bei", "ben", "beng", "bi", "bian", "biao",
            "bie", "bin", "bing", "bo", "bu", "ca", "cai", "can", "cang", "cao", "ce", "cen",
            "ceng", "cha", "chai", "chan", "chang", "chao", "che", "chen", "cheng", "chi",
            "chong", "chou", "chu", "chuai", "chuan", "chuang", "chui", "chun", "chuo", "ci",
            "cong", "cou", "cu", "cuan", "cui", "cun", "cuo", "da", "dai", "dan", "dang",
            "dao", "de", "deng", "di", "dian", "diao", "die", "ding", "diu", "dong", "dou",
            "du", "duan", "dui", "dun", "duo", "e", "en", "er", "fa", "fan", "fang", "fei",
            "fen", "feng", "fo", "fou", "fu", "ga", "gai", "gan", "gang", "gao", "ge", "gei",
            "gen", "geng", "gong", "gou", "gu", "gua", "guai", "guan", "guang", "gui", "gun",
            "guo", "ha", "hai", "han", "hang", "hao", "he", "hei", "hen", "heng", "hong",
            "hou", "hu", "hua", "huai", "huan", "huang", "hui", "hun", "huo", "ji", "jia",
            "jian", "jiang", "jiao", "jie", "jin", "jing", "jiong", "jiu", "ju", "juan",
            "jue", "jun", "ka", "kai", "kan", "kang", "kao", "ke", "ken", "keng", "kong",
            "kou", "ku", "kua", "kuai", "kuan", "kuang", "kui", "kun", "kuo", "la", "lai",
            "lan", "lang", "lao", "le", "lei", "leng", "li", "lia", "lian", "liang", "liao",
            "lie", "lin", "ling", "liu", "long", "lou", "lu", "luan", "lue", "lun", "luo",
            "ma", "mai", "man", "mang", "mao", "me", "mei", "men", "meng", "mi", "mian",
            "miao", "mie", "min", "ming", "miu", "mo", "mou", "mu", "na", "nai", "nan",
            "nang", "nao", "ne", "nei", "nen", "neng", "ni", "nian", "niang", "niao", "nie",
            "nin", "ning", "niu", "nong", "nou", "nu", "nuan", "nue", "nuo", "o", "ou",
            "pa", "pai", "pan", "pang", "pao", "pei", "pen", "peng", "pi", "pian", "piao",
            "pie", "pin", "ping", "po", "pou", "pu", "qi", "qia", "qian", "qiang", "qiao",
            "qie", "qin", "qing", "qiong", "qiu", "qu", "quan", "que", "qun", "ran", "rang",
            "rao", "re", "ren", "reng", "ri", "rong", "rou", "ru", "ruan", "rui", "run",
            "ruo", "sa", "sai", "san", "sang", "sao", "se", "sen", "seng", "sha", "shai",
            "shan", "shang", "shao", "she", "shen", "sheng", "shi", "shou", "shu", "shua",
            "shuai", "shuan", "shuang", "shui", "shun", "shuo", "si", "song", "sou", "su",
            "suan", "sui", "sun", "suo", "ta", "tai", "tan", "tang", "tao", "te", "teng",
            "ti", "tian", "tiao", "tie", "ting", "tong", "tou", "tu", "tuan", "tui", "tun",
            "tuo", "wa", "wai", "wan", "wang", "wei", "wen", "weng", "wo", "wu", "xi", "xia",
            "xian", "xiang", "xiao", "xie", "xin", "xing", "xiong", "xiu", "xu", "xuan",
            "xue", "xun", "ya", "yan", "yang", "yao", "ye", "yi", "yin", "ying", "yo", "yong",
            "you", "yu", "yuan", "yue", "yun", "za", "zai", "zan", "zang", "zao", "ze", "zei",
            "zen", "zeng", "zha", "zhai", "zhan", "zhang", "zhao", "zhe", "zhen", "zheng",
            "zhi", "zhong", "zhou", "zhu", "zhua", "zhuai", "zhuan", "zhuang", "zhui", "zhun",
            "zhuo", "zi", "zong", "zou", "zu", "zuan", "zui", "zun", "zuo"
        ];

        let tokens: Vec<String> = clean
            .to_ascii_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if tokens.is_empty() {
            return false;
        }

        let match_count = tokens
            .iter()
            .filter(|t| PINYIN_SYLLABLES.contains(&t.as_str()))
            .count();

        // 若大於 50% 的分詞落在拼音音節庫中，判定為音譯拼音
        (match_count as f64) / (tokens.len() as f64) >= 0.5
    }

    /// 判斷 Open Library 或其他來源的記錄是否為中文/漢語書籍
    pub fn is_chinese_book_record(info: &Value) -> bool {
        // 1. 檢查 languages 欄位
        if let Some(langs) = info.get("languages").and_then(|v| v.as_array()) {
            for l in langs {
                let key = l.get("key").and_then(|k| k.as_str()).unwrap_or("");
                let name = l.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if key.contains("chi") || key.contains("zho") || name.eq_ignore_ascii_case("chinese") {
                    return true;
                }
            }
        }
        if let Some(details) = info.get("details") {
            if let Some(langs) = details.get("languages").and_then(|v| v.as_array()) {
                for l in langs {
                    let key = l.get("key").and_then(|k| k.as_str()).unwrap_or("");
                    if key.contains("chi") || key.contains("zho") {
                        return true;
                    }
                }
            }
        }

        // 2. 檢查 publish_places / publish_country 出版地
        let place_match = |val: &str| {
            let s = val.to_ascii_lowercase();
            s.contains("taiwan") || s.contains("taibei") || s.contains("taipei")
                || s.contains("china") || s.contains("beijing") || s.contains("hong kong")
                || s.contains("zhonghe") || s.contains("kaohsiung")
        };

        if let Some(places) = info.get("publish_places").and_then(|v| v.as_array()) {
            for p in places {
                if let Some(name) = p.get("name").and_then(|n| n.as_str()) {
                    if place_match(name) {
                        return true;
                    }
                }
            }
        }
        if let Some(details) = info.get("details") {
            if let Some(places) = details.get("publish_places").and_then(|v| v.as_array()) {
                for p in places {
                    if let Some(name) = p.as_str().or_else(|| p.get("name").and_then(|n| n.as_str())) {
                        if place_match(name) {
                            return true;
                        }
                    }
                }
            }
            if let Some(country) = details.get("publish_country").and_then(|c| c.as_str()) {
                if country.trim().eq_ignore_ascii_case("ch") {
                    return true;
                }
            }
        }

        false
    }

    /// 常用簡體漢字精準轉為台灣正體繁體中文
    pub fn to_traditional_chinese(simplified: &str) -> String {
        let mut out = String::with_capacity(simplified.len());
        for ch in simplified.chars() {
            let t = match ch {
                '画' => '畫', '国' => '國', '记' => '記', '经' => '經', '传' => '傳',
                '书' => '書', '语' => '語', '话' => '話', '门' => '門', '义' => '義',
                '会' => '會', '学' => '學', '历' => '歷', '史' => '史', '梦' => '夢',
                '楼' => '樓', '红' => '紅', '游' => '遊', '录' => '錄', '选' => '選',
                '总' => '總', '编' => '編', '译' => '譯', '著' => '著', '释' => '釋',
                '评' => '評', '点' => '點', '汉' => '漢', '简' => '簡', '繁' => '繁',
                '体' => '體', '馆' => '館', '图' => '圖', '连' => '連', '环' => '環',
                '报' => '報', '说' => '說', '论' => '論', '资' => '資', '库' => '庫',
                '术' => '術', '医' => '醫', '药' => '藥', '师' => '師', '发' => '發',
                '展' => '展', '导' => '導', '读' => '讀', '儿' => '兒', '乐' => '樂',
                '园' => '園', '动' => '動', '网' => '網', '络' => '絡', '线' => '線',
                '现' => '現', '实' => '實', '验' => '驗', '证' => '證', '关' => '關',
                '标' => '標', '签' => '籤', '类' => '類', '别' => '別', '级' => '級',
                '数' => '數', '价' => '價', '钱' => '錢', '币' => '幣', '买' => '買',
                '卖' => '賣', '风' => '風', '险' => '險', '战' => '戰', '争' => '爭',
                '胜' => '勝', '问' => '問', '题' => '題', '难' => '難', '开' => '開',
                '户' => '戶', '机' => '機', '电' => '電', '脑' => '腦', '习' => '習',
                '惯' => '慣', '规' => '規', '则' => '則', '制' => '制', '权' => '權',
                '责' => '責', '爱' => '愛', '气' => '氣', '质' => '質', '变' => '變',
                '头' => '頭', '灵' => '靈', '圣' => '聖', '宝' => '寶', '贝' => '貝',
                '财' => '財', '产' => '產', '业' => '業', '务' => '務', '营' => '營',
                '专' => '專', '长' => '長', '间' => '間', '时' => '時', '岁' => '歲',
                '纪' => '紀', '保' => '保', '护' => '護', '卫' => '衛',
                '院' => '院', '众' => '眾', '夥' => '夥', '谊' => '誼', '亲' => '親',
                '校' => '校', '教' => '教', '育' => '育', '课' => '課', '页' => '頁',
                '软' => '軟', '统' => '統', '装' => '裝', '备' => '備', '处' => '處',
                '运' => '運', '输' => '輸', '车' => '車', '辆' => '輛', '飞' => '飛',
                '海' => '海', '陆' => '陸', '阶' => '階', '段' => '段', '铁' => '鐵',
                _ => ch,
            };
            out.push(t);
        }
        out
    }

    /// 漢語羅馬拼音音譯轉繁體中文（如 "Man hua San guo zhi" -> "漫畫三國志"）
    pub async fn pinyin_to_traditional_chinese(&self, pinyin: &str) -> Result<String, String> {
        let clean = pinyin.trim().replace('-', " ").to_ascii_lowercase();
        if clean.is_empty() {
            return Ok("".to_string());
        }

        let url = "https://www.google.com/inputtools/request";

        let resp = self.client.get(url)
            .query(&[
                ("text", clean.as_str()),
                ("ime", "pinyin"),
                ("num", "1"),
            ])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .send()
            .await
            .map_err(|e| format!("連線拼音還原服務失敗: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("拼音還原服務回傳狀態碼: {}", resp.status()));
        }

        let json = resp.json::<Value>().await
            .map_err(|e| format!("解析拼音回傳資料失敗: {}", e))?;

        // Google Input Tools 格式：["SUCCESS", [["man hua san guo zhi", ["漫画三国志"], ...]]]
        let candidate = json.get(1)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get(1))
            .and_then(|candidates| candidates.as_array())
            .and_then(|list| list.first())
            .and_then(|s| s.as_str());

        if let Some(simplified) = candidate {
            let traditional = Self::to_traditional_chinese(simplified);
            Ok(traditional)
        } else {
            Err("未能辨識該拼音詞組".to_string())
        }
    }

    fn chinese_priority_score(info: &Value) -> u8 {
        let mut score = 0u8;

        if let Some(lang) = info.get("language").and_then(|v| v.as_str()) {
            let normalized = lang.trim().to_ascii_lowercase();
            if normalized.starts_with("zh") {
                score += 3;
            }
        }

        if let Some(title) = info.get("title").and_then(|v| v.as_str()) {
            if Self::contains_chinese_text(title) {
                score += 2;
            }
        }

        if let Some(author) = info.get("authors").and_then(|v| v.as_array()) {
            if author.iter().any(|v| v.as_str().is_some_and(Self::contains_chinese_text)) {
                score += 1;
            }
        }

        if let Some(author) = info.get("author").and_then(|v| v.as_str()) {
            if Self::contains_chinese_text(author) {
                score += 1;
            }
        }

        score
    }

    pub fn new(covers_dir: PathBuf) -> Self {
        fs::create_dir_all(&covers_dir).ok();
        let client = reqwest::Client::builder()
            .user_agent("ModernLibrarySystem/1.0 (Windows NT 10.0; Win64; x64)")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            progress: Arc::new(RwLock::new(EnrichmentProgress {
                is_running: false,
                total: 0,
                processed: 0,
                success: 0,
                not_found: 0,
                failed: 0,
                current_book: None,
                last_message: "待命狀態".to_string(),
            })),
            cancel_flag: Arc::new(Mutex::new(false)),
            client,
            covers_dir,
        }
    }

    /// 從金石堂網路書店查詢書目資訊 (針對台灣繁體中文書目優化，涵蓋 978-626, 978-957, 978-986 等)
    async fn fetch_from_kingstone(&self, clean_isbn: &str) -> Option<FetchedBookInfo> {
        let url = format!("https://www.kingstone.com.tw/search/key/{}", clean_isbn);
        let resp = self.client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7")
            .send().await.ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let html = resp.text().await.ok()?;
        if !html.contains("displayunit") && !html.contains("beta_display") {
            return None;
        }

        // 書名提取：pdnamebox 中的 <a>，或 coverbox alt
        let raw_title = if let Some(p1) = html.find("class=\"pdnamebox\"") {
            let sub = &html[p1..];
            let a_start = sub.find("<a")?;
            let text_start = sub[a_start..].find('>')? + a_start + 1;
            let text_end = sub[text_start..].find("</a>")? + text_start;
            clean_html_text(&sub[text_start..text_end])
        } else if let Some(c1) = html.find("class=\"coverbox\"") {
            let sub = &html[c1..];
            let alt_idx = sub.find("alt=\"")?;
            let alt_start = alt_idx + 5;
            let alt_end = sub[alt_start..].find('"')? + alt_start;
            clean_html_text(&sub[alt_start..alt_end])
        } else {
            None
        }?;

        let title = sanitize_book_title(&raw_title);

        // 作者提取：class="author"
        let author = if let Some(a_idx) = html.find("class=\"author\"") {
            let sub = &html[a_idx..];
            let end_idx = sub.find("</span>").unwrap_or_else(|| sub.len().min(300));
            let author_snippet = &sub[..end_idx];
            if let Some(b_start) = author_snippet.find("<b>") {
                let b_end = author_snippet[b_start..].find("</b>").unwrap_or_else(|| author_snippet.len() - b_start) + b_start;
                clean_html_text(&author_snippet[b_start + 3..b_end])
            } else {
                clean_html_text(author_snippet)
            }
        } else {
            None
        };

        // 出版社提取：class="publish"
        let publisher = if let Some(p_idx) = html.find("class=\"publish\"") {
            let sub = &html[p_idx..];
            let end_idx = sub.find("</span>").unwrap_or_else(|| sub.len().min(300));
            let pub_snippet = &sub[..end_idx];
            if let Some(b_start) = pub_snippet.find("<b>") {
                let b_end = pub_snippet[b_start..].find("</b>").unwrap_or_else(|| pub_snippet.len() - b_start) + b_start;
                clean_html_text(&pub_snippet[b_start + 3..b_end])
            } else {
                clean_html_text(pub_snippet)
            }
        } else {
            None
        };

        // 簡介提取：class="excerptbox"
        let description = if let Some(e_idx) = html.find("class=\"excerptbox\"") {
            let sub = &html[e_idx..];
            if let Some(tag_end) = sub.find('>') {
                let text_start = tag_end + 1;
                let text_end = sub[text_start..].find("</p>").unwrap_or_else(|| sub.len() - text_start) + text_start;
                clean_html_text(&sub[text_start..text_end])
            } else {
                None
            }
        } else {
            None
        };

        // 封面圖片提取：class="coverbox" 中的 <img src="..."
        let mut cover_url = if let Some(c_idx) = html.find("class=\"coverbox\"") {
            let sub = &html[c_idx..];
            if let Some(src_idx) = sub.find("src=\"") {
                let src_start = src_idx + 5;
                let src_end = sub[src_start..].find('"').unwrap_or(0) + src_start;
                let raw_url = sub[src_start..src_end].trim().to_string();
                if !raw_url.is_empty() && !raw_url.contains("noimg") {
                    Some(raw_url.replace("m.jpg", "b.jpg"))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 出版年提取：可嘗試取得 /basic/{code}/
        let mut pub_year = None;
        if let Some(code_idx) = html.find("href=\"/basic/") {
            let sub = &html[code_idx + 13..];
            let code_end = sub.find('/').unwrap_or(0);
            let code = &sub[..code_end];
            if !code.is_empty() && code.chars().all(|c| c.is_ascii_digit()) {
                let detail_url = format!("https://www.kingstone.com.tw/basic/{}/", code);
                if let Ok(d_resp) = self.client.get(&detail_url)
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                    .send().await
                {
                    if let Ok(d_html) = d_resp.text().await {
                        if let Some(d_idx) = d_html.find("出版日") {
                            let d_sub = &d_html[d_idx..d_html.len().min(d_idx + 200)];
                            for part in d_sub.split(|c: char| !c.is_ascii_digit()) {
                                if part.len() == 4 && (part.starts_with("19") || part.starts_with("20")) {
                                    pub_year = Some(part.to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let cover_image_bytes = if let Some(ref u) = cover_url {
            if let Ok(resp) = self.client.get(u).send().await {
                if resp.status().is_success() {
                    resp.bytes().await.ok().map(|b| b.to_vec())
                } else {
                    let fallback_u = u.replace("b.jpg", "m.jpg");
                    if let Ok(resp2) = self.client.get(&fallback_u).send().await {
                        if resp2.status().is_success() {
                            cover_url = Some(fallback_u);
                            resp2.bytes().await.ok().map(|b| b.to_vec())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let is_chinese = Self::contains_chinese_text(&title);

        Some(FetchedBookInfo {
            title: Some(title),
            author,
            publisher,
            pub_year,
            description,
            cover_image_bytes,
            cover_url,
            source: "金石堂 (Kingstone)".to_string(),
            is_chinese,
        })
    }

    /// 從三民網路書店查詢書目資訊 (互補學術、政府與本土出版品)
    async fn fetch_from_sanmin(&self, clean_isbn: &str) -> Option<FetchedBookInfo> {
        let url = format!("https://www.sanmin.com.tw/search/index/?ct=K&qu={}", clean_isbn);
        let resp = self.client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7")
            .send().await.ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let html = resp.text().await.ok()?;
        if !html.contains("class=\"Title") {
            return None;
        }

        let title = if let Some(h3_start) = html.find("<h3>") {
            let text_start = h3_start + 4;
            let text_end = html[text_start..].find("</h3>")? + text_start;
            clean_html_text(&html[text_start..text_end])
        } else {
            None
        }?;

        let title = sanitize_book_title(&title);

        let author = if let Some(a_idx) = html.find("class=\"Author\"") {
            let sub = &html[a_idx..];
            let end_idx = sub.find("</span>").unwrap_or_else(|| sub.len().min(300));
            let author_snippet = &sub[..end_idx];
            if let Some(link_start) = author_snippet.find("<a") {
                let t_start = author_snippet[link_start..].find('>')? + link_start + 1;
                let t_end = author_snippet[t_start..].find("</a>")? + t_start;
                clean_html_text(&author_snippet[t_start..t_end])
            } else {
                clean_html_text(author_snippet)
            }
        } else {
            None
        };

        let publisher = if let Some(p_idx) = html.find("class=\"Publisher\"") {
            let sub = &html[p_idx..];
            let end_idx = sub.find("</span>").unwrap_or_else(|| sub.len().min(300));
            let pub_snippet = &sub[..end_idx];
            if let Some(link_start) = pub_snippet.find("<a") {
                let t_start = pub_snippet[link_start..].find('>')? + link_start + 1;
                let t_end = pub_snippet[t_start..].find("</a>")? + t_start;
                clean_html_text(&pub_snippet[t_start..t_end])
            } else {
                clean_html_text(pub_snippet)
            }
        } else {
            None
        };

        let pub_year = if let Some(d_idx) = html.find("PublishDate") {
            let sub = &html[d_idx..html.len().min(d_idx + 200)];
            let mut year = None;
            for part in sub.split(|c: char| !c.is_ascii_digit()) {
                if part.len() == 4 && (part.starts_with("19") || part.starts_with("20")) {
                    year = Some(part.to_string());
                    break;
                }
            }
            year
        } else {
            None
        };

        let description = if let Some(d_idx) = html.find("class=\"Description") {
            let sub = &html[d_idx..];
            let tag_end = sub.find('>')?;
            let text_start = tag_end + 1;
            let text_end = sub[text_start..].find("</div>").unwrap_or_else(|| sub.len() - text_start) + text_start;
            clean_html_text(&sub[text_start..text_end])
        } else {
            None
        };

        let cover_url = if let Some(img_idx) = html.find("data-src=\"https://cdnec.sanmin.com.tw/product_images/") {
            let sub = &html[img_idx + 10..];
            let end = sub.find('"').unwrap_or(0);
            let u = sub[..end].trim().to_string();
            if !u.is_empty() { Some(u) } else { None }
        } else {
            None
        };

        let cover_image_bytes = if let Some(ref u) = cover_url {
            if let Ok(resp) = self.client.get(u).send().await {
                if resp.status().is_success() {
                    resp.bytes().await.ok().map(|b| b.to_vec())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let is_chinese = Self::contains_chinese_text(&title);

        Some(FetchedBookInfo {
            title: Some(title),
            author,
            publisher,
            pub_year,
            description,
            cover_image_bytes,
            cover_url,
            source: "三民書局 (Sanmin)".to_string(),
            is_chinese,
        })
    }

    /// 根據 ISBN 以中文資料來源為優先查詢書目資訊
    /// 順序：1) 金石堂 Kingstone (繁體中文台灣書目首選)  2) 三民書局 Sanmin (老牌書局互補)
    ///       3) WorldCat (OCLC)  4) Google Books (zh-TW)  5) Open Library
    pub async fn fetch_book_by_isbn(&self, raw_isbn: &str) -> Result<Option<FetchedBookInfo>, String> {
        let clean_isbn = raw_isbn.trim().replace('-', "").replace(' ', "");
        if clean_isbn.is_empty() {
            return Ok(None);
        }

        // ── 輔助閉包：下載封面圖片位元組 ──────────────────────────────
        let download_cover = |url: String| {
            let client = self.client.clone();
            async move {
                if let Ok(resp) = client.get(&url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            return Some(bytes.to_vec());
                        }
                    }
                }
                None
            }
        };

        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        // 1. 金石堂網路書店（台灣繁體中文資料第一首選，涵蓋 978-626, 978-957, 978-986 等最新與各類圖書）
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        if let Some(info) = self.fetch_from_kingstone(&clean_isbn).await {
            return Ok(Some(info));
        }

        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        // 2. 三民書局（台灣三大老牌書局，互補學術、政府、本土繁體書）
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        if let Some(info) = self.fetch_from_sanmin(&clean_isbn).await {
            return Ok(Some(info));
        }

        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        // 2. WorldCat ISBN Search（OCLC 開放 API，常有中文資料）
        //    URL: https://www.worldcat.org/isbn/{isbn} → 解析 JSON-LD
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        let worldcat_url = format!(
            "https://www.worldcat.org/api/search?q=bn%3A{}&heldBy=&limit=1",
            clean_isbn
        );
        if let Ok(resp) = self.client.get(&worldcat_url)
            .header("Accept", "application/json")
            .send().await
        {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    let record = json.get("briefRecords").and_then(|b| b.as_array())
                        .and_then(|a| a.first());
                    if let Some(rec) = record {
                        let title = rec.get("title").and_then(|v| v.as_str())
                            .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                        let author = rec.get("creator").and_then(|v| v.as_str())
                            .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                        let publisher = rec.get("publisher").and_then(|v| v.as_str())
                            .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                        let pub_year = rec.get("date").and_then(|v| v.as_str())
                            .map(|s| s.chars().filter(|c| c.is_ascii_digit()).take(4).collect::<String>())
                            .filter(|s| !s.is_empty());

                        if title.is_some() {
                            // WorldCat 封面圖片
                            let cover_id = rec.get("oclcNumber").and_then(|v| v.as_str())
                                .or_else(|| rec.get("oclcNum").and_then(|v| v.as_str()));
                            let cover_url = cover_id.map(|oclc| {
                                format!("https://covers.openlibrary.org/b/oclc/{}-M.jpg", oclc)
                            }).or_else(|| {
                                Some(format!("https://covers.openlibrary.org/b/isbn/{}-M.jpg", clean_isbn))
                            });
                            let cover_image_bytes = match &cover_url {
                                Some(u) => download_cover(u.clone()).await,
                                None => None,
                            };
                            let is_chinese = title.as_deref().map(Self::contains_chinese_text).unwrap_or(false);
                            return Ok(Some(FetchedBookInfo {
                                title,
                                author,
                                publisher,
                                pub_year,
                                description: None,
                                cover_image_bytes,
                                cover_url,
                                source: "WorldCat (OCLC)".to_string(),
                                is_chinese,
                            }));
                        }
                    }
                }
            }
        }

        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        // 3. Google Books API（限制語言為中文，優先 zh-TW）
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        let gbooks_url = format!(
            "https://www.googleapis.com/books/v1/volumes?q=isbn:{}&langRestrict=zh",
            clean_isbn
        );
        // 若中文限制查無結果，再不帶語言限制重查一次
        for url in [gbooks_url,
                    format!("https://www.googleapis.com/books/v1/volumes?q=isbn:{}", clean_isbn)]
        {
            if let Ok(resp) = self.client.get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<Value>().await {
                        if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
                            let mut best_info: Option<&Value> = None;
                            let mut best_score = u8::MIN;

                            for item in items {
                                let Some(info) = item.get("volumeInfo") else {
                                    continue;
                                };
                                let score = Self::chinese_priority_score(info);
                                if score > best_score {
                                    best_score = score;
                                    best_info = Some(info);
                                }
                            }

                            if let Some(info) = best_info.or_else(|| items.first().and_then(|first| first.get("volumeInfo"))) {
                                let title = info.get("title")
                                    .and_then(|v| v.as_str()).map(|s| s.to_string());
                                let authors = info.get("authors")
                                    .and_then(|v| v.as_array()).map(|arr| {
                                        arr.iter().filter_map(|a| a.as_str())
                                            .collect::<Vec<_>>().join("、")
                                    });
                                let publisher = info.get("publisher")
                                    .and_then(|v| v.as_str()).map(|s| s.to_string());
                                let pub_year = info.get("publishedDate")
                                    .and_then(|v| v.as_str()).map(|s| {
                                        s.chars().take(4).collect::<String>()
                                    });
                                let description = info.get("description")
                                    .and_then(|v| v.as_str()).map(|s| s.to_string());

                                let cover_url = info.get("imageLinks")
                                    .and_then(|img| {
                                        img.get("large")
                                            .or_else(|| img.get("medium"))
                                            .or_else(|| img.get("thumbnail"))
                                            .or_else(|| img.get("smallThumbnail"))
                                    })
                                    .and_then(|u| u.as_str())
                                    .map(|s| s.replace("http://", "https://"));

                                let cover_image_bytes = match &cover_url {
                                    Some(u) => download_cover(u.clone()).await,
                                    None => None,
                                };

                                let is_chinese = title.as_deref().map(Self::contains_chinese_text).unwrap_or(false)
                                    || description.as_deref().map(Self::contains_chinese_text).unwrap_or(false);
                                let source_label = if is_chinese {
                                    "Google Books (中文)".to_string()
                                } else {
                                    "Google Books (英文/外文)".to_string()
                                };

                                return Ok(Some(FetchedBookInfo {
                                    title,
                                    author: authors,
                                    publisher,
                                    pub_year,
                                    description,
                                    cover_image_bytes,
                                    cover_url,
                                    source: source_label,
                                    is_chinese,
                                }));
                            }
                        }
                    }
                }
            }
        }

        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        // 4. Open Library（具備中文漢語羅馬拼音音譯智慧辨識與還原）
        // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        let ol_url = format!(
            "https://openlibrary.org/api/books?bibkeys=ISBN:{}&format=json&jscmd=details",
            clean_isbn
        );
        if let Ok(resp) = self.client.get(&ol_url).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    let key = format!("ISBN:{}", clean_isbn);
                    if let Some(book_entry) = json.get(&key) {
                        let details = book_entry.get("details");
                        let raw_title = details.and_then(|d| d.get("title")).or_else(|| book_entry.get("title"))
                            .and_then(|v| v.as_str()).map(|s| s.trim().to_string()).unwrap_or_default();

                        let authors = details.and_then(|d| d.get("authors")).or_else(|| book_entry.get("authors"))
                            .and_then(|v| v.as_array()).map(|arr| {
                                arr.iter().filter_map(|a| a.get("name").and_then(|n| n.as_str())).collect::<Vec<_>>().join("、")
                            });

                        let publisher = details.and_then(|d| d.get("publishers")).or_else(|| book_entry.get("publishers"))
                            .and_then(|v| v.as_array()).and_then(|arr| {
                                arr.first().and_then(|p| {
                                    p.as_str().or_else(|| p.get("name").and_then(|n| n.as_str()))
                                }).map(|s| s.to_string())
                            });

                        let pub_year = details.and_then(|d| d.get("publish_date")).or_else(|| book_entry.get("publish_date"))
                            .and_then(|v| v.as_str()).map(|s| {
                                s.chars().filter(|c| c.is_ascii_digit()).take(4).collect::<String>()
                            });

                        let cover_url = book_entry.get("cover").or_else(|| details.and_then(|d| d.get("cover")))
                            .and_then(|c| {
                                c.get("large").or_else(|| c.get("medium")).or_else(|| c.get("small"))
                            })
                            .and_then(|u| u.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| Some(format!("https://covers.openlibrary.org/b/isbn/{}-M.jpg", clean_isbn)));

                        let cover_image_bytes = match &cover_url {
                            Some(u) => download_cover(u.clone()).await,
                            None => None,
                        };

                        // 判斷此書是否為中文/漢語書籍，且書名是音譯的英文
                        let is_chinese_book = Self::is_chinese_book_record(book_entry)
                            || Self::looks_like_chinese_pinyin(&raw_title);

                        let mut final_title = raw_title.clone();
                        let mut final_author = authors;
                        let mut final_publisher = publisher;
                        let mut source = "Open Library".to_string();
                        let mut is_chinese = Self::contains_chinese_text(&raw_title);

                        // 關鍵需求：如果是中文書(漢語)且書名是音譯的英文，才轉成中文
                        if is_chinese_book && Self::looks_like_chinese_pinyin(&raw_title) && !is_chinese {
                            if let Ok(zh_title) = self.pinyin_to_traditional_chinese(&raw_title).await {
                                if !zh_title.trim().is_empty() {
                                    final_title = zh_title;
                                    source = "Open Library (漢語音譯轉正體)".to_string();
                                    is_chinese = true;
                                }
                            }
                        }

                        // 同步檢查並將作者與出版社更換為中文
                        if is_chinese_book || is_chinese {
                            if let Some(ref a) = final_author {
                                if !Self::contains_chinese_text(a) {
                                    if let Ok(zh_a) = self.translate_to_traditional_chinese(a).await {
                                        if !zh_a.trim().is_empty() && zh_a != *a {
                                            final_author = Some(zh_a);
                                        }
                                    }
                                }
                            }
                            if let Some(ref p) = final_publisher {
                                if !Self::contains_chinese_text(p) {
                                    if let Ok(zh_p) = self.translate_to_traditional_chinese(p).await {
                                        if !zh_p.trim().is_empty() && zh_p != *p {
                                            final_publisher = Some(zh_p);
                                        }
                                    }
                                }
                            }
                        }

                        if !final_title.is_empty() {
                            return Ok(Some(FetchedBookInfo {
                                title: Some(final_title),
                                author: final_author,
                                publisher: final_publisher,
                                pub_year,
                                description: None,
                                cover_image_bytes,
                                cover_url,
                                source,
                                is_chinese,
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// 啟動背景批次補齊任務（具備精確限速器與斷點續傳）
    pub async fn start_batch_enrichment(
        self: Arc<Self>,
        db: DbPool,
        config: RateLimitConfig,
    ) {
        {
            let mut prog = self.progress.write().await;
            if prog.is_running {
                return; // 已在執行中
            }
            prog.is_running = true;
            prog.last_message = "正在準備待補齊書籍清單...".to_string();
        }

        {
            let mut cancel = self.cancel_flag.lock().await;
            *cancel = false;
        }

        let covers_dir = self.covers_dir.clone();

        tokio::spawn(async move {
            // 查詢需要補齊的書籍 (完全在獨立 scope 內完成並釋放 conn 鎖，避免跨 await 持有 MutexGuard)
            let query_res = {
                let conn = db.lock().unwrap();
                let stmt = conn.prepare(
                    "SELECT id, title, isbn, description, cover_url FROM books
                     WHERE (isbn != '' AND (cover_url = '' OR description = '' OR enrichment_status = 'none' OR enrichment_status = 'pending'))
                     ORDER BY id ASC"
                );
                match stmt {
                    Ok(mut s) => {
                        let rows = s.query_map([], |r| {
                            Ok((
                                r.get(0)?,
                                r.get(1)?,
                                r.get(2)?,
                                r.get(3)?,
                                r.get(4)?,
                            ))
                        }).and_then(|it| it.collect::<Result<Vec<(String, String, String, String, String)>, _>>());
                        rows.map_err(|e| e.to_string())
                    }
                    Err(e) => Err(e.to_string()),
                }
            };

            let books_to_process = match query_res {
                Ok(b) => b,
                Err(e) => {
                    let mut p = self.progress.write().await;
                    p.is_running = false;
                    p.last_message = format!("查詢書籍出錯: {}", e);
                    return;
                }
            };

            let total = books_to_process.len();
            {
                let mut p = self.progress.write().await;
                p.total = total;
                p.processed = 0;
                p.success = 0;
                p.not_found = 0;
                p.failed = 0;
                p.last_message = format!("共有 {} 本書籍等待補齊，開始依速率執行...", total);
            }

            if total == 0 {
                let mut p = self.progress.write().await;
                p.is_running = false;
                p.last_message = "所有書籍資料已完整，無需補齊！".to_string();
                return;
            }

            // 限速參數計算：每分鐘 N 次 -> 基礎間隔 = 60000 / N 毫秒
            let rpm = if config.requests_per_minute == 0 { 20 } else { config.requests_per_minute.min(60) };
            let base_delay_ms = 60_000 / (rpm as u64);
            let jitter_ms = config.jitter_ms.min(2000);

            for (id, title, isbn, desc, cover) in books_to_process {
                // 檢查是否要求取消
                {
                    let cancel = self.cancel_flag.lock().await;
                    if *cancel {
                        let mut p = self.progress.write().await;
                        p.is_running = false;
                        p.last_message = "使用者已暫停/取消補齊任務".to_string();
                        break;
                    }
                }

                {
                    let mut p = self.progress.write().await;
                    p.current_book = Some(format!("《{}》 (條碼:{}, ISBN:{})", title, id, isbn));
                    p.last_message = format!("正在查詢: 《{}》", title);
                }

                // 執行查詢
                match self.fetch_book_by_isbn(&isbn).await {
                    Ok(Some(info)) => {
                        // 儲存封面圖片至本機目錄
                        let mut local_cover_path = "".to_string();
                        if let Some(bytes) = info.cover_image_bytes {
                            let filename = format!("{}.jpg", id.replace('/', "_").replace('\\', "_"));
                            let save_path = covers_dir.join(&filename);
                            if fs::write(&save_path, &bytes).is_ok() {
                                local_cover_path = format!("/covers/{}", filename);
                            }
                        }

                        // 更新資料庫
                        {
                            let conn = db.lock().unwrap();
                            let new_desc = if config.overwrite_existing || desc.trim().is_empty() {
                                info.description.unwrap_or(desc)
                            } else {
                                desc
                            };

                            let new_cover = if !local_cover_path.is_empty() {
                                local_cover_path
                            } else {
                                cover
                            };

                            let _ = conn.execute(
                                "UPDATE books SET description = ?1, cover_url = ?2, enrichment_status = 'completed' WHERE id = ?3",
                                rusqlite::params![new_desc, new_cover, id],
                            );
                        }

                        let mut p = self.progress.write().await;
                        p.processed += 1;
                        p.success += 1;
                        p.last_message = format!("成功補齊：《{}》", title);
                    }
                    Ok(None) => {
                        // 網路查無此書
                        {
                            let conn = db.lock().unwrap();
                            let _ = conn.execute(
                                "UPDATE books SET enrichment_status = 'not_found' WHERE id = ?1",
                                rusqlite::params![id],
                            );
                        }
                        let mut p = self.progress.write().await;
                        p.processed += 1;
                        p.not_found += 1;
                        p.last_message = format!("查無資料：《{}》 (ISBN: {})", title, isbn);
                    }
                    Err(err_msg) => {
                        let mut p = self.progress.write().await;
                        p.processed += 1;
                        p.failed += 1;
                        p.last_message = format!("查詢失敗：《{}》- {}", title, err_msg);
                    }
                }

                // 限速延遲：基礎延遲 + 隨機抖動 (避免被判斷為惡意攻擊)
                let delay = base_delay_ms + (fastrand::u64(0..jitter_ms.max(1)));
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }

            let mut p = self.progress.write().await;
            p.is_running = false;
            p.current_book = None;
            p.last_message = format!(
                "補齊完成！共處理 {} 本（成功 {} 本，查無 {} 本，失敗 {} 本）",
                p.processed, p.success, p.not_found, p.failed
            );
        });
    }

    pub async fn stop_batch_enrichment(&self) {
        let mut cancel = self.cancel_flag.lock().await;
        *cancel = true;
    }

    /// 呼叫開放翻譯介面，將英文/外文書目資訊一鍵翻譯為繁體中文
    pub async fn translate_to_traditional_chinese(&self, text: &str) -> Result<String, String> {
        let clean = text.trim();
        if clean.is_empty() {
            return Ok("".to_string());
        }

        // 1. 先嘗試 MyMemory 開放翻譯 API（針對人名、出版社、常用字詞準確率高且不被封鎖）
        let mymemory_url = "https://api.mymemory.translated.net/get";
        if let Ok(resp) = self.client.get(mymemory_url)
            .query(&[("q", clean), ("langpair", "en|zh-TW")])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(t_text) = json.get("responseData").and_then(|d| d.get("translatedText")).and_then(|v| v.as_str()) {
                        let clean_t = t_text.trim();
                        // 確保不是回傳空白或原本英文，且包含中文字元
                        if !clean_t.is_empty() && Self::contains_chinese_text(clean_t) {
                            return Ok(Self::to_traditional_chinese(clean_t));
                        }
                    }
                }
            }
        }

        // 2. Google Translate 作為備用服務
        let url = "https://translate.googleapis.com/translate_a/single";
        let resp = self.client.get(url)
            .query(&[
                ("client", "gtx"),
                ("sl", "auto"),
                ("tl", "zh-TW"),
                ("dt", "t"),
                ("q", clean),
            ])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .send()
            .await;

        if let Ok(resp) = resp {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    let mut translated = String::new();
                    if let Some(sentences) = json.get(0).and_then(|v| v.as_array()) {
                        for s in sentences {
                            if let Some(part) = s.get(0).and_then(|v| v.as_str()) {
                                translated.push_str(part);
                            }
                        }
                    }
                    if !translated.trim().is_empty() {
                        return Ok(Self::to_traditional_chinese(&translated));
                    }
                }
            }
        }

        Ok(clean.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::EnricherManager;

    #[test]
    fn detects_chinese_text() {
        assert!(EnricherManager::contains_chinese_text("哈利波特"));
        assert!(!EnricherManager::contains_chinese_text("Harry Potter"));
    }

    #[test]
    fn detects_chinese_pinyin() {
        assert!(EnricherManager::looks_like_chinese_pinyin("Man hua San guo zhi"));
        assert!(EnricherManager::looks_like_chinese_pinyin("Hong lou meng"));
        assert!(!EnricherManager::looks_like_chinese_pinyin("Atomic Habits"));
        assert!(!EnricherManager::looks_like_chinese_pinyin("The Great Gatsby"));
        assert!(!EnricherManager::looks_like_chinese_pinyin("漫畫三國志"));
    }

    #[tokio::test]
    async fn converts_pinyin_to_chinese_async() {
        let manager = EnricherManager::new(std::path::PathBuf::from("covers"));
        let res1 = manager.pinyin_to_traditional_chinese("Man hua San guo zhi").await;
        assert_eq!(res1.unwrap(), "漫畫三國志");

        let res2 = manager.pinyin_to_traditional_chinese("Hong lou meng").await;
        assert_eq!(res2.unwrap(), "紅樓夢");

        let res3 = manager.pinyin_to_traditional_chinese("Xi you ji").await;
        assert_eq!(res3.unwrap(), "西遊記");
    }

    #[tokio::test]
    async fn fetches_taiwan_isbn_626() {
        let manager = EnricherManager::new(std::path::PathBuf::from("covers"));
        let book = manager.fetch_book_by_isbn("9786263182035").await.unwrap();
        assert!(book.is_some(), "應成功取得 9786263182035 書目");
        let b = book.unwrap();
        assert!(b.title.as_deref().unwrap().contains("遊戲人生72變"));
        assert!(b.author.as_deref().unwrap().contains("莊越翔"));
        assert!(b.publisher.as_deref().unwrap().contains("商周"));
        assert!(b.is_chinese);
    }
}
