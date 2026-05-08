use crate::types::{Book, Chapter, ChapterContent};
use reqwest::Client;
use serde_json::Value;
use std::sync::LazyLock;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap()
});

const HOSTS: &[&str] = &[
    "https://v1.gyks.cf",
    "https://v2.gyks.cf",
    "https://v3.gyks.cf",
    "https://v4.gyks.cf",
    "https://v5.gyks.cf",
    "https://v6.gyks.cf",
    "https://v7.gyks.cf",
];

fn map_book(item: &Value) -> Book {
    let title = item["book_name"].as_str().unwrap_or("");
    Book {
        title: clean_book_name(title),
        author: item["author"].as_str().map(|s| s.to_string()),
        cover: item["thumb_url"].as_str().map(|s| s.to_string()),
        intro: item["abstract"].as_str().map(|s| s.to_string()),
        kind: {
            let parts: Vec<&str> = [
                item["status"].as_str(),
                item["score"].as_str(),
                item["tags"].as_str(),
                item["last_chapter_update_time"].as_str(),
            ]
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect();
            if parts.is_empty() { None } else { Some(parts.join(" / ")) }
        },
        last_chapter: {
            let src = item["source"].as_str().unwrap_or("");
            let lct = item["last_chapter_title"].as_str().unwrap_or("");
            if src.is_empty() && lct.is_empty() { None }
            else { Some(format!("{} {}", src, lct).trim().to_string()) }
        },
        word_count: item["word_number"].as_str().map(|s| s.to_string()),
        book_id: item["book_id"].as_str().unwrap_or("").to_string(),
        source_key: "guangyu".into(),
        source: item["source"].as_str().unwrap_or("番茄").to_string(),
        tab: item["tab"].as_str().unwrap_or("小说").to_string(),
    }
}

fn clean_book_name(name: &str) -> String {
    let re = regex_lite::Regex::new(r"[（(]别名[：:].*?[）)]").unwrap();
    re.replace(name, "").trim().to_string()
}

pub async fn search_books(keyword: &str) -> Result<Vec<Book>, String> {
    let kw = keyword.to_owned();
    let mut last_err = String::new();
    for host in HOSTS {
        let h = host.to_string();
        let kw = kw.clone();
        let result = async {
            let resp = CLIENT.get(format!("{}/search", h))
                .query(&[("title", kw.as_str()), ("tab", "小说"), ("source", "番茄"), ("page", "1"), ("disabled_sources", "0")])
                .send().await.map_err(|e| e.to_string())?;
            let json: Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok::<Vec<Book>, String>(json["data"].as_array().map(|a| a.iter().map(map_book).collect()).unwrap_or_default())
        }.await;
        match result {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub async fn get_hot_books() -> Result<Vec<Book>, String> {
    let mut last_err = String::new();
    for host in HOSTS {
        let h = host.to_string();
        let result = async {
            let resp = CLIENT.get(format!("{}/get_discover", h))
                .query(&[("source", "番茄"), ("tab", "小说"), ("bdtype", "热搜榜"), ("gender", "1"), ("is_ranking", "1"), ("page", "1")])
                .send().await.map_err(|e| e.to_string())?;
            let json: Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok::<Vec<Book>, String>(json["data"].as_array().map(|a| a.iter().take(12).map(map_book).collect()).unwrap_or_default())
        }.await;
        match result {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub async fn get_chapters(book_id: &str, source: &str, tab: &str) -> Result<Vec<Chapter>, String> {
    let bid = book_id.to_owned();
    let src = source.to_owned();
    let tb = tab.to_owned();
    let mut last_err = String::new();
    for host in HOSTS {
        let h = host.to_string();
        let bid = bid.clone();
        let src = src.clone();
        let tb = tb.clone();
        let result = async {
            let resp = CLIENT.get(format!("{}/catalog", h))
                .query(&[("book_id", &bid), ("source", &src), ("tab", &tb)])
                .send().await.map_err(|e| e.to_string())?;
            let json: Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok::<Vec<Chapter>, String>(json["data"].as_array().map(|arr| arr.iter().map(|item| Chapter {
                title: item["title"].as_str().unwrap_or("").to_string(),
                item_id: item["item_id"].as_str().unwrap_or("").to_string(),
            }).collect()).unwrap_or_default())
        }.await;
        match result {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub async fn get_chapter_content(
    _book_id: &str, item_id: &str, source: &str, tab: &str,
) -> Result<ChapterContent, String> {
    let iid = item_id.to_owned();
    let src = source.to_owned();
    let tb = tab.to_owned();
    let mut last_err = String::new();
    for host in HOSTS {
        let h = host.to_string();
        let iid = iid.clone();
        let src = src.clone();
        let tb = tb.clone();
        let result = async {
            let body = serde_json::json!({"html":"","item_id":iid,"source":src,"tab":tb,"tone_id":"4","variable":"","version":"4.11.5.1"});
            let resp = CLIENT.post(format!("{}/content", h)).json(&body).send().await.map_err(|e| e.to_string())?;
            let json: Value = resp.json().await.map_err(|e| e.to_string())?;
            let raw = json["content"].as_str().unwrap_or("");
            if raw.contains("免登录访问次数已达上限") {
                return Err("今日免费阅读次数已用完（每日3次），请明天再试".into());
            }
            Ok(ChapterContent { title: json["title"].as_str().unwrap_or("").to_string(), content: clean_content(raw) })
        }.await;
        match result {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

fn clean_content(text: &str) -> String {
    let re = regex_lite::Regex::new(r#"\s*ident="[^"]*""#).unwrap();
    re.replace_all(text, "").split('\n').map(|l| l.trim()).filter(|l| !l.is_empty())
        .map(|l| format!("<p>{}</p>", l)).collect::<Vec<_>>().join("\n")
}
