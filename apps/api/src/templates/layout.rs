// SPDX-License-Identifier: AGPL-3.0-or-later
use std::borrow::Cow;

use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::content::ContentStore;
use crate::security;

const SITE_NAME: &str = "Robert Shalders";
const SITE_URL: &str = "https://robertshalders.com";
const TWITTER_HANDLE: &str = "@ICreateThunder";
/// Default social-share card for pages without their own image (raster - social
/// platforms do not render SVG `og:image`).
const DEFAULT_OG_IMAGE: &str = "/static/images/og-default.png";

/// Per-article metadata that drives `og:type=article`, the `article:*` tags, and
/// the `BlogPosting` JSON-LD. Absent on non-article pages.
#[derive(Default)]
pub struct ArticleMeta {
    pub published: Cow<'static, str>,
    pub author: Cow<'static, str>,
    pub section: Cow<'static, str>,
    pub tags: Vec<Cow<'static, str>>,
}

/// Page metadata for SEO, Open Graph, and structured data.
///
/// `Cow<'static, str>` lets static pages pass string literals (borrowed, free)
/// and dynamic pages pass owned `String`s - no `Box::leak`. Construct with
/// `..Default::default()` to omit `image`/`article`.
#[derive(Default)]
pub struct Meta {
    pub title: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub path: Cow<'static, str>,
    /// Page image (path under `/static` or absolute). Falls back to the site card.
    pub image: Option<Cow<'static, str>>,
    /// Present on article pages.
    pub article: Option<ArticleMeta>,
}

/// Render context threaded into every page: the inlined CSS, its precomputed
/// CSP hash (computed once at startup, see [`crate::AppState`]), and the content
/// store (so the global command palette can list every page + article).
pub struct Render<'a> {
    pub inline_css: &'a str,
    pub style_hash: &'a str,
    pub content: &'a crate::content::ContentStore,
}

fn abs_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("{SITE_URL}{path}")
    }
}

/// Build the JSON-LD structured-data blob for this page: `BlogPosting` on
/// articles, site-level `WebSite` + `Person` elsewhere. `</` is escaped so the
/// payload can never break out of the `<script>` element.
fn json_ld(meta: &Meta, canonical: &str, og_image: &str) -> String {
    use serde_json::json;
    let value = match &meta.article {
        Some(a) => json!({
            "@context": "https://schema.org",
            "@type": "BlogPosting",
            "headline": meta.title,
            "description": meta.description,
            "datePublished": a.published,
            "author": { "@type": "Person", "name": a.author, "url": SITE_URL },
            "publisher": { "@type": "Person", "name": SITE_NAME, "url": SITE_URL },
            "url": canonical,
            "mainEntityOfPage": canonical,
            "image": og_image,
            "articleSection": a.section,
            "keywords": a.tags.join(", "),
        }),
        None => json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": SITE_NAME,
            "url": SITE_URL,
            "author": { "@type": "Person", "name": SITE_NAME, "url": SITE_URL },
        }),
    };
    value.to_string().replace("</", "<\\/")
}

/// Base HTML layout wrapping all pages. Returns a response carrying a strict,
/// per-page `Content-Security-Policy` whose `script-src`/`style-src` hashes
/// match the inline JSON-LD and stylesheet exactly.
///
/// Critical CSS is inlined in `<style>` to eliminate FOUC on cold cache; fonts
/// and the noise texture are preloaded to avoid late discovery.
pub fn base(meta: Meta, render: &Render, body: Markup) -> impl IntoResponse + use<> {
    let canonical = format!("{}{}", SITE_URL, meta.path);
    let og_image = abs_url(meta.image.as_deref().unwrap_or(DEFAULT_OG_IMAGE));
    let og_type = if meta.article.is_some() {
        "article"
    } else {
        "website"
    };
    let author = meta
        .article
        .as_ref()
        .map(|a| a.author.as_ref())
        .unwrap_or(SITE_NAME)
        .to_string();

    let json_ld = json_ld(&meta, &canonical, &og_image);
    let script_hash = security::sha256_b64(&json_ld);
    let csp = security::content_security_policy(render.style_hash, &script_hash);

    let markup = html! {
        (DOCTYPE)
        // "the static is not decorative. - a humble software engineer"
        html lang="en" class="bg-black text-white font-trivial" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,initial-scale=1";
                meta name="theme-color" content="#000000";

                // Inline critical CSS - no render-blocking request, paints correctly on first frame
                style { (PreEscaped(render.inline_css)) }

                // Preload critical fonts - discovered immediately instead of after CSS parse
                link rel="preload" href="/static/fonts/Trivial-Regular.otf" as="font" type="font/otf" crossorigin="anonymous";
                link rel="preload" href="/static/fonts/Trivial-Bold.otf" as="font" type="font/otf" crossorigin="anonymous";
                link rel="preload" href="/static/fonts/Trivial-Heavy.otf" as="font" type="font/otf" crossorigin="anonymous";
                link rel="preload" href="/static/fonts/BebasNeue-Regular.ttf" as="font" type="font/ttf" crossorigin="anonymous";
                link rel="preload" href="/static/images/noise.png" as="image";

                meta name="description" content=(meta.description);
                meta name="author" content=(author);
                link rel="canonical" href=(canonical);
                link rel="alternate" type="application/rss+xml" title=(SITE_NAME) href="/rss.xml";
                link rel="sitemap" href="/sitemap.xml";
                meta name="robots" content="index,follow";

                // Open Graph
                meta property="og:title" content=(meta.title);
                meta property="og:description" content=(meta.description);
                meta property="og:type" content=(og_type);
                meta property="og:url" content=(canonical);
                meta property="og:site_name" content=(SITE_NAME);
                meta property="og:locale" content="en_GB";
                meta property="og:image" content=(og_image);
                meta property="og:image:width" content="1200";
                meta property="og:image:height" content="630";
                meta property="og:image:alt" content=(meta.title);
                @if let Some(a) = &meta.article {
                    meta property="article:published_time" content=(a.published);
                    meta property="article:author" content=(a.author);
                    meta property="article:section" content=(a.section);
                    @for tag in &a.tags {
                        meta property="article:tag" content=(tag);
                    }
                }

                // Twitter
                meta name="twitter:card" content="summary_large_image";
                meta name="twitter:creator" content=(TWITTER_HANDLE);
                meta name="twitter:title" content=(meta.title);
                meta name="twitter:description" content=(meta.description);
                meta name="twitter:image" content=(og_image);

                // JSON-LD structured data
                script type="application/ld+json" { (PreEscaped(&json_ld)) }

                link rel="icon" href="/static/favicon.svg";

                // HTMX - deferred, not render-blocking
                script src="/static/vendor/htmx.min.js" defer {}
                script src="/static/vendor/htmx-preload.js" defer {}
                // Retro/CRT progressive enhancement - view transitions, boot,
                // typewriter, cursor-reactive hero, hover-glitch, scroll-reveal
                script src="/static/vendor/app.js" defer {}

                title { (meta.title) }
            }
            body hx-ext="preload" {
                // Skip link - first focusable element, jumps to <main>.
                a class="skip-link sr-only focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:z-[10001] focus:bg-black focus:text-teal-400 focus:border focus:border-teal-400 focus:px-3 focus:py-2 focus:no-underline" href="#main" { "Skip to content" }

                // First-load boot loader (abstract, no text). JS toggles
                // `.booting` on <html> once per session: a single neobrutalist
                // meter fills, then the overlay clears to the page.
                div class="boot" aria-hidden="true" {
                    div class="boot-bar" {}
                }
                (body)
                // Persistent CRT scanline + vignette overlay (above content).
                div class="crt-overlay" aria-hidden="true" {}
                // Command palette (⌘/Ctrl-K) - global; driven by app.js.
                (command_palette(render.content))
                // Screen-reader announcement of route changes on HTMX nav.
                div id="route-announcer" class="sr-only" aria-live="polite" aria-atomic="true" {}
            }
        }
    };

    let csp = HeaderValue::from_str(&csp).expect("CSP is ASCII");
    ([(header::CONTENT_SECURITY_POLICY, csp)], markup)
}

/// The ⌘/Ctrl-K command palette: a global, accessible quick-jump to every page,
/// collection, article, external link, and a couple of actions. Rendered into
/// every page (hidden until opened by `app.js`). Plain links/buttons → works as
/// a no-JS site map too; no inline scripts/styles, so CSP is unaffected.
fn command_palette(content: &ContentStore) -> Markup {
    html! {
        div class="cmdk-overlay" data-cmdk hidden {
            div class="cmdk" role="dialog" aria-modal="true" aria-label="Command menu" {
                input
                    class="cmdk-input"
                    type="text"
                    autocomplete="off"
                    autocapitalize="off"
                    spellcheck="false"
                    placeholder="Jump to\u{2026}  (pages · articles · actions)"
                    aria-label="Search commands"
                    aria-controls="cmdk-list";
                ul class="cmdk-list" id="cmdk-list" role="listbox" aria-label="Commands" {
                    li class="cmdk-group" aria-hidden="true" { "Pages" }
                    (cmdk_link("Home", "/", "home start landing"))
                    (cmdk_link("Profile", "/profile", "profile about bio whoami"))
                    (cmdk_link("Articles", "/articles", "articles writing posts blog"))

                    li class="cmdk-group" aria-hidden="true" { "Collections" }
                    (cmdk_link("Newsletters", "/newsletters", "newsletters"))
                    (cmdk_link("Projects", "/projects", "projects"))
                    (cmdk_link("Resources", "/resources", "resources"))
                    (cmdk_link("Tips & Tricks", "/tricks", "tricks tips"))

                    li class="cmdk-group" aria-hidden="true" { "Articles" }
                    @for a in content.all_articles() {
                        (cmdk_link(
                            &a.meta.title,
                            &format!("/{}/{}", a.collection, a.meta.slug),
                            &format!("{} {}", a.collection, a.meta.tags.join(" ")),
                        ))
                    }

                    li class="cmdk-group" aria-hidden="true" { "Links" }
                    (cmdk_ext("GitHub", "https://github.com/ICreateThunder", "github code"))
                    (cmdk_ext("LinkedIn", "https://www.linkedin.com/in/robertshalders/", "linkedin"))
                    (cmdk_ext("Email", "mailto:robert@shalders.co.uk", "email contact mail"))
                    (cmdk_ext("RSS feed", "/rss.xml", "rss feed subscribe"))
                    (cmdk_ext("Source code", "https://github.com/ICreateThunder/profile", "source repo agpl github"))

                    li class="cmdk-group" aria-hidden="true" { "Actions" }
                    (cmdk_action("Toggle visual effects", "toggle-fx", "fx effects crt glitch motion reduce", ""))
                    (cmdk_action("Copy email address", "copy-email", "copy email clipboard", "robert@shalders.co.uk"))

                    li class="cmdk-empty" hidden { "No matches" }
                }
            }
        }
    }
}

/// Internal palette entry - navigates via HTMX (with a plain `href` fallback).
fn cmdk_link(label: &str, href: &str, keys: &str) -> Markup {
    let key = format!("{label} {keys}").to_lowercase();
    html! {
        li class="cmdk-item" role="option" data-key=(key) {
            a class="cmdk-link" href=(href)
                hx-get=(href) hx-target="body" hx-push-url="true" hx-swap="outerHTML" preload="mouseover" {
                (label)
            }
        }
    }
}

/// External / feed palette entry - opens in a new tab when absolute.
fn cmdk_ext(label: &str, href: &str, keys: &str) -> Markup {
    let key = format!("{label} {keys}").to_lowercase();
    let external = href.starts_with("http");
    html! {
        li class="cmdk-item" role="option" data-key=(key) {
            a class="cmdk-link" href=(href)
                target=[if external { Some("_blank") } else { None }]
                rel=[if external { Some("noopener noreferrer") } else { None }] {
                (label)
                @if external { span class="cmdk-ext" aria-hidden="true" { " \u{2197}" } }
            }
        }
    }
}

/// Action palette entry - a button handled by `app.js` via `data-action`.
fn cmdk_action(label: &str, action: &str, keys: &str, value: &str) -> Markup {
    let key = format!("{label} {keys}").to_lowercase();
    html! {
        li class="cmdk-item" role="option" data-key=(key) {
            button class="cmdk-link" type="button" data-action=(action) data-value=(value) {
                (label)
            }
        }
    }
}
