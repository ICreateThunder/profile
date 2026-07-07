// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::AppState;
use crate::content::COLLECTIONS;
use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;

const SITE_URL: &str = "https://robertshalders.com";

/// GET /rss.xml - RSS 2.0 feed of all articles.
pub async fn rss(State(state): State<AppState>) -> impl IntoResponse {
    let articles = state.content.all_articles();

    let mut items = String::new();
    for article in articles {
        let link = format!("{}/{}/{}", SITE_URL, article.collection, article.meta.slug);
        let description = article
            .meta
            .description
            .as_deref()
            .unwrap_or("")
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let title = article
            .meta
            .title
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        items.push_str(&format!(
            "    <item>\n      <title>{title}</title>\n      <link>{link}</link>\n      <description>{description}</description>\n      <pubDate>{}</pubDate>\n      <guid>{link}</guid>\n    </item>\n",
            article.meta.published,
        ));
    }

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Robert Shalders</title>
    <link>{SITE_URL}</link>
    <description>Technical articles on distributed systems, Rust, infrastructure, and more.</description>
    <language>en-gb</language>
    <atom:link href="{SITE_URL}/rss.xml" rel="self" type="application/rss+xml"/>
{items}  </channel>
</rss>"#
    );

    (
        [
            (header::CONTENT_TYPE, "application/rss+xml; charset=utf-8"),
            (
                header::CACHE_CONTROL,
                "public, max-age=3600, s-maxage=86400",
            ),
        ],
        xml,
    )
}

/// GET /sitemap.xml - XML sitemap for search engines.
pub async fn sitemap(State(state): State<AppState>) -> impl IntoResponse {
    let mut urls = String::new();

    // Static pages
    for path in ["/", "/profile", "/articles"] {
        urls.push_str(&format!(
            "  <url><loc>{SITE_URL}{path}</loc><changefreq>weekly</changefreq></url>\n"
        ));
    }

    // Collection index pages
    for collection in COLLECTIONS {
        urls.push_str(&format!(
            "  <url><loc>{SITE_URL}/{collection}</loc><changefreq>weekly</changefreq></url>\n"
        ));
    }

    // Individual articles
    for article in state.content.all_articles() {
        let loc = format!("{}/{}/{}", SITE_URL, article.collection, article.meta.slug);
        urls.push_str(&format!(
            "  <url><loc>{loc}</loc><lastmod>{}</lastmod></url>\n",
            article.meta.published,
        ));
    }

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{urls}</urlset>"#
    );

    (
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (
                header::CACHE_CONTROL,
                "public, max-age=3600, s-maxage=86400",
            ),
        ],
        xml,
    )
}
