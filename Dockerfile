# Stage 1: chef - shared base with cargo-chef installed.
# cargo-chef caches the dependency-compile as its own layer, so ordinary source
# or asset edits no longer trigger a full dependency rebuild - only a change to
# the dependency set (recipe.json) does.
# Base pinned by manifest-list digest (tag kept for readability). Dependabot
# (docker ecosystem) bumps the digest; a moved/compromised tag cannot swap it.
FROM rust:1.97-bookworm@sha256:7d0723df719e7f213b69dc7c8c595985c3f4b060cfbee4f7bc0e347a86fe3b6a AS chef
# Pinned for reproducibility; Dependabot (cargo) surfaces bumps.
RUN cargo install cargo-chef --locked --version 0.1.77
WORKDIR /app

# Stage 2: planner - distil the workspace into a dependency recipe.
# recipe.json is a function of the manifests only, so the cook layer below stays
# cached across edits that do not touch dependencies.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: builder - cook dependencies (cached), then build the binary.
FROM chef AS builder

# Bun compiles Tailwind via apps/api/build.rs; needed for the real build only.
# (build.rs skips the Tailwind step when styles/input.css is absent, as during
# the cook stage below.)
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# cargo-auditable embeds a compressed dependency manifest into the binary, so an
# image/binary scanner (Trivy) can enumerate Rust-crate CVEs in the shipped
# artifact - a stripped distroless image has no Cargo.lock for scanners to read.
RUN cargo install cargo-auditable --locked --version 0.7.5

# Cook: compile just the dependencies described by the recipe. This layer is
# reused as long as recipe.json is unchanged - the expensive part is cached.
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build: full source (context already trimmed by .dockerignore), Tailwind, binary
# with the embedded audit manifest.
COPY . .
RUN cd apps/api && bun install --frozen-lockfile
RUN cargo auditable build --release --locked -p portfolio-api

# Stage 4: runtime
# Distroless static - no libc beyond glibc, no shell, no package manager.
# Pinned by manifest-list digest (see note on the builder base above).
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:ce0d66bc0f64aae46e6a03add867b07f42cc7b8799c949c2e898057b7f75a151

WORKDIR /app

COPY --from=builder /app/target/release/portfolio-api ./portfolio-api
COPY --from=builder /app/apps/api/static ./static
COPY --from=builder /app/apps/api/src/content ./src/content

ENV PORT=8080
ENV METRICS_PORT=9090
EXPOSE 8080 9090

ENTRYPOINT ["./portfolio-api"]
