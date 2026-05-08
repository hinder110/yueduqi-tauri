use crate::types::{Book, Chapter, ChapterContent};
use encoding_rs::GBK;
use scraper::{Html, Selector};
use std::sync::LazyLock;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap()
});

const BASE: &str = "http://m.biquge900.com";

async fn fetch_gbk(url: &str, body: Option<&[u8]>) -> Result<Html, String> {
    let mut req = CLIENT
        .get(url)
        .header("Referer", format!("{}/", BASE));
    if let Some(b) = body {
        req = CLIENT
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Referer", format!("{}/", BASE))
            .body(b.to_vec());
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let (html, _, _) = GBK.decode(&bytes);
    Ok(Html::parse_document(&html.into_owned()))
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
    format!("{}/{}", BASE, path.trim_start_matches("./"))
}

pub async fn search_books(keyword: &str) -> Result<Vec<Book>, String> {
    let (kw_gbk, _, _) = GBK.encode(keyword);
    let body = [
        b"searchkey=".as_ref(),
        &kw_gbk,
        b"&t=1".as_ref(),
    ]
    .concat();

    let html = fetch_gbk(&format!("{}/modules/article/search.php", BASE), Some(&body)).await?;
    let sel_hot = Selector::parse(".hot_sale").map_err(|e| e.to_string())?;
    let sel_a = Selector::parse("a").map_err(|e| e.to_string())?;
    let sel_title = Selector::parse(".title").map_err(|e| e.to_string())?;
    let sel_author = Selector::parse(".author").map_err(|e| e.to_string())?;
    let sel_review = Selector::parse(".review").map_err(|e| e.to_string())?;
    let sel_p = Selector::parse("p").map_err(|e| e.to_string())?;

    let mut books = Vec::new();
    for hot_el in html.select(&sel_hot) {
        let a = match hot_el.select(&sel_a).next() {
            Some(a) => a,
            None => continue,
        };
        let href = a.value().attr("href").unwrap_or("");
        if href.is_empty() {
            continue;
        }

        let name = a
            .select(&sel_title)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .or_else(|| {
                a.select(&sel_p)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
            })
            .unwrap_or_default();

        if name.is_empty() {
            continue;
        }

        books.push(Book {
            title: name,
            author: a
                .select(&sel_author)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string()),
            kind: a
                .select(&sel_review)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string()),
            last_chapter: None,
            book_id: to_abs_url(href),
            source_key: "biquge900".into(),
            source: "biquge900".into(),
            tab: String::new(),
            cover: None,
            intro: None,
            word_count: None,
        });
    }
    Ok(books)
}

pub async fn get_chapters(book_url: &str) -> Result<Vec<Chapter>, String> {
    let html = fetch_gbk(book_url, None).await?;
    let sel = Selector::parse(".directoryArea p a").map_err(|e| e.to_string())?;

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
    let html = fetch_gbk(chapter_url, None).await?;

    let title = html
        .select(&Selector::parse(".title").map_err(|e| e.to_string())?)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    // 用 inner_html 保留 <br> 再转换
    let content_sel = Selector::parse("#chaptercontent").map_err(|e| e.to_string())?;
    let mut raw = String::new();
    if let Some(el) = html.select(&content_sel).next() {
        raw = el.inner_html();
    }

    // 移除 script/style/div/a 标签
    let re_tag = regex_lite::Regex::new(r"<(script|style|div|a)\b[^>]*>.*?</\1>").unwrap();
    let cleaned = re_tag.replace_all(&raw, "");
    // <br> 转 \n
    let re_br = regex_lite::Regex::new(r"<br\s*/?>").unwrap();
    let with_nl = re_br.replace_all(&cleaned, "\n");
    // 去掉所有 HTML 标签
    let re_html = regex_lite::Regex::new(r"<[^>]+>").unwrap();
    let text = re_html.replace_all(&with_nl, "");

    let final_text = text
        .replace("笔趣阁最新域名：", "")
        .replace("，请牢记本域名并相互转告！", "")
        .trim()
        .to_string();

    let mut paragraphs = Vec::new();
    for line in final_text.split('\n') {
        let trimmed = line.trim();
        // 过滤空格缩进
        let trimmed = trimmed.trim_start_matches(|c: char| c == ' ' || c == '\t');
        if !trimmed.is_empty() {
            paragraphs.push(format!("<p>{}</p>", trimmed));
        }
    }

    Ok(ChapterContent {
        title,
        content: paragraphs.join("\n"),
    })
}
