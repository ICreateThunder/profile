// SPDX-License-Identifier: AGPL-3.0-or-later
use std::path::Path;
use std::process::Command;

fn main() {
    // Re-run if the input CSS or any Rust template source changes
    println!("cargo::rerun-if-changed=styles/input.css");
    println!("cargo::rerun-if-changed=src/templates/");

    // Skip the Tailwind step when the stylesheet source is absent. cargo-chef's
    // `cook` stage compiles dependencies against a dummy source skeleton that
    // omits non-Rust assets (styles/input.css), so building it there would fail.
    // The real build has the full source present and compiles the stylesheet.
    if !Path::new("styles/input.css").exists() {
        println!(
            "cargo::warning=styles/input.css absent; skipping Tailwind build (dependency-only build)"
        );
        return;
    }

    // Compile Tailwind CSS via Bun + @tailwindcss/cli
    let status = Command::new("bun")
        .args([
            "x",
            "@tailwindcss/cli",
            "-i",
            "styles/input.css",
            "-o",
            "static/styles.css",
            "--minify",
        ])
        .status()
        .expect("failed to run `bun x @tailwindcss/cli` - is Bun installed?");

    assert!(
        status.success(),
        "Tailwind CSS compilation failed with exit code: {status}"
    );
}
