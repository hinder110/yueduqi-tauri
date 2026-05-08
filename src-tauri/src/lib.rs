mod parsers;
mod types;

use types::{ApiResponse, Book, Chapter, ChapterContent};

#[tauri::command]
async fn search_books(keyword: String, source: String) -> ApiResponse<Vec<Book>> {
    if keyword.trim().is_empty() {
        return ApiResponse::err("请输入搜索关键词");
    }
    parsers::search_books(&source, keyword.trim()).await
}

#[tauri::command]
async fn get_hot_books() -> ApiResponse<Vec<Book>> {
    match parsers::guangyu::get_hot_books().await {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => ApiResponse::err(e),
    }
}

#[tauri::command]
async fn get_chapters(book_id: String, source: String) -> ApiResponse<Vec<Chapter>> {
    if book_id.is_empty() {
        return ApiResponse::err("缺少 bookId 参数");
    }
    parsers::get_chapters(&source, &book_id, "番茄", "小说").await
}

#[tauri::command]
async fn get_chapter_content(
    book_id: String,
    item_id: String,
    source: String,
) -> ApiResponse<ChapterContent> {
    if book_id.is_empty() || item_id.is_empty() {
        return ApiResponse::err("缺少参数");
    }
    parsers::get_chapter_content(&source, &book_id, &item_id, "番茄", "小说").await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_books,
            get_hot_books,
            get_chapters,
            get_chapter_content,
        ])
        .run(tauri::generate_context!())
        .expect("启动阅读器失败");
}
