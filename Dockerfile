ARG userid=10001
ARG groupid=10001

# Build stage: uses cargo-chef so that dependency compilation is cached across
# builds -- only the "cook" layer needs to rerun when the recipe changes.
# See https://github.com/LukeMathWalker/cargo-chef for more information.
FROM lukemathwalker/cargo-chef:latest-rust-1-trixie AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Read by build.rs to construct the build URL embedded in version.json.
ARG github_server_url
ARG github_repository
ARG github_run_id
ENV GITHUB_SERVER_URL=${github_server_url}
ENV GITHUB_REPOSITORY=${github_repository}
ENV GITHUB_RUN_ID=${github_run_id}
RUN cargo build --release --bin reliost

# Runtime stage.
FROM debian:trixie-slim AS runtime

ARG userid
ARG groupid

# openssl: dynamically linked by some dependencies.
# ca-certificates: needed for HTTPS to upstream symbol servers.
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

RUN set -x \
  && groupadd --gid $groupid app \
  && useradd -g app --uid $userid --shell /usr/sbin/nologin --create-home --home-dir /app app

WORKDIR /app
COPY --from=builder /app/target/release/reliost reliost
COPY configuration configuration

ENV APP_ENVIRONMENT="production"
# mozcloud tenant declares application_ports: [8000]. This overrides the
# port from configuration/production.toml, which is 8080 for the hetzner
# deploy (nginx proxies to :8080 there).
ENV RELIOST_SERVER_PORT=8000
EXPOSE 8000

USER app
ENTRYPOINT ["./reliost"]
