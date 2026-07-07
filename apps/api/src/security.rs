// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content-Security-Policy construction. The CSP is hash-based (not nonce-based)
//! so it stays deterministic per URL and therefore edge-cacheable: the inline
//! `<style>` is constant (its hash is computed once at startup), and the inline
//! JSON-LD `<script>` hash is computed per page. Both are `sha256-…` source
//! expressions - no `unsafe-inline`, no `unsafe-eval`.

use base64::Engine;
use sha2::{Digest, Sha256};

/// Base64 SHA-256 of `input`, formatted for a CSP `sha256-…` source expression.
pub fn sha256_b64(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(digest)
}

/// Build the Content-Security-Policy for an HTML document, given the base64
/// SHA-256 hashes of the inline `<style>` and inline JSON-LD `<script>`.
///
/// htmx is self-hosted (`script-src 'self'`) and our usage needs no
/// `unsafe-eval`. There are no inline `style=`/`on*=` attributes in the markup,
/// so `style-src` carries only the stylesheet hash.
pub fn content_security_policy(style_hash: &str, script_hash: &str) -> String {
    format!(
        "default-src 'none'; \
         script-src 'self' 'sha256-{script_hash}'; \
         style-src 'sha256-{style_hash}'; \
         img-src 'self' data:; \
         font-src 'self'; \
         connect-src 'self'; \
         base-uri 'none'; \
         form-action 'self'; \
         frame-ancestors 'none'"
    )
}
