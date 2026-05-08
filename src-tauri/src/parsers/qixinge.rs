use crate::types::{Book, Chapter, ChapterContent};
use scraper::{Html, Selector};
use std::sync::LazyLock;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap()
});

const BASE: &str = "http://www.qixinge.net";

async fn fetch_utf8(url: &str) -> Result<Html, String> {
    let resp = CLIENT
        .get(url)
        .header("Referer", format!("{}/", BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    Ok(Html::parse_document(&text))
}

fn to_abs_url(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    if path.starts_with("http") {
        return path.to_string();
    }
    if path.starts_with('/') {
        return format!("{}{}", BASE, path);
    }
    format!("{}/{}", BASE, path)
}

pub async fn search_books(keyword: &str) -> Result<Vec<Book>, String> {
    let url = format!(
        "{}/search.php?q={}&p=1",
        BASE,
        urlencoding::encode(keyword)
    );
    let html = fetch_utf8(&url).await?;

    let sel_dl = Selector::parse(".col-md-6 dl").map_err(|e| e.to_string())?;
    let sel_img = Selector::parse("dt img").map_err(|e| e.to_string())?;
    let sel_h3a = Selector::parse("h3 a").map_err(|e| e.to_string())?;
    let sel_bo = Selector::parse(".book_other").map_err(|e| e.to_string())?;
    let sel_span = Selector::parse("span").map_err(|e| e.to_string())?;
    let sel_a = Selector::parse("a").map_err(|e| e.to_string())?;

    let mut books = Vec::new();
    for dl in html.select(&sel_dl) {
        let cover = dl
            .select(&sel_img)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(|s| to_abs_url(s));

        let name_link = match dl.select(&sel_h3a).next() {
            Some(a) => a,
            None => continue,
        };
        let href = name_link.value().attr("href").unwrap_or("");
        let name = name_link
            .text()
            .collect::<String>()
            .replace("免费阅读小说", "")
            .trim()
            .to_string();
        // 去掉 [分类] 前缀
        let name = regex::Regex::new(r"^\[.*?\]")
            .unwrap()
            .replace(&name, "")
            .trim()
            .to_string();

        if name.is_empty() || href.is_empty() {
            continue;
        }

        let book_others: Vec<_> = dl.select(&sel_bo).collect();
        let author = book_others
            .first()
            .and_then(|bo| bo.select(&sel_span).next())
            .map(|s| s.text().collect::<String>().trim().to_string());

        let kind = book_others
            .get(1)
            .map(|bo| bo.text().collect::<String>().trim().to_string())
            .map(|s| s.replacen(|c: char| c == '：' || c == ':', "", 1))
            .filter(|s| !s.is_empty());

        let last_chapter = book_others
            .get(3)
            .and_then(|bo| bo.select(&sel_a).next())
            .map(|a| a.text().collect::<String>().trim().to_string());

        books.push(Book {
            title: name,
            author,
            cover,
            kind,
            last_chapter,
            book_id: to_abs_url(href),
            source_key: "qixinge".into(),
            source: "qixinge".into(),
            tab: String::new(),
            intro: None,
            word_count: None,
        });
    }
    Ok(books)
}

pub async fn get_chapters(book_url: &str) -> Result<Vec<Chapter>, String> {
    let html = fetch_utf8(book_url).await?;
    let sel = Selector::parse(".book_list2 li a").map_err(|e| e.to_string())?;

    let chapters: Vec<Chapter> = html
        .select(&sel)
        .filter_map(|a| {
            let href = a.value().attr("href").unwrap_or("");
            let title = a.text().collect::<String>().trim().to_string();
            if href.is_empty() || title.is_empty() {
                None
            } else {
                Some(Chapter {
                    title,
                    item_id: to_abs_url(href),
                })
            }
        })
        .collect();
    Ok(chapters)
}

pub async fn get_chapter_content(chapter_url: &str) -> Result<ChapterContent, String> {
    let html = fetch_utf8(chapter_url).await?;

    let title = html
        .select(&Selector::parse("h1").map_err(|e| e.to_string())?)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    let title = regex::Regex::new(r"-《.*》")
        .unwrap()
        .replace(&title, "")
        .trim()
        .to_string();

    let article_sel = Selector::parse("article.font_max").map_err(|e| e.to_string())?;
    let mut raw = String::new();
    if let Some(el) = html.select(&article_sel).next() {
        raw = el.inner_html();
    }

    let re_tag = regex::Regex::new(r"<(script|style|div|a)\b[^>]*>.*?</(script|style|div|a)>").unwrap();
    let cleaned = re_tag.replace_all(&raw, "");
    let re_br = regex::Regex::new(r"<br\s*/?>").unwrap();
    let with_nl = re_br.replace_all(&cleaned, "\n");
    let re_html = regex::Regex::new(r"<[^>]+>").unwrap();
    let text = re_html.replace_all(&with_nl, "");

    let final_text = regex::Regex::new(r"本章未完.*")
        .unwrap()
        .replace_all(&text, "");
    let final_text = regex::Regex::new(r"第\s*\(?\s*\d+\s*/\s*\d+\s*\)?\s*页")
        .unwrap()
        .replace_all(&final_text, "");
    let final_text = regex::Regex::new(r"\n{3,}")
        .unwrap()
        .replace_all(&final_text, "\n\n")
        .trim()
        .to_string();

    let paragraphs: Vec<String> = final_text
        .split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| format!("<p>{}</p>", l))
        .collect();

    Ok(ChapterContent {
        title,
        content: paragraphs.join("\n"),
    })
}
