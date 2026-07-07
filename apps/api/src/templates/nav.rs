// SPDX-License-Identifier: AGPL-3.0-or-later
use maud::{Markup, html};

struct NavItem {
    url: &'static str,
    name: &'static str,
    resource: &'static str,
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        url: "/",
        name: "Home",
        resource: "home",
    },
    NavItem {
        url: "/profile",
        name: "Profile",
        resource: "profile",
    },
    NavItem {
        url: "/articles",
        name: "Articles",
        resource: "articles",
    },
];

/// Render just the `<nav>` element - no wrapper div.
/// Used on the homepage where the parent column already provides padding and layout.
pub fn navigation_items(current: &str) -> Markup {
    html! {
        nav class="flex md:flex-col justify-evenly shrink-0" {
            @for item in NAV_ITEMS {
                a
                    href=(item.url)
                    aria-current=[if item.resource == current { Some("page") } else { None }]
                    class="flex flex-col justify-center items-center md:hidden flex-1 py-2 no-underline text-neutral-400 hover:text-white transition-all duration-300 aria-[current=page]:text-teal-400"
                    hx-get=(item.url)
                    hx-target="body"
                    hx-push-url="true"
                    hx-swap="outerHTML"
                    preload="mouseover"
                {
                    span class="text-xs mt-0.5" { (item.name) }
                }
            }

            @for item in NAV_ITEMS {
                a
                    href=(item.url)
                    aria-current=[if item.resource == current { Some("page") } else { None }]
                    data-text=(item.name)
                    class="glitch-hover hidden md:block text-2xl text-indigo-400 cursor-pointer hover:bg-teal-400 hover:text-white align-middle transition-all duration-300 aria-[current=page]:text-teal-400 aria-[current=page]:border-l-4 aria-[current=page]:border-teal-400 aria-[current=page]:pl-2 aria-[current=page]:hover:text-white"
                    hx-get=(item.url)
                    hx-target="body"
                    hx-push-url="true"
                    hx-swap="outerHTML"
                    preload="mouseover"
                {
                    (item.name)
                }
            }

            span class="hidden md:block text-slate-400 text-xs select-none lg:whitespace-nowrap" {
                "© Robert Shalders, 2026"
            }
            // AGPL §13: the corresponding source of this network service.
            a
                href="https://github.com/ICreateThunder/profile"
                target="_blank"
                rel="noopener noreferrer"
                class="hidden md:block text-slate-400 text-xs hover:text-teal-400 transition-colors lg:whitespace-nowrap"
            {
                "source"
            }
        }
    }
}

/// Render the navigation sidebar/bottom bar with wrapper div.
/// Used on profile, articles, collection, and article pages.
pub fn navigation(current: &str) -> Markup {
    html! {
        div class="flex flex-col p-8 justify-end shrink-0" {
            (navigation_items(current))
        }
    }
}
