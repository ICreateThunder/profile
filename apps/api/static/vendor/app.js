// SPDX-License-Identifier: AGPL-3.0-or-later
// Client-side retro/CRT enhancements. Progressive enhancement only - the site
// is fully functional without this file. Vendored (served from /static) so it
// satisfies the strict `script-src 'self'` CSP with no inline scripts.
(function () {
  "use strict";

  var reduceMotion =
    window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  var finePointer =
    window.matchMedia && window.matchMedia("(pointer: fine)").matches;
  // Touch phones/tablets: skip the always-on canvas backdrops (warp/grid). They
  // run a per-frame rAF loop that costs more than it adds on mobile; the CSS
  // drops the matching background effects under the same media query.
  var lowPowerBackdrop =
    window.matchMedia &&
    window.matchMedia("(hover: none) and (pointer: coarse)").matches;

  // Flag JS as available - CSS gates the scroll-reveal hidden state on `.js` so
  // the site shows all content if this script never runs.
  document.documentElement.classList.add("js");

  // Manual "reduce effects" toggle (⌘K action), persisted. Folds into the same
  // gate as OS reduced-motion so every motion effect honours it.
  var fxOff = false;
  try {
    fxOff = localStorage.getItem("fx") === "off";
  } catch (e) {
    /* storage blocked */
  }
  if (fxOff) document.documentElement.classList.add("fx-off");
  reduceMotion = reduceMotion || fxOff;

  // ---------------------------------------------------------------------------
  // HTMX View Transitions - turns the body swap into the CRT power-off→on wipe
  // (animation authored in input.css under ::view-transition-*). Honour reduce.
  // ---------------------------------------------------------------------------
  function configureHtmx() {
    if (window.htmx) {
      window.htmx.config.globalViewTransitions = !reduceMotion;
      // htmx injects an inline <style> for .htmx-indicator on load; under our
      // strict hash-based `style-src` (no nonce) that is blocked and raises a
      // CSP violation on every page. We don't use the default indicator styles
      // (add any to the hashed input.css instead), so disable the injection.
      window.htmx.config.includeIndicatorStyles = false;
    }
  }
  if (window.htmx) {
    configureHtmx();
  } else {
    document.addEventListener("htmx:load", configureHtmx, { once: true });
  }

  // Back/forward (popstate) should fade, not play the forward CRT wipe. A
  // transient `.vt-back` on <html> swaps the view-transition animation (CSS).
  window.addEventListener("popstate", function () {
    var el = document.documentElement;
    el.classList.add("vt-back");
    window.setTimeout(function () {
      el.classList.remove("vt-back");
    }, 700);
  });

  // ---------------------------------------------------------------------------
  // Boot loader - first full document load per session only. The overlay markup
  // lives in the layout; we just toggle `.booting` on <html>, then fade out.
  // (Full loads are rare - in-site nav is HTMX - so this is the very first hit.)
  // ---------------------------------------------------------------------------
  function runBoot() {
    if (reduceMotion) return;
    try {
      if (sessionStorage.getItem("booted") === "1") return;
      sessionStorage.setItem("booted", "1");
    } catch (e) {
      /* sessionStorage blocked - just skip the boot */
      return;
    }
    var root = document.documentElement;
    var overlay = document.querySelector(".boot");
    if (!overlay) return;
    var bar = overlay.querySelector(".boot-bar");
    root.classList.add("booting");
    bootMeter(bar, function () {
      overlay.classList.add("boot-fade-out");
      window.setTimeout(function () {
        root.classList.remove("booting");
      }, 360);
    });
  }

  // A small, fluid fill: the meter eases smoothly toward full and completes once
  // the page has actually loaded (or at a hard cap). No stutter or glitch tears -
  // just one clean sweep, then fade.
  function bootMeter(bar, done) {
    var start = (window.performance && performance.now()) || 0;
    var MIN = 360; // let the sweep read before it can finish
    var MAX = 880; // never hang
    var loaded = document.readyState === "complete";
    window.addEventListener(
      "load",
      function () {
        loaded = true;
      },
      { once: true },
    );
    var shown = 0;
    function frame() {
      var elapsed = ((window.performance && performance.now()) || 0) - start;
      // Ease toward ~90% over MIN; finish once loaded past MIN, or at MAX.
      var target = loaded && elapsed >= MIN ? 100 : Math.min(90, (elapsed / MIN) * 90);
      if (elapsed >= MAX) target = 100;
      shown += (target - shown) * 0.12;
      if (target >= 100 && shown > 99.4) shown = 100;
      if (bar) bar.style.setProperty("--p", shown.toFixed(1) + "%");
      if (shown >= 100) {
        if (done) done();
        return;
      }
      window.requestAnimationFrame(frame);
    }
    window.requestAnimationFrame(frame);
  }

  // ---------------------------------------------------------------------------
  // Typewriter tagline - types out real descriptors with the existing blink
  // cursor. Targets [data-typewriter] whose JSON `data-roles` holds the cycle.
  // ---------------------------------------------------------------------------
  var typewriterTimers = [];
  function clearTypewriter() {
    typewriterTimers.forEach(clearTimeout);
    typewriterTimers = [];
  }
  function initTypewriter() {
    var el = document.querySelector("[data-typewriter]");
    if (!el) return;
    var roles;
    try {
      roles = JSON.parse(el.getAttribute("data-roles") || "[]");
    } catch (e) {
      roles = [];
    }
    if (!roles.length) return;

    if (reduceMotion) {
      el.textContent = roles[0];
      return;
    }

    // Start showing the full first role, hold, then delete + cycle.
    var ri = 0,
      ci = roles[0].length,
      deleting = true;
    el.textContent = roles[0];
    function tick() {
      var word = roles[ri];
      ci += deleting ? -1 : 1;
      if (ci < 0) ci = 0;
      el.textContent = word.slice(0, ci);
      var delay = deleting ? 45 : 90;
      if (!deleting && ci >= word.length) {
        deleting = true;
        delay = 1600; // hold the full word
      } else if (deleting && ci <= 0) {
        deleting = false;
        ri = (ri + 1) % roles.length;
        delay = 350;
      }
      typewriterTimers.push(window.setTimeout(tick, delay));
    }
    // Hold the initial full word ~2s before the first delete.
    typewriterTimers.push(window.setTimeout(tick, 2000));
  }

  // ---------------------------------------------------------------------------
  // Cursor-reactive hero - the headline parallax-shifts toward the pointer and
  // a phosphor glow tracks it. Desktop / fine-pointer only, rAF-throttled.
  // ---------------------------------------------------------------------------
  var heroMoveHandler = null;
  function clearHero() {
    if (heroMoveHandler) {
      window.removeEventListener("mousemove", heroMoveHandler);
      heroMoveHandler = null;
    }
  }
  function initHero() {
    if (reduceMotion || !finePointer) return;
    var hero = document.querySelector("[data-hero-react]");
    if (!hero) return;
    var ticking = false;
    var lastX = 0,
      lastY = 0;
    heroMoveHandler = function (e) {
      lastX = e.clientX;
      lastY = e.clientY;
      if (ticking) return;
      ticking = true;
      window.requestAnimationFrame(function () {
        var w = window.innerWidth || 1;
        var h = window.innerHeight || 1;
        var dx = (lastX / w - 0.5) * 2; // -1..1
        var dy = (lastY / h - 0.5) * 2;
        hero.style.transform =
          "translate(" + dx * 8 + "px," + dy * 8 + "px)";
        ticking = false;
      });
    };
    window.addEventListener("mousemove", heroMoveHandler);
  }

  // ---------------------------------------------------------------------------
  // Hover-glitch fallback - ensure any .glitch-hover without data-text gets one
  // from its text content (templates set it explicitly; this catches the rest).
  // ---------------------------------------------------------------------------
  function initGlitchData() {
    document.querySelectorAll(".glitch-hover").forEach(function (el) {
      if (!el.hasAttribute("data-text")) {
        el.setAttribute("data-text", el.textContent.trim());
      }
    });
  }

  // ---------------------------------------------------------------------------
  // Scroll-reveal - stagger .reveal elements in as they enter the viewport.
  // ---------------------------------------------------------------------------
  var revealObserver = null;
  function initReveal() {
    var items = document.querySelectorAll(".reveal:not(.in)");
    if (!items.length) return;
    if (reduceMotion || !("IntersectionObserver" in window)) {
      items.forEach(function (el) {
        el.classList.add("in");
      });
      return;
    }
    if (revealObserver) revealObserver.disconnect();
    revealObserver = new IntersectionObserver(
      function (entries) {
        entries.forEach(function (entry, i) {
          if (entry.isIntersecting) {
            var el = entry.target;
            window.setTimeout(function () {
              el.classList.add("in");
            }, i * 70);
            revealObserver.unobserve(el);
          }
        });
      },
      { rootMargin: "0px 0px -10% 0px", threshold: 0.1 }
    );
    items.forEach(function (el) {
      revealObserver.observe(el);
    });
  }

  // ---------------------------------------------------------------------------
  // 3D synthwave grid background (homepage) - pure canvas 2D pseudo-3D, no
  // dependency. A receding floor grid + horizon "sun" arc, scrolling toward the
  // viewer. Self-contained so it stays small and CSP-clean; degrades to the
  // noise texture when JS is off (the canvas simply stays blank).
  // ---------------------------------------------------------------------------
  var bgRaf = 0;
  var bgResize = null;
  function clearBgGrid() {
    if (bgRaf) {
      window.cancelAnimationFrame(bgRaf);
      bgRaf = 0;
    }
    if (bgResize) {
      window.removeEventListener("resize", bgResize);
      bgResize = null;
    }
  }
  function initBgGrid() {
    var canvas = document.querySelector("[data-bg-grid]");
    if (!canvas || !canvas.getContext) return;
    var ctx = canvas.getContext("2d");
    if (!ctx) return;

    var w = 0,
      h = 0,
      dpr = 1;
    function resize() {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = canvas.clientWidth;
      h = canvas.clientHeight;
      canvas.width = Math.max(1, Math.floor(w * dpr));
      canvas.height = Math.max(1, Math.floor(h * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
    resize();
    bgResize = resize;
    window.addEventListener("resize", bgResize);

    var teal = "45,212,191";

    function frame(t) {
      ctx.clearRect(0, 0, w, h);
      var horizon = h * 0.46;
      var vpx = w / 2;

      // Horizon sun - soft teal radial glow rising from the horizon line.
      var sunR = Math.min(w, h) * 0.22;
      var grad = ctx.createRadialGradient(vpx, horizon, 0, vpx, horizon, sunR);
      grad.addColorStop(0, "rgba(" + teal + ",0.18)");
      grad.addColorStop(1, "rgba(" + teal + ",0)");
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(vpx, horizon, sunR, Math.PI, 0);
      ctx.fill();

      // Floor grid - perspective rows scrolling toward the viewer.
      var rows = 20;
      var scroll = t * 0.00009;
      for (var i = 0; i < rows; i++) {
        var p = ((i + (scroll % 1)) % rows) / rows; // 0..1
        var z = p * p; // bunch near the horizon
        var y = horizon + z * (h - horizon);
        var a = 0.22 * (1 - p) + 0.02;
        ctx.strokeStyle = "rgba(" + teal + "," + a + ")";
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
        ctx.stroke();
      }

      // Retrowave perspective lines - evenly spaced by ANGLE from the vanishing
      // point, fanning the whole half-plane below the horizon. Even angular
      // spacing keeps the lines consistent and covers the full frame edge to
      // edge (no middle bunching, no bare corners).
      ctx.lineWidth = 1;
      ctx.strokeStyle = "rgba(" + teal + ",0.13)";
      var n = 22;
      var D = (w + h) * 2;
      for (var j = 1; j < n; j++) {
        var ang = Math.PI * (j / n);
        ctx.beginPath();
        ctx.moveTo(vpx, horizon);
        ctx.lineTo(vpx + Math.cos(ang) * D, horizon + Math.sin(ang) * D);
        ctx.stroke();
      }

      // bright horizon line
      ctx.strokeStyle = "rgba(" + teal + ",0.35)";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(0, horizon);
      ctx.lineTo(w, horizon);
      ctx.stroke();

      bgRaf = window.requestAnimationFrame(frame);
    }

    if (reduceMotion) {
      frame(0);
      clearBgGrid(); // single static frame, no loop
    } else {
      bgRaf = window.requestAnimationFrame(frame);
    }
  }

  // ---------------------------------------------------------------------------
  // Warp starfield (profile) - teal + indigo streaks emanating from the centre
  // and accelerating outward (a calm "warp" field). One canvas, recycled
  // particles, rAF-driven - no dependency, CSP-clean. Kept low-alpha so the card
  // text stays readable. Under reduced motion it paints a single still frame.
  // ---------------------------------------------------------------------------
  var warpRaf = 0;
  var warpResize = null;
  function clearWarp() {
    if (warpRaf) {
      window.cancelAnimationFrame(warpRaf);
      warpRaf = 0;
    }
    if (warpResize) {
      window.removeEventListener("resize", warpResize);
      warpResize = null;
    }
  }
  function initWarp() {
    var canvas = document.querySelector("[data-warp]");
    if (!canvas || !canvas.getContext) return;
    var ctx = canvas.getContext("2d");
    if (!ctx) return;

    var w = 0,
      h = 0,
      dpr = 1,
      cx = 0,
      cy = 0,
      maxR = 1;
    function resize() {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = canvas.clientWidth;
      h = canvas.clientHeight;
      canvas.width = Math.max(1, Math.floor(w * dpr));
      canvas.height = Math.max(1, Math.floor(h * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      cx = w / 2;
      cy = h / 2;
      maxR = Math.sqrt(cx * cx + cy * cy) || 1;
    }
    resize();
    warpResize = resize;
    window.addEventListener("resize", warpResize);

    var colors = ["45,212,191", "129,140,248"]; // teal-400, indigo-400
    var N = 120;
    var stars = [];
    // `seeded` spreads the initial field across the radius so the first frame is
    // already populated; respawns start near the centre so streaks grow outward.
    function reset(s, seeded) {
      s.a = Math.random() * Math.PI * 2;
      s.r = seeded ? Math.random() * maxR : 4 + Math.random() * 16;
      s.speed = 0.006 + Math.random() * 0.013; // fraction of travel per frame
      s.c = colors[(Math.random() * colors.length) | 0];
      return s;
    }
    for (var i = 0; i < N; i++) stars.push(reset({}, true));

    function frame() {
      ctx.clearRect(0, 0, w, h);
      for (var i = 0; i < stars.length; i++) {
        var s = stars[i];
        var pr = s.r;
        s.r += (s.r + 6) * s.speed; // accelerate as it nears the edge
        var ca = Math.cos(s.a),
          sa = Math.sin(s.a);
        var t = s.r / maxR; // 0 at centre … 1 at the corner
        var alpha = Math.min(0.8, t * 0.95); // fade in from the centre
        ctx.strokeStyle = "rgba(" + s.c + "," + alpha.toFixed(3) + ")";
        ctx.lineWidth = 0.9 + t * 2;
        ctx.beginPath();
        ctx.moveTo(cx + ca * pr, cy + sa * pr);
        ctx.lineTo(cx + ca * s.r, cy + sa * s.r);
        ctx.stroke();
        if (s.r > maxR) reset(s, false);
      }
      warpRaf = window.requestAnimationFrame(frame);
    }

    if (reduceMotion) {
      frame(); // a single still field …
      clearWarp(); // … no loop
    } else {
      warpRaf = window.requestAnimationFrame(frame);
    }
  }

  // ---------------------------------------------------------------------------
  // Articles live filter - filters the cards as you type. Progressive
  // enhancement: the search box is hidden in markup and revealed here, so the
  // no-JS site still shows the full list. Token AND-match over each card's
  // `data-search` haystack (title + collection + tags + description).
  // ---------------------------------------------------------------------------
  function initArticleSearch() {
    var input = document.querySelector("[data-article-search]");
    if (!input) return;
    var wrap = document.querySelector("[data-article-search-wrap]");
    if (wrap) wrap.hidden = false;
    var empty = document.querySelector("[data-article-empty]");
    var cards = Array.prototype.slice.call(document.querySelectorAll("[data-article]"));
    function apply() {
      var tokens = input.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
      var any = false;
      cards.forEach(function (card) {
        var key = card.getAttribute("data-search") || "";
        var match = tokens.every(function (t) {
          return key.indexOf(t) !== -1;
        });
        card.hidden = !match;
        if (match) any = true;
      });
      if (empty) empty.hidden = any;
    }
    if (!input.dataset.searchBound) {
      input.dataset.searchBound = "1";
      input.addEventListener("input", apply);
    }
    apply();
  }

  // ---------------------------------------------------------------------------
  // (Re)bind everything. Called on first load and after every HTMX body swap
  // (the <body> is replaced, so per-node handlers must be re-attached).
  // ---------------------------------------------------------------------------
  function bind() {
    clearTypewriter();
    clearHero();
    clearBgGrid();
    clearWarp();
    initGlitchData();
    initTypewriter();
    initHero();
    initReveal();
    if (!lowPowerBackdrop) {
      initBgGrid();
      initWarp();
    }
    initArticleSearch();
  }

  function onReady() {
    runBoot();
    bind();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", onReady);
  } else {
    onReady();
  }

  // After every HTMX body swap: re-init effects, announce the new route to
  // screen readers, and move focus to <main> (the body was replaced).
  document.addEventListener("htmx:afterSettle", function () {
    bind();
    var ann = document.getElementById("route-announcer");
    if (ann) ann.textContent = document.title;
    var main = document.getElementById("main");
    if (main && main.focus) main.focus({ preventScroll: true });
  });

  // ---------------------------------------------------------------------------
  // Command palette (⌘/Ctrl-K) + keyboard shortcuts. Listeners are bound once on
  // document and query the palette on demand, so they survive HTMX body swaps.
  // ---------------------------------------------------------------------------
  var paletteReturnFocus = null;
  var pendingG = false;

  function getPalette() {
    return document.querySelector("[data-cmdk]");
  }
  function visibleItems(pal) {
    return Array.prototype.slice
      .call(pal.querySelectorAll(".cmdk-item"))
      .filter(function (li) {
        return !li.hidden;
      });
  }
  function setActive(pal, idx) {
    var items = visibleItems(pal);
    if (!items.length) {
      pal._active = 0;
      return;
    }
    idx = (idx + items.length) % items.length;
    items.forEach(function (li, i) {
      li.classList.toggle("is-active", i === idx);
    });
    items[idx].scrollIntoView({ block: "nearest" });
    pal._active = idx;
  }
  function filterPalette(pal, q) {
    var tokens = q.trim().toLowerCase().split(/\s+/).filter(Boolean);
    var any = false;
    pal.querySelectorAll(".cmdk-item").forEach(function (li) {
      var key = li.getAttribute("data-key") || "";
      var match = tokens.every(function (t) {
        return key.indexOf(t) !== -1;
      });
      li.hidden = !match;
      if (match) any = true;
    });
    // hide group headers that have no visible items beneath them
    pal.querySelectorAll(".cmdk-group").forEach(function (g) {
      var li = g.nextElementSibling;
      var vis = false;
      while (li && !li.classList.contains("cmdk-group")) {
        if (li.classList.contains("cmdk-item") && !li.hidden) {
          vis = true;
          break;
        }
        li = li.nextElementSibling;
      }
      g.hidden = !vis;
    });
    var empty = pal.querySelector(".cmdk-empty");
    if (empty) empty.hidden = any;
    setActive(pal, 0);
  }
  function openPalette() {
    var pal = getPalette();
    if (!pal || !pal.hidden) return;
    paletteReturnFocus = document.activeElement;
    pal.hidden = false;
    var input = pal.querySelector(".cmdk-input");
    input.value = "";
    filterPalette(pal, "");
    input.focus();
  }
  function closePalette() {
    var pal = getPalette();
    if (!pal || pal.hidden) return;
    pal.hidden = true;
    if (paletteReturnFocus && paletteReturnFocus.focus) paletteReturnFocus.focus();
  }
  function activatePalette(pal) {
    var items = visibleItems(pal);
    var item = items[pal._active || 0];
    if (!item) return;
    var el = item.querySelector(".cmdk-link");
    if (!el) return;
    var action = el.getAttribute("data-action");
    if (action) {
      runAction(action, el.getAttribute("data-value") || "");
      closePalette();
    } else {
      closePalette();
      el.click(); // HTMX nav or external link
    }
  }
  function runAction(action, value) {
    if (action === "toggle-fx") {
      toggleFx();
    } else if (action === "copy-email" && navigator.clipboard) {
      navigator.clipboard.writeText(value);
    }
  }
  function toggleFx() {
    var off = !document.documentElement.classList.contains("fx-off");
    document.documentElement.classList.toggle("fx-off", off);
    try {
      localStorage.setItem("fx", off ? "off" : "on");
    } catch (e) {
      /* ignore */
    }
    reduceMotion =
      (window.matchMedia &&
        window.matchMedia("(prefers-reduced-motion: reduce)").matches) ||
      off;
    bind(); // restart/stop the motion-driven effects live
  }
  function navigateTo(href) {
    var link = document.querySelector('.cmdk-link[href="' + href + '"]');
    if (link) {
      link.click();
    } else if (window.htmx) {
      window.htmx.ajax("GET", href, { target: "body", swap: "outerHTML" });
    } else {
      window.location.href = href;
    }
  }

  document.addEventListener("keydown", function (e) {
    var pal = getPalette();
    if ((e.metaKey || e.ctrlKey) && (e.key === "k" || e.key === "K")) {
      e.preventDefault();
      if (pal && !pal.hidden) closePalette();
      else openPalette();
      return;
    }
    if (pal && !pal.hidden) {
      if (e.key === "Escape") {
        e.preventDefault();
        closePalette();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setActive(pal, (pal._active || 0) + 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setActive(pal, (pal._active || 0) - 1);
      } else if (e.key === "Enter") {
        e.preventDefault();
        activatePalette(pal);
      }
      return;
    }
    // plain shortcuts - ignore while typing in a field
    var tag = (e.target && e.target.tagName) || "";
    if (tag === "INPUT" || tag === "TEXTAREA" || (e.target && e.target.isContentEditable))
      return;
    if (e.key === "/") {
      e.preventDefault();
      openPalette();
      return;
    }
    if (e.key === "g") {
      pendingG = true;
      window.setTimeout(function () {
        pendingG = false;
      }, 800);
      return;
    }
    if (pendingG) {
      pendingG = false;
      var dest = { h: "/", p: "/profile", a: "/articles" }[e.key];
      if (dest) {
        e.preventDefault();
        navigateTo(dest);
      }
    }
  });

  document.addEventListener("input", function (e) {
    if (e.target && e.target.classList && e.target.classList.contains("cmdk-input")) {
      var pal = getPalette();
      if (pal) filterPalette(pal, e.target.value);
    }
  });

  document.addEventListener("click", function (e) {
    if (!e.target.closest) return;
    var link = e.target.closest(".cmdk-link");
    if (link) {
      var action = link.getAttribute("data-action");
      if (action) {
        e.preventDefault();
        runAction(action, link.getAttribute("data-value") || "");
      }
      closePalette();
      return;
    }
    // click on the backdrop (outside the dialog) closes the palette
    var pal = getPalette();
    if (pal && !pal.hidden && !e.target.closest(".cmdk")) closePalette();
  });
})();
