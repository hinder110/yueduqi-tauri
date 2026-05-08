use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub title: String,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub intro: Option<String>,
    pub kind: Option<String>,
    pub last_chapter: Option<String>,
    pub word_count: Option<String>,
    pub book_id: String,
    pub source_key: String,
    pub source: String,
    pub tab: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub title: String,
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterContent {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 搜索书籍（占位，后续实现具体解析逻辑）
#[tauri::command]
async fn search_books(_keyword: String, _source: String) -> ApiResponse<Vec<Book>> {
    // TODO: 实现各书源的搜索逻辑
    ApiResponse {
        success: false,
        data: None,
        error: Some(format!("书源 {} 的搜索尚未实现", _source)),
    }
}

/// 获取章节目录（占位）
#[tauri::command]
async fn get_chapters(_book_id: String, _source: String) -> ApiResponse<Vec<Chapter>> {
    ApiResponse {
        success: false,
        data: None,
        error: Some("章节目录获取尚未实现".into()),
    }
}

/// 获取章节正文（占位）
#[tauri::command]
async fn get_chapter_content(
    _book_id: String,
    _item_id: String,
    _source: String,
) -> ApiResponse<ChapterContent> {
    ApiResponse {
        success: false,
        data: None,
        error: Some("正文获取尚未实现".into()),
    }
}

/// 热门推荐（占位）
#[tauri::command]
async fn get_hot_books() -> ApiResponse<Vec<Book>> {
    ApiResponse {
        success: false,
        data: None,
        error: Some("热门推荐尚未实现".into()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_books,
            get_chapters,
            get_chapter_content,
            get_hot_books,
        ])
        .run(tauri::generate_context!())
        .expect("启动阅读器失败");
}
