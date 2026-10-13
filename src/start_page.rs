use crate::storage::BookmarkItem;

pub fn get_start_page_html(dark_mode: bool, bookmarks: &[BookmarkItem]) -> String {
    let bg_color = if dark_mode { "#121212" } else { "#f8f9fa" };
    let text_color = if dark_mode { "#e0e0e0" } else { "#202124" };
    let card_bg = if dark_mode { "#1e1e1e" } else { "#ffffff" };
    let border_color = if dark_mode { "#333333" } else { "#dfe1e5" };
    let input_bg = if dark_mode { "#2a2a2a" } else { "#ffffff" };
    let accent_color = if dark_mode { "#8ab4f8" } else { "#1a73e8" };

    let mut bookmarks_html = String::new();
    if bookmarks.is_empty() {
        bookmarks_html.push_str("<div class='no-bookmarks'>No bookmarks saved yet</div>");
    } else {
        bookmarks_html.push_str("<div class='bookmarks-grid'>");
        for bm in bookmarks.iter().take(8) {
            let title = if bm.title.is_empty() {
                &bm.url
            } else {
                &bm.title
            };
            bookmarks_html.push_str(&format!(
                r#"<a class="bookmark-card" href="{}">
                    <div class="bm-icon">★</div>
                    <div class="bm-title">{}</div>
                </a>"#,
                html_escape(&bm.url),
                html_escape(title)
            ));
        }
        bookmarks_html.push_str("</div>");
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>MidLevel - New Tab</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background-color: {bg_color};
            color: {text_color};
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            display: flex; flex-direction: column; align-items: center; justify-content: center;
            min-height: 100vh; padding: 20px; transition: background-color 0.3s, color 0.3s;
        }}
        .container {{ max-width: 680px; width: 100%; text-align: center; }}
        .logo {{ font-size: 48px; font-weight: 700; letter-spacing: -1px; margin-bottom: 30px; color: {accent_color}; user-select: none; }}
        .logo span {{ color: {text_color}; font-weight: 300; }}
        .search-box {{ width: 100%; margin-bottom: 40px; }}
        .search-form {{ display: flex; background-color: {input_bg}; border: 1px solid {border_color}; border-radius: 24px; padding: 12px 20px; box-shadow: 0 2px 6px rgba(0,0,0,0.05); transition: box-shadow 0.2s, border-color 0.2s; }}
        .search-form:focus-within {{ box-shadow: 0 4px 12px rgba(0,0,0,0.15); border-color: {accent_color}; }}
        .search-input {{ flex: 1; border: none; outline: none; background: transparent; font-size: 16px; color: {text_color}; }}
        .search-button {{ background: none; border: none; color: {accent_color}; font-weight: 600; cursor: pointer; font-size: 15px; padding-left: 10px; }}
        .section-title {{ font-size: 14px; text-transform: uppercase; letter-spacing: 1px; color: {border_color}; margin-bottom: 15px; text-align: left; }}
        .bookmarks-grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 15px; }}
        .bookmark-card {{ background-color: {card_bg}; border: 1px solid {border_color}; border-radius: 12px; padding: 15px 10px; text-decoration: none; color: {text_color}; display: flex; flex-direction: column; align-items: center; transition: transform 0.15s, box-shadow 0.15s; }}
        .bookmark-card:hover {{ transform: translateY(-2px); box-shadow: 0 4px 10px rgba(0,0,0,0.1); }}
        .bm-icon {{ font-size: 20px; margin-bottom: 8px; color: {accent_color}; }}
        .bm-title {{ font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; }}
        .no-bookmarks {{ font-size: 14px; color: {border_color}; padding: 10px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">Mid<span>Level</span></div>
        <div class="search-box">
            <form class="search-form" action="https://www.google.com/search" method="get">
                <input class="search-input" type="text" name="q" placeholder="Search Google or type a URL..." autofocus />
                <button class="search-button" type="submit">Search</button>
            </form>
        </div>
        <div>
            <div class="section-title">Bookmarks</div>
            {bookmarks_html}
        </div>
    </div>
</body>
</html>"#,
        bg_color = bg_color, text_color = text_color, card_bg = card_bg, border_color = border_color, input_bg = input_bg, accent_color = accent_color, bookmarks_html = bookmarks_html
    )
}

fn html_escape(input: &str) -> String {
    input.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}