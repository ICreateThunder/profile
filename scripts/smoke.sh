#!/usr/bin/env bash
# Container smoke test (Tier 2). Builds the runtime image, runs it, and probes
# happy + unhappy paths and the security-header invariants against the *real
# image* - catching what in-process tests can't: a broken Dockerfile, an asset
# that wasn't baked in, a wrong static path, a bad entrypoint.
#
# Works with docker or podman (auto-detected; override with ENGINE=...).
# Used locally (`scripts/smoke.sh`) and by the CI `smoke` job.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

ENGINE="${ENGINE:-}"
if [ -z "$ENGINE" ]; then
    if command -v podman >/dev/null 2>&1; then ENGINE=podman; else ENGINE=docker; fi
fi
IMG="profile:smoke"
NAME="profile-smoke-$$"
PORT="${PORT:-18080}"
B="http://127.0.0.1:${PORT}"
fail=0

cleanup() { "$ENGINE" rm -f "$NAME" >/dev/null 2>&1 || true; }
trap cleanup EXIT

echo "== build image ($ENGINE) =="
"$ENGINE" build -t "$IMG" -f Dockerfile .

echo "== run =="
"$ENGINE" run -d --name "$NAME" -p "${PORT}:8080" "$IMG" >/dev/null

echo "== wait for readiness =="
for _ in $(seq 1 30); do
    if curl -fsS "$B/healthz" >/dev/null 2>&1; then break; fi
    sleep 1
done

status() { # path expected
    local got
    got=$(curl -s -o /dev/null -w '%{http_code}' "$B$1" || echo 000)
    if [ "$got" != "$2" ]; then
        echo "FAIL  $1  expected $2 got $got"; fail=1
    else
        echo "ok    $1  -> $got"
    fi
}

echo "== happy paths =="
for p in / /profile /articles /rss.xml /sitemap.xml \
         /.well-known/security.txt /.well-known/http-msg-sig.jwk /healthz /readyz; do
    status "$p" 200
done
status /teapot 418

echo "== unhappy paths =="
status /nope-404 404
status /projects/no-such-slug 404

echo "== security headers on / =="
hdrs=$(curl -sS -D - -o /dev/null "$B/")
for h in 'content-security-policy' 'x-frame-options: DENY' \
         'x-content-type-options: nosniff' 'cross-origin-opener-policy' \
         'repr-digest' 'signature-input'; do
    if echo "$hdrs" | grep -iq "$h"; then echo "ok    header ~ $h"; else echo "FAIL  header missing: $h"; fail=1; fi
done
if echo "$hdrs" | grep -i 'content-security-policy' | grep -qi 'unsafe-inline'; then
    echo "FAIL  CSP contains unsafe-inline"; fail=1
else
    echo "ok    CSP has no unsafe-inline"
fi

if [ "$fail" -eq 0 ]; then echo; echo "SMOKE OK"; else echo; echo "SMOKE FAILED"; fi
exit $fail
