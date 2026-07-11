// SPDX-License-Identifier: AGPL-3.0-or-later
use axum::response::IntoResponse;
use base64::Engine;
use maud::{Markup, PreEscaped, html};
use std::sync::Arc;

use super::layout::{self, ArticleMeta, Meta};
use super::nav;
use crate::content::Article;

/// Noise-texture background layer (behind the frame). Only the z-index varies
/// per page, so the class string is defined once here.
const NOISE_LAYER: &str = "absolute animated-background animate-shift -inset-full";

/// The shared page shell used by every page except `home` (which has a bespoke
/// two-column hero): the padded `<main>`, the bordered flex frame carrying the
/// nav rail, and the noise background. `inner` is everything inside the frame
/// after the nav - the page content plus any page-specific background canvas.
fn page_frame(
    active: &str,
    main_class: &str,
    border_extra: &str,
    noise_z: &str,
    inner: Markup,
) -> Markup {
    html! {
        main id="main" tabindex="-1" class=(main_class) {
            div class={ "flex flex-col-reverse lg:flex-row h-full w-full border-4 " (border_extra) } {
                (nav::navigation(active))
                (inner)
            }
            div class={ (NOISE_LAYER) " " (noise_z) } {}
        }
    }
}

/// GET / - landing page with hero, social links, CTA, marquee
pub fn home(render: &layout::Render) -> impl IntoResponse + use<> {
    layout::base(
        Meta {
            title: "Shalders' Site".into(),
            description: "Software engineer focused on distributed systems, embedded Rust, and high-performance computing.".into(),
            path: "/".into(),
            ..Default::default()
        },
        render,
        html! {
            main id="main" tabindex="-1" class="h-svh min-h-142 md:min-h-117 lg:min-h-192 md:p-12 p-8 relative overflow-hidden" {
                div class="relative isolate overflow-hidden flex flex-col-reverse lg:flex-row h-full w-full border-4 justify-between" {
                    // Left column - decorative wordmark + nav (fixed-width so the
                    // long words never widen the column / shove the hero off-centre)
                    div class="flex flex-col animate-fade-in p-8 justify-end lg:h-full lg:w-56 lg:shrink-0 lg:justify-between" {
                        div class="hidden lg:flex flex-col select-none leading-none" aria-hidden="true" {
                            span class="text-lg lg:text-3xl font-thin motion-safe:animate-slide-down hero-text text-neutral-400" { "unreasonably" }
                            span class="text-lg lg:text-3xl font-extralight motion-safe:animate-slide-down hero-text text-neutral-400" { "close" }
                            span class="text-lg lg:text-3xl font-normal motion-safe:animate-slide-down hero-text text-neutral-300" { "to the" }
                            span class="text-lg lg:text-3xl font-bold motion-safe:animate-slide-down hero-text text-neutral-200" { "bare" }
                            span class="text-2xl lg:text-5xl font-black motion-safe:animate-slide-down hero-text glitch-hover glow-cyan text-white" data-text="METAL" { "METAL" }
                        }
                        (nav::navigation_items("home"))
                    }

                    // Right column - hero content + marquee
                    div class="flex flex-col h-full w-full min-w-0 select-none" {
                        // Hero content
                        div class="flex flex-col justify-center items-center text-center flex-1 gap-y-8 md:gap-y-6 p-8 lg:p-0" {
                            p class="text-sm uppercase tracking-widest text-neutral-400" { "Hi, I'm" }
                            h1 class="font-bebas text-6xl md:text-7xl lg:text-8xl uppercase tracking-wide text-black leading-none text-center glitch hero-3d" data-text="Robert Shalders" data-hero-react {
                                "Robert Shalders"
                                span class="animate-blink" { "_" }
                            }
                            p class="text-sm text-neutral-400 tracking-wide" {
                                span data-typewriter data-roles=r#"["Software Engineer","Embedded Rust","Distributed Systems","Pilot in Training"]"# { "Software Engineer" }
                                span class="animate-blink text-neutral-400" { "_" }
                            }
                            div class="flex gap-6 lg:gap-8 pt-4" {
                                a
                                    href="https://github.com/ICreateThunder"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    aria-label="Robert Shalders on GitHub"
                                    class="p-2 transition-transform duration-150 hover:scale-110 hover:text-teal-400"
                                {
                                    // GitHub icon - inline SVG
                                    (github_icon())
                                }
                                a
                                    href="https://www.linkedin.com/in/robertshalders/"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    aria-label="Robert Shalders on LinkedIn"
                                    class="p-2 transition-transform duration-150 hover:scale-110 hover:text-teal-400"
                                {
                                    (linkedin_icon())
                                }
                            }
                        }

                        // Marquee - desktop only
                        div class="hidden lg:flex justify-center items-center shrink-0 w-full" {
                            div class="w-full overflow-hidden border-t border-b border-neutral-800 marquee-fade" aria-label="Featured projects" {
                                div class="marquee-track" {
                                    (marquee_content(false))
                                    (marquee_content(true))
                                }
                            }
                        }
                    }
                    // 3D synthwave grid - drawn by app.js, clipped to this bordered
                    // box. Empty/transparent without JS, so the noise layer shows.
                    canvas data-bg-grid aria-hidden="true" class="absolute inset-0 -z-10 w-full h-full pointer-events-none" {}
                }
                // No-JS fallback: animated noise texture (behind the whole frame).
                div class="absolute animated-background -z-20 animate-shift -inset-full" {}
            }
        },
    )
}

/// GET /profile - minimal identity card: Bebas name (echoing the home hero), a
/// concise overview, a technical `.nfo` info box, and a link tree. Shares the
/// home page's frame/typography; reduced-motion safe.
pub fn profile(render: &layout::Render) -> impl IntoResponse + use<> {
    layout::base(
        Meta {
            title: "Robert Shalders - Profile".into(),
            description: "Robert Shalders - software engineer: high-performance systems, fault-tolerant distributed services, embedded firmware, and quality-of-life apps. Rust by choice, Python as appropriate.".into(),
            path: "/profile".into(),
            ..Default::default()
        },
        render,
        page_frame(
            "profile",
            "h-svh md:p-12 p-8 relative overflow-hidden",
            "relative overflow-hidden",
            "-z-20",
            html! {
                    div class="flex flex-col items-center justify-start lg:justify-center gap-8 w-full h-full overflow-y-auto p-6 sm:p-8" {
                        h1 class="sr-only" { "Robert Shalders - software engineer" }

                        // Identity - same hero treatment as the home page
                        div class="flex flex-col items-center gap-2 select-none text-center" {
                            p class="text-sm uppercase tracking-widest text-neutral-400" { "// profile" }
                            span class="font-bebas text-4xl md:text-5xl lg:text-6xl uppercase tracking-wide text-black leading-none text-center glitch hero-3d" data-text="Robert Shalders" {
                                "Robert Shalders"
                            }
                        }

                        // Overview - genuine, concise
                        p class="max-w-xl text-center text-sm text-neutral-300 leading-relaxed" {
                            "Software engineer building high-performance systems, fault-tolerant distributed services, embedded firmware, and the occasional quality-of-life app. Rust by choice, Python as appropriate. In a stable position - open to good conversations."
                        }

                        // Info box (.nfo-style, technical/tabular)
                        dl class="font-jetbrains text-left text-xs sm:text-sm border border-neutral-700 p-4 grid grid-cols-[5.5rem_1fr] gap-x-3 gap-y-1.5 max-w-md w-full" {
                            (nfo("TRADE", "systems \u{00b7} embedded \u{00b7} distributed"))
                            (nfo("STACK", "Rust \u{00b7} Python \u{00b7} Go \u{00b7} STM32 \u{00b7} K8s"))
                            (nfo("LOCATION", "UK / remote"))
                            (nfo("STATUS", "stable \u{00b7} open to discussions"))
                        }

                        // Link tree
                        nav class="font-jetbrains w-full max-w-md flex flex-col" aria-label="Profile links" {
                            (link_row("GitHub", "https://github.com/ICreateThunder", true))
                            (link_row("LinkedIn", "https://www.linkedin.com/in/robertshalders/", true))
                            (link_row("Email", "mailto:robert@shalders.co.uk?subject=Hello", false))
                            (link_row("Source", "https://github.com/ICreateThunder/profile", true))
                        }

                        // Releases
                        div class="w-full max-w-md" {
                            p class="font-jetbrains text-[0.7rem] uppercase tracking-widest text-neutral-400 mb-2" { "// releases" }
                            div class="flex flex-wrap gap-x-4 gap-y-1 font-jetbrains text-sm" {
                                a class="text-cyan-300 hover:text-white transition-colors" href="https://github.com/ICreateThunder/oxiflight" target="_blank" rel="noopener noreferrer" { "oxiflight" }
                                a class="text-cyan-300 hover:text-white transition-colors" href="https://github.com/ICreateThunder/BEAT-Consensus-Algorithm" target="_blank" rel="noopener noreferrer" { "beat" }
                                a class="text-cyan-300 hover:text-white transition-colors" href="https://github.com/ICreateThunder/ykdf" target="_blank" rel="noopener noreferrer" { "ykdf" }
                                a class="text-cyan-300 hover:text-white transition-colors" href="https://github.com/flight-academy-uk/flight-academy" target="_blank" rel="noopener noreferrer" { "flight-academy" }
                            }
                        }
                    }
                // Invisible mirror of the nav (lg only): an equal-width flex
                // sibling on the right balances the row, so the content column
                // is centred on the frame rather than offset by the left nav.
                div class="invisible hidden lg:flex" aria-hidden="true" { (nav::navigation("profile")) }
                // Warp starfield - opaque teal + indigo streaks accelerating
                // outward from the centre. Drawn by app.js; transparent (so the
                // noise layer shows through) without JS. Reduced-motion safe.
                canvas data-warp aria-hidden="true" class="absolute inset-0 -z-10 w-full h-full pointer-events-none" {}
            },
        ),
    )
}

/// GET /articles - all articles across collections
pub fn articles(
    render: &layout::Render,
    all_articles: &[Arc<Article>],
) -> impl IntoResponse + use<> {
    layout::base(
        Meta {
            title: "Robert Shalders - Articles".into(),
            description:
                "Technical articles on distributed systems, Rust, infrastructure, and more.".into(),
            path: "/articles".into(),
            ..Default::default()
        },
        render,
        articles_view("articles", all_articles),
    )
}

/// GET /:collection - list articles in a specific collection
pub fn collection_page(
    render: &layout::Render,
    collection: &str,
    articles: &[Arc<Article>],
) -> impl IntoResponse + use<> {
    layout::base(
        Meta {
            title: format!("Robert Shalders - {}", capitalise(collection)).into(),
            description: format!("Articles in the {collection} collection.").into(),
            path: format!("/{collection}").into(),
            ..Default::default()
        },
        render,
        articles_view(collection, articles),
    )
}

/// GET /:collection/:slug - individual article page
pub fn article_page(render: &layout::Render, article: &Article) -> impl IntoResponse + use<> {
    let m = &article.meta;
    layout::base(
        Meta {
            title: format!("{} - Robert Shalders", m.title).into(),
            description: m.description.clone().unwrap_or_default().into(),
            path: format!("/{}/{}", article.collection, m.slug).into(),
            image: m.image.clone().map(Into::into),
            article: Some(ArticleMeta {
                published: m.published.clone().into(),
                author: m.author.clone().into(),
                section: article.collection.clone().into(),
                tags: m.tags.iter().cloned().map(Into::into).collect(),
            }),
        },
        render,
        page_frame(
            "articles",
            "h-svh md:p-12 p-8 relative overflow-hidden",
            "",
            "-z-10",
            html! {
                    div class="flex flex-col w-full h-full overflow-y-auto animate-fade-in" {
                        // Hero banner (16:9) - only when the author supplied one.
                        @if let Some(ref banner) = article.meta.image {
                            div class="w-full aspect-video overflow-hidden border-b border-neutral-800" {
                                img class="w-full h-full object-cover object-center" src=(banner) alt="" aria-hidden="true" width="1600" height="900" decoding="async";
                            }
                        }
                        // Article header
                        div class="border-b border-neutral-800 p-6 lg:p-8" {
                            a
                                href={"/" (article.collection)}
                                class="text-xs text-neutral-400 uppercase tracking-widest hover:text-teal-400 transition-colors"
                                hx-get={"/" (article.collection)}
                                hx-target="body"
                                hx-push-url="true"
                                hx-swap="outerHTML"
                                preload="mouseover"
                            {
                                "← " (capitalise(&article.collection))
                            }
                            h1 class="text-3xl lg:text-4xl font-black mt-3" { (article.meta.title) }
                            div class="flex gap-4 mt-3 text-xs text-neutral-400" {
                                span { (format_date(&article.meta.published)) }
                                span { "·" }
                                span { (article.meta.author) }
                            }
                            @if !article.meta.tags.is_empty() {
                                div class="flex flex-wrap gap-2 mt-3" {
                                    @for tag in &article.meta.tags {
                                        span class="text-xs px-2 py-0.5 border border-neutral-800 text-neutral-400" { (tag) }
                                    }
                                }
                            }
                        }
                        // Article body - prose styling via @tailwindcss/typography
                        // Body set in JetBrains Mono for readable long-form text;
                        // headings kept in the display font (Trivial) for hierarchy.
                        div class="p-6 lg:p-8 prose prose-invert prose-sm max-w-none font-jetbrains prose-headings:font-trivial prose-h2:font-black prose-h2:text-2xl prose-h2:text-white prose-h2:mt-10 prose-h2:mb-4 prose-h3:font-bold prose-h3:text-lg prose-h3:uppercase prose-h3:tracking-wide prose-h3:text-teal-400 prose-h3:mt-8 prose-h3:mb-2 prose-h4:font-bold prose-h4:text-base prose-h4:text-neutral-200 prose-a:text-teal-400 prose-code:text-cyan-300 prose-img:w-full prose-img:border prose-img:border-neutral-800 prose-img:bg-black prose-figure:my-6 prose-figcaption:text-center prose-figcaption:text-xs prose-figcaption:text-neutral-400 prose-figcaption:mt-2" {
                            (PreEscaped(&article.html))
                        }
                    }
            },
        ),
    )
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// 404 - not found
pub fn not_found(render: &layout::Render) -> impl IntoResponse + use<> {
    layout::base(
        Meta {
            title: "404 - Not Found".into(),
            description: "The page you're looking for doesn't exist.".into(),
            path: "/error".into(),
            ..Default::default()
        },
        render,
        page_frame(
            "",
            "h-svh min-h-128 md:p-12 p-8 relative overflow-hidden",
            "",
            "-z-10",
            html! {
                    div class="flex flex-col gap-y-8 justify-center items-center w-full p-4 lg:p-0 mb-16 lg:m-9" {
                        div class="flex flex-col justify-center items-center gap-y-4" {
                            span class="font-bebas text-8xl text-teal-400 glitch hero-3d" data-text="404" { "404" }
                            span class="text-2xl text-neutral-400" { "Signal lost" }
                            p class="text-sm text-neutral-400 mt-2" { "The page you're looking for doesn't exist." }
                            a
                                href="/"
                                class="relative inline-block overflow-hidden border-teal-400 border-2 mt-6 px-4 py-2 text-sm font-black cta cursor-pointer"
                                hx-get="/"
                                hx-target="body"
                                hx-push-url="true"
                                hx-swap="outerHTML"
                                preload="mouseover"
                            {
                                "Return Home"
                            }
                        }
                    }
            },
        ),
    )
}

// --- Helper fragments ---

/// Per-collection tag colours. Deliberately chosen from Tailwind families *not*
/// used elsewhere on the site (the accents are teal/cyan/indigo) and spread
/// across the hue wheel, so the tags read as distinct. All are `-400` shades -
/// vivid on the black background and comfortably above 4.5:1 contrast.
fn chip_color(collection: &str) -> &'static str {
    match collection {
        "newsletters" => "text-orange-400 border-orange-400/30",
        "projects" => "text-lime-400 border-lime-400/30",
        "resources" => "text-fuchsia-400 border-fuchsia-400/30",
        "tricks" => "text-rose-400 border-rose-400/30",
        _ => "text-neutral-400 border-neutral-400/30",
    }
}

/// Display a `published` date as UK `dd·mm·yyyy`. Accepts the `dd-mm-yyyy`
/// authoring convention or ISO `yyyy-mm-dd` (detected by the four-digit year).
fn format_date(published: &str) -> String {
    let p: Vec<&str> = published.split('-').collect();
    if p.len() == 3 {
        let (d, m, y) = if p[0].len() == 4 {
            (p[2], p[1], p[0]) // ISO yyyy-mm-dd
        } else {
            (p[0], p[1], p[2]) // dd-mm-yyyy
        };
        format!("{d}·{m}·{y}")
    } else {
        published.to_string()
    }
}

/// Shared `<img>` styling for cards - desaturated at rest, colourised on hover
/// (so varied real screenshots unify into the monochrome look).
const CARD_IMG_CLASS: &str = "w-full h-full object-cover object-center saturate-0 contrast-125 brightness-75 transition-all duration-300 group-hover:saturate-100 group-hover:contrast-100 group-hover:brightness-90";

/// Render a card image - lazy, async-decoded, with explicit dimensions (no
/// layout shift). `src` is the resolved author image (banner or thumbnail); use
/// **AVIF/WebP/JPEG directly** (set e.g. `image = ".../foo.avif"`). When `src`
/// is `None`, a deterministic generated cover (seeded from collection+slug) is
/// used instead. (We deliberately don't synthesise `<picture>` sibling sources -
/// that 404s for any format an author hasn't actually produced.)
///
/// `w`/`h` are the intrinsic dimensions used to reserve space - pass the canonical
/// shape: `1600`×`900` (16:9 banner) for featured cards, `400`×`400` for thumbs.
fn card_image(
    src: Option<&str>,
    alt: &str,
    collection: &str,
    slug: &str,
    w: &str,
    h: &str,
) -> Markup {
    match src {
        Some(p) => html! {
            img class=(CARD_IMG_CLASS) src=(p) alt=(alt)
                width=(w) height=(h) loading="lazy" decoding="async";
        },
        None => {
            // Decorative (the title is shown beside it) → empty alt.
            let cover = generated_cover(collection, slug);
            html! {
                img class=(CARD_IMG_CLASS) src=(cover) alt="" aria-hidden="true"
                    width=(w) height=(h) loading="lazy" decoding="async";
            }
        }
    }
}

/// Tailwind accent hex per collection (matches `chip_color`), for generated art.
fn accent_hex(collection: &str) -> &'static str {
    match collection {
        "newsletters" => "#fb923c", // orange-400
        "projects" => "#a3e635",    // lime-400
        "resources" => "#e879f9",   // fuchsia-400
        "tricks" => "#fb7185",      // rose-400
        _ => "#2dd4bf",             // teal-400
    }
}

/// A deterministic, palette-matched SVG cover seeded from the article identity -
/// a "data field" of cells whose pattern is derived from the SHA-256 of the
/// slug, over a seeded horizon line. No stock imagery; unique per article;
/// desaturated at rest and colourised on hover like any other card image.
/// Returned as a `data:` URI (allowed by the CSP `img-src 'self' data:`).
fn generated_cover(collection: &str, slug: &str) -> String {
    let digest = <sha2::Sha256 as sha2::Digest>::digest(format!("{collection}/{slug}").as_bytes());
    let accent = accent_hex(collection);
    let (cols, rows) = (12u32, 7u32);
    let (cw, ch) = (320.0 / cols as f64, 180.0 / rows as f64);
    let mut cells = String::new();
    let mut idx = 0usize;
    for r in 0..rows {
        for c in 0..cols {
            let bit = (digest[idx % 32] >> (idx % 8)) & 1;
            if bit == 1 {
                let op = 0.12 + ((digest[(idx + 7) % 32] & 0x07) as f64) * 0.05;
                let x = c as f64 * cw + cw * 0.18;
                let y = r as f64 * ch + ch * 0.18;
                cells.push_str(&format!(
                    "<rect x='{x:.1}' y='{y:.1}' width='{:.1}' height='{:.1}' fill='{accent}' opacity='{op:.2}'/>",
                    cw * 0.64,
                    ch * 0.64
                ));
            }
            idx += 1;
        }
    }
    let hy = 60.0 + (digest[0] as f64 / 255.0) * 60.0;
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 320 180' preserveAspectRatio='xMidYMid slice'>\
         <rect width='320' height='180' fill='#0a0a0a'/>{cells}\
         <line x1='0' y1='{hy:.1}' x2='320' y2='{hy:.1}' stroke='{accent}' stroke-width='1.5' opacity='0.55'/></svg>"
    );
    format!(
        "data:image/svg+xml;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(svg)
    )
}

/// Shared listing view for `/articles` and each `/{collection}`. A Bebas header
/// (echoing the home hero), an inline live-filter search box (progressively
/// enhanced - revealed by app.js), responsive category pills, then the cards.
fn articles_view(active: &str, articles: &[Arc<Article>]) -> Markup {
    let featured = &articles[..articles.len().min(4)];
    let remaining = if articles.len() > 4 {
        &articles[4..]
    } else {
        &[]
    };
    let heading = if active == "articles" {
        "Articles".to_string()
    } else {
        capitalise(active)
    };

    page_frame(
        "articles",
        "h-svh min-h-128 md:p-12 p-8 relative overflow-hidden",
        "overflow-hidden",
        "-z-10",
        html! {
                // Cap the reading width and centre it - on ultrawide the column
                // would otherwise stretch the cards across the whole frame.
                div class="flex flex-col min-h-0 w-full max-w-7xl mx-auto" {
                    // Header - Bebas title + inline search
                    div class="shrink-0 border-b border-neutral-800 p-4 lg:p-6 flex flex-col gap-3" {
                        div class="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3" {
                            div class="select-none" {
                                p class="text-xs uppercase tracking-widest text-neutral-400" { "// archive" }
                                h1 class="font-bebas text-4xl md:text-5xl uppercase tracking-wide text-white leading-none" { (heading) }
                            }
                            // Search box - hidden until app.js reveals it (no-JS shows the full list).
                            div data-article-search-wrap hidden class="w-full sm:w-64" {
                                input
                                    data-article-search
                                    type="search"
                                    placeholder="filter\u{2026}"
                                    aria-label="Filter articles"
                                    autocomplete="off"
                                    class="w-full bg-black border border-neutral-700 focus:border-teal-400 text-sm text-neutral-200 placeholder:text-neutral-400 px-3 py-1.5 outline-none transition-colors";
                            }
                        }
                        (category_pills(active))
                    }
                    // List
                    div class="flex-1 min-h-0 overflow-y-auto animate-fade-in" {
                        @if articles.is_empty() {
                            div class="flex flex-col items-center justify-center w-full h-full text-neutral-400 text-center p-8" {
                                h2 class="text-2xl font-bold mb-2 flicker-a" { "SYSTEM ALERT: CONTENT UNAVAILABLE" }
                                p class="text-sm flicker-b" { "No articles in this collection yet." }
                            }
                        } @else {
                            div class="p-4 flex flex-col gap-6" data-article-list {
                                @if !featured.is_empty() {
                                    div class="grid grid-cols-1 md:grid-cols-2 gap-4" {
                                        @for article in featured {
                                            (featured_card(article))
                                        }
                                    }
                                }
                                @if !remaining.is_empty() {
                                    div class="flex flex-col gap-3" {
                                        @for article in remaining {
                                            (remaining_card(article))
                                        }
                                    }
                                }
                            }
                            div data-article-empty hidden class="text-center text-neutral-400 text-sm p-8" {
                                "No articles match your filter."
                            }
                        }
                    }
                }
        },
    )
}

fn category_pills(active: &str) -> Markup {
    let categories = [
        ("/articles", "All", "articles"),
        ("/newsletters", "Newsletters", "newsletters"),
        ("/projects", "Projects", "projects"),
        ("/resources", "Resources", "resources"),
        ("/tricks", "Tips & Tricks", "tricks"),
    ];

    html! {
        // Scrolls horizontally on mobile so every category is reachable.
        div class="flex gap-1 overflow-x-auto -mx-1 px-1 pb-1" {
            @for (href, label, resource) in &categories {
                a
                    href=(href)
                    class={
                        "shrink-0 px-3 py-1.5 text-xs uppercase tracking-widest border transition-colors duration-200 "
                        @if *resource == active {
                            "text-teal-400 border-teal-400"
                        } @else {
                            "text-neutral-400 border-neutral-800 hover:text-neutral-200 hover:border-neutral-600"
                        }
                    }
                    hx-get=(href)
                    hx-target="body"
                    hx-push-url="true"
                    hx-swap="outerHTML"
                    preload="mouseover"
                {
                    (label)
                }
            }
        }
    }
}

/// Lowercased haystack for the client-side article filter (title, collection,
/// tags, description).
fn search_key(a: &Article) -> String {
    let mut s = format!(
        "{} {} {}",
        a.meta.title,
        a.collection,
        a.meta.tags.join(" ")
    );
    if let Some(d) = &a.meta.description {
        s.push(' ');
        s.push_str(d);
    }
    s.to_lowercase()
}

/// Collection chip (prefix before the per-collection accent colour).
const CHIP_CLASS: &str = "text-[0.6rem] uppercase tracking-widest border px-1.5 py-0.5 ";

/// Wrap card content in the shared HTMX-nav anchor (identical across both card
/// shapes): internal link + `data-article`/`data-search` filter hooks.
fn card_link(href: &str, article: &Article, inner: Markup) -> Markup {
    html! {
        a
            href=(href)
            class="group"
            data-article
            data-search=(search_key(article))
            hx-get=(href)
            hx-target="body"
            hx-push-url="true"
            hx-swap="outerHTML"
            preload="mouseover"
        {
            (inner)
        }
    }
}

/// The shared card text block: collection chip + date, title, optional excerpt.
fn card_meta(article: &Article) -> Markup {
    html! {
        div class="flex items-center gap-2 mb-2 flex-wrap overflow-hidden" {
            span class={ (CHIP_CLASS) (chip_color(&article.collection)) } { (article.collection) }
            span class="text-[0.6rem] text-neutral-400" { (format_date(&article.meta.published)) }
        }
        p class="text-sm text-neutral-300 leading-snug transition-colors duration-200 group-hover:text-white" {
            (article.meta.title)
        }
        @if let Some(ref desc) = article.meta.description {
            p class="text-xs text-neutral-400 mt-1 line-clamp-2" { (desc) }
        }
    }
}

fn featured_card(article: &Article) -> Markup {
    let href = format!("/{}/{}", article.collection, article.meta.slug);
    card_link(
        &href,
        article,
        html! {
            article class="reveal flex flex-col border border-neutral-800 overflow-hidden transition-all duration-200 hover:border-teal-400 hover:-translate-y-0.5 hover:shadow-[0_4px_20px_oklch(78.9%_0.154_211.53_/_0.15)]" {
                // 16:9 banner - matches the canonical generated image shape, so
                // a 1600×900 source shows with no surprise crop.
                div class="w-full aspect-video overflow-hidden" {
                    (card_image(article.meta.image.as_deref(), article.meta.title.as_str(), &article.collection, &article.meta.slug, "1600", "900"))
                }
                div class="p-3" { (card_meta(article)) }
            }
        },
    )
}

fn remaining_card(article: &Article) -> Markup {
    let href = format!("/{}/{}", article.collection, article.meta.slug);
    card_link(
        &href,
        article,
        html! {
            article class="reveal flex items-center gap-4 p-4 border border-neutral-800 transition-all duration-200 hover:border-teal-400 hover:shadow-[0_0_12px_oklch(78.9%_0.154_211.53_/_0.2)]" {
                div class="w-14 h-14 md:w-18 md:h-18 overflow-hidden rounded-sm shrink-0" {
                    (card_image(
                        article.meta.thumbnail.as_deref().or(article.meta.image.as_deref()),
                        article.meta.title.as_str(), &article.collection, &article.meta.slug, "400", "400"))
                }
                div class="flex-1 min-w-0" { (card_meta(article)) }
            }
        },
    )
}

// --- Profile variant: .nfo helper ---

/// One `.nfo`-style field row (flows into the parent grid via display:contents).
fn nfo(key: &str, val: &str) -> Markup {
    html! {
        div class="contents" {
            dt class="text-cyan-300" { (key) }
            dd class="text-neutral-300 m-0" { (val) }
        }
    }
}

/// One row in the profile link tree - a full-width terminal-style link.
fn link_row(label: &str, href: &str, external: bool) -> Markup {
    html! {
        a
            href=(href)
            target=[if external { Some("_blank") } else { None }]
            rel=[if external { Some("noopener noreferrer") } else { None }]
            class="group flex items-center justify-between border-b border-neutral-800 py-2.5 text-sm text-neutral-300 hover:text-teal-400 hover:pl-1 transition-all duration-200"
        {
            span { (label) }
            span class="text-neutral-400 group-hover:text-teal-400 transition-colors" aria-hidden="true" { "\u{2192}" }
        }
    }
}

fn marquee_content(duplicate: bool) -> Markup {
    let tabindex = if duplicate { Some("-1") } else { None };
    let aria_hidden = if duplicate { Some("true") } else { None };

    html! {
        div class="marquee-content" aria-hidden=[aria_hidden] {
            (marquee_card("Oxiflight", "Betaflight FC firmware rewritten in embedded Rust for STM32", "https://github.com/ICreateThunder/oxiflight", tabindex))
            (marquee_card("BEAT", "BEAT async BFT consensus protocol in Rust", "https://github.com/ICreateThunder/BEAT-Consensus-Algorithm", tabindex))
            (marquee_card("YKDF", "YubiKey key derivation framework - Rust, post-quantum keys", "https://github.com/ICreateThunder/ykdf", tabindex))
            (marquee_card("Flight Academy", "Open-source platform for UK aviation - Rust + MASH", "https://github.com/flight-academy-uk/flight-academy", tabindex))
            (marquee_card("Cognosco", "Game engine built from scratch in C++", "https://github.com/ICreateThunder/cognosco", tabindex))
            (marquee_card("Profile", "This site - Rust, Maud, HTMX, Tailwind", "https://github.com/ICreateThunder/profile", tabindex))
        }
    }
}

fn marquee_card(name: &str, desc: &str, href: &str, tabindex: Option<&str>) -> Markup {
    html! {
        a href=(href) target="_blank" rel="noopener noreferrer" class="marquee-card" tabindex=[tabindex] {
            span class="text-neutral-400 text-xs uppercase tracking-widest" { (name) }
            span class="text-neutral-400 text-xs" { (desc) }
        }
    }
}

// --- Inline SVG icons (no external icon dependency) ---

fn github_icon() -> Markup {
    html! {
        svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true" {
            path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" {}
        }
    }
}

fn linkedin_icon() -> Markup {
    html! {
        svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true" {
            path d="M0 1.146C0 .513.526 0 1.175 0h13.65C15.474 0 16 .513 16 1.146v13.708c0 .633-.526 1.146-1.175 1.146H1.175C.526 16 0 15.487 0 14.854V1.146zm4.943 12.248V6.169H2.542v7.225h2.401zm-1.2-8.212c.837 0 1.358-.554 1.358-1.248-.015-.709-.52-1.248-1.342-1.248-.822 0-1.359.54-1.359 1.248 0 .694.521 1.248 1.327 1.248h.016zm4.908 8.212V9.359c0-.216.016-.432.08-.586.173-.431.568-.878 1.232-.878.869 0 1.216.662 1.216 1.634v3.865h2.401V9.25c0-2.22-1.184-3.252-2.764-3.252-1.274 0-1.845.7-2.165 1.193v.025h-.016a5.54 5.54 0 01.016-.025V6.169h-2.4c.03.678 0 7.225 0 7.225h2.4z" {}
        }
    }
}
