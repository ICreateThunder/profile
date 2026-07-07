// SPDX-License-Identifier: AGPL-3.0-or-later
use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Known content collections - maps to subdirectories under `src/content/`.
pub const COLLECTIONS: &[&str] = &["newsletters", "projects", "resources", "tricks"];

/// Frontmatter parsed from TOML between `+++` delimiters (Zola/Hugo convention).
#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub slug: String,
    pub published: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_author")]
    pub author: String,
    /// Banner / hero image - 16:9 (recommended 1600×900). Drives the article
    /// hero, the featured-card banner, and the social/OG preview.
    #[serde(default)]
    pub image: Option<String>,
    /// Optional square thumbnail - 1:1 (recommended 400×400) for the compact
    /// list cards. Falls back to a centre-crop of `image`, then a generated cover.
    #[serde(default)]
    pub thumbnail: Option<String>,
}

/// Parse a `published` date into a `(year, month, day)` key for chronological
/// sorting. Accepts both the UK authoring convention `dd-mm-yyyy` and ISO
/// `yyyy-mm-dd`, detected by whichever end carries the four-digit year.
/// Unparseable values sort oldest.
fn date_key(published: &str) -> (u32, u32, u32) {
    let p: Vec<&str> = published.split('-').collect();
    if p.len() == 3 {
        if p[0].len() == 4 {
            if let (Ok(y), Ok(m), Ok(d)) = (p[0].parse(), p[1].parse(), p[2].parse()) {
                return (y, m, d); // yyyy-mm-dd
            }
        } else if p[2].len() == 4
            && let (Ok(d), Ok(m), Ok(y)) = (p[0].parse(), p[1].parse(), p[2].parse())
        {
            return (y, m, d); // dd-mm-yyyy
        }
    }
    (0, 0, 0)
}

fn default_author() -> String {
    "Robert Shalders".to_string()
}

/// A fully parsed article - frontmatter + rendered HTML body.
#[derive(Debug, Clone)]
pub struct Article {
    pub meta: Frontmatter,
    pub collection: String,
    pub html: String,
}

/// All articles, indexed by collection and slug.
/// Loaded once at startup - content is static, no filesystem I/O at request time.
#[derive(Debug, Clone)]
pub struct ContentStore {
    /// collection → [articles sorted by published desc]
    pub by_collection: HashMap<String, Vec<Arc<Article>>>,
    /// (collection, slug) → article. Shares the `Arc`s in `by_collection` -
    /// each article is stored once, not cloned per index.
    pub by_slug: HashMap<(String, String), Arc<Article>>,
    /// All articles, newest-first - precomputed once at load (the command
    /// palette renders it on every page; the feeds consume it too).
    all_sorted: Vec<Arc<Article>>,
}

impl ContentStore {
    /// Load all articles from the `src/content/` directory tree.
    pub fn load(content_dir: &Path) -> Self {
        let mut by_collection: HashMap<String, Vec<Arc<Article>>> = HashMap::new();
        let mut by_slug: HashMap<(String, String), Arc<Article>> = HashMap::new();

        for &collection in COLLECTIONS {
            let dir = content_dir.join(collection);
            if !dir.is_dir() {
                tracing::warn!("content directory not found: {}", dir.display());
                continue;
            }

            let mut articles: Vec<Arc<Article>> = Vec::new();

            let entries: Vec<_> = std::fs::read_dir(&dir)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", dir.display()))
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                .collect();

            for entry in entries {
                let path = entry.path();
                let raw = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

                match parse_article(&raw, collection) {
                    Some(article) => {
                        let article = Arc::new(article);
                        by_slug.insert(
                            (collection.to_string(), article.meta.slug.clone()),
                            Arc::clone(&article),
                        );
                        articles.push(article);
                    }
                    None => {
                        tracing::warn!("failed to parse {}", path.display());
                    }
                }
            }

            // Sort by published date descending (newest first)
            articles.sort_by_key(|a| std::cmp::Reverse(date_key(&a.meta.published)));

            tracing::info!("loaded {} articles from {collection}", articles.len());
            by_collection.insert(collection.to_string(), articles);
        }

        // Newest-first across all collections - precomputed once.
        let mut all_sorted: Vec<Arc<Article>> = by_collection
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect();
        all_sorted.sort_by_key(|a| std::cmp::Reverse(date_key(&a.meta.published)));

        ContentStore {
            by_collection,
            by_slug,
            all_sorted,
        }
    }

    /// All articles across all collections, newest-first. Precomputed at load -
    /// no per-call allocation or re-sort.
    pub fn all_articles(&self) -> &[Arc<Article>] {
        &self.all_sorted
    }

    /// Get articles interleaved round-robin across collections (most recent first per collection),
    /// so the articles index shows a balanced mix rather than one collection dominating.
    pub fn round_robin_articles(&self, per_collection: usize) -> Vec<Arc<Article>> {
        let groups: Vec<Vec<&Arc<Article>>> = COLLECTIONS
            .iter()
            .map(|&c| {
                self.by_collection
                    .get(c)
                    .map(|articles| articles.iter().take(per_collection).collect())
                    .unwrap_or_default()
            })
            .collect();

        let max_len = groups.iter().map(|g| g.len()).max().unwrap_or(0);
        let mut result = Vec::new();
        for i in 0..max_len {
            for group in &groups {
                if let Some(&article) = group.get(i) {
                    result.push(Arc::clone(article));
                }
            }
        }
        result
    }
}

/// Parse a raw markdown string (with TOML frontmatter) into an Article.
fn parse_article(raw: &str, collection: &str) -> Option<Article> {
    let (frontmatter_str, body) = split_frontmatter(raw)?;
    let meta: Frontmatter = toml::from_str(frontmatter_str).ok()?;

    // Render markdown to HTML
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(body, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Some(Article {
        meta,
        collection: collection.to_string(),
        html: sanitise_html(&html_output),
    })
}

/// Sanitise rendered markdown HTML: strip `<script>`/`<iframe>`/`on*=` handlers
/// and `javascript:` URLs, while keeping the markup our markdown features emit.
/// Defence-in-depth *behind* the CSP: content is author-trusted today, but this
/// closes the raw-HTML-passthrough class ahead of any PR-authored
/// content. The default allowlist already permits our tables, figures/captions,
/// links and images; we additionally keep footnote anchor `id`s, any generated
/// `class`es, and task-list checkboxes so no feature regresses.
fn sanitise_html(html: &str) -> String {
    let mut b = ammonia::Builder::default();
    b.add_generic_attributes(["id", "class"]);
    b.add_tags(["input"]);
    b.add_tag_attributes("input", ["type", "checked", "disabled"]);
    b.clean(html).to_string()
}

/// Split raw file content into (frontmatter, body) at the `+++` delimiters.
fn split_frontmatter(raw: &str) -> Option<(&str, &str)> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("+++") {
        return None;
    }
    let after_first = &trimmed[3..];
    let end = after_first.find("\n+++")?;
    let frontmatter = &after_first[..end];
    let body = &after_first[end + 4..];
    Some((frontmatter.trim(), body.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter() {
        let raw = "+++\ntitle = \"Test\"\nslug = \"test\"\n+++\n\n# Hello\n\nWorld";
        let (fm, body) = split_frontmatter(raw).unwrap();
        assert!(fm.contains("title"));
        assert!(body.contains("# Hello"));
    }

    #[test]
    fn sanitiser_strips_active_content_keeps_features() {
        // Active content is removed…
        let dirty = r#"<p>ok</p><script>alert(1)</script><img src=x onerror="alert(1)"><iframe src="//evil"></iframe><a href="javascript:alert(1)">x</a>"#;
        let clean = sanitise_html(dirty);
        assert!(!clean.contains("<script"), "script survived: {clean}");
        assert!(!clean.contains("onerror"), "on*= survived: {clean}");
        assert!(!clean.contains("<iframe"), "iframe survived: {clean}");
        assert!(!clean.contains("javascript:"), "js: URL survived: {clean}");
        // …but our markdown features are preserved.
        let ok = sanitise_html(
            r##"<figure><img src="/x.svg" alt="d"><figcaption>c</figcaption></figure><table><tr><td>1</td></tr></table><sup class="footnote-reference"><a href="#fn1" id="fnref1">1</a></sup>"##,
        );
        assert!(
            ok.contains("<figure>") && ok.contains("<figcaption>"),
            "{ok}"
        );
        assert!(ok.contains("<table>") && ok.contains("<td>"), "{ok}");
        assert!(ok.contains(r#"id="fnref1""#), "footnote id dropped: {ok}");
    }

    #[test]
    fn test_parse_article() {
        let raw = r#"+++
title = "Test Article"
slug = "test"
published = "2026-01-01"
tags = ["rust"]
+++

# Hello World

This is a test."#;
        let article = parse_article(raw, "projects").unwrap();
        assert_eq!(article.meta.title, "Test Article");
        assert_eq!(article.meta.slug, "test");
        assert!(article.html.contains("<h1>Hello World</h1>"));
        assert!(article.html.contains("<p>This is a test.</p>"));
    }
}
