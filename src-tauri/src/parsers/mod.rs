pub mod guangyu;
pub mod biquge;
pub mod qixinge;

use crate::types::{ApiResponse, Book, Chapter, ChapterContent};

pub async fn search_books(source: &str, keyword: &str) -> ApiResponse<Vec<Book>> {
    let result = match source {
        "guangyu" => guangyu::search_books(keyword).await,
        "biquge900" => {
            let (biquge_books, guangyu_books) = tokio::join!(
                biquge::search_books(keyword),
                async { guangyu::search_books(keyword).await.unwrap_or_default() }
            );
            biquge_books.map(|books| merge_covers(books, &guangyu_books))
        }
        "qixinge" => qixinge::search_books(keyword).await,
        _ => return ApiResponse::err(format!("未知书源: {}", source)),
    };

    match result {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => ApiResponse::err(e),
    }
}

pub async fn get_chapters(source: &str, book_id: &str, inner_source: &str, inner_tab: &str) -> ApiResponse<Vec<Chapter>> {
    let result = match source {
        "guangyu" => guangyu::get_chapters(book_id, inner_source, inner_tab).await,
        "biquge900" => biquge::get_chapters(book_id).await,
        "qixinge" => qixinge::get_chapters(book_id).await,
        _ => return ApiResponse::err(format!("未知书源: {}", source)),
    };

    match result {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => ApiResponse::err(e),
    }
}

pub async fn get_chapter_content(
    source: &str,
    book_id: &str,
    item_id: &str,
    inner_source: &str,
    inner_tab: &str,
) -> ApiResponse<ChapterContent> {
    let result = match source {
        "guangyu" => guangyu::get_chapter_content(book_id, item_id, inner_source, inner_tab).await,
        "biquge900" => biquge::get_chapter_content(item_id).await,
        "qixinge" => qixinge::get_chapter_content(item_id).await,
        _ => return ApiResponse::err(format!("未知书源: {}", source)),
    };

    match result {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => ApiResponse::err(e),
    }
}

/// 把光遇搜索结果的封面/简介按书名模糊匹配到笔趣阁结果中
fn merge_covers(target: Vec<Book>, supplement: &[Book]) -> Vec<Book> {
    target
        .into_iter()
        .map(|mut book| {
            if book.cover.is_some() && book.intro.is_some() {
                return book;
            }
            if let Some(m) = supplement
                .iter()
                .find(|s| fuzzy_match(&book.title, &s.title))
            {
                if book.cover.is_none() {
                    book.cover = m.cover.clone();
                }
                if book.intro.is_none() {
                    book.intro = m.intro.clone();
                }
            }
            book
        })
        .collect()
}

fn fuzzy_match(a: &str, b: &str) -> bool {
    let na = normalize_name(a);
    let nb = normalize_name(b);
    if na.is_empty() || nb.is_empty() {
        return false;
    }
    na == nb || na.contains(&nb) || nb.contains(&na)
}

fn normalize_name(s: &str) -> String {
    let re_paren = regex_lite::Regex::new(r"[（(].*?[）)]").unwrap();
    let re_sym = regex_lite::Regex::new(r"[^一-龥a-zA-Z0-9]").unwrap();
    let no_paren = re_paren.replace_all(s, "").to_string();
    re_sym.replace_all(&no_paren, "").to_lowercase()
}
