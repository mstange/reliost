ARG userid=10001
ARG groupid=10001

# First, we need to build and cache our dependencies so it doesn't have to
# rebuild them every time we build this Dockerfile.
# See https://github.com/LukeMathWalker/cargo-chef for more information.
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

# ARG statements goe out of scope after each FROM statement. We need to
# re-declare them on each stage where we want to use.
ARG userid
ARG groupid

# Create the user we'll run the build commands with. Its home is configured to
# be the directory /app. It helps avoiding warnings when running tests and
# building the app later.
RUN set -x \
  && groupadd --gid $groupid app \
  && useradd -g app --uid $userid --shell /usr/sbin/nologin --create-home --home-dir /app app

USER app
WORKDIR /app

# Let's start with preparing the dependencies.
FROM chef AS planner
# Copy the project files into the planner.
COPY --chown=app:app . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

# Continue with building our project.
FROM chef AS builder

# Copy the recipe.json file from the planner stage.
COPY --from=planner /app/recipe.json recipe.json

# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY --chown=app:app . .

# These environment variables from GitHub Actions are needed when generating the
# version file. We pass it using the "arguments" mechanism from docker.
ARG github_build_url
ARG github_run_id
ENV GITHUB_BUILD_URL=${github_build_url}
ENV GITHUB_RUN_ID=${github_run_id}

# Build our project.
RUN cargo build --release --bin reliost

# This is the final image that will be used in production.
FROM debian:bookworm-slim AS runtime
ENV PORT=8080
# Set the app environment to production.
ENV APP_ENVIRONMENT="production"

# ARG statements goe out of scope after each FROM statement. We need to
# re-declare them on each stage where we want to use.
ARG userid
ARG groupid

# We create the runtime user.
RUN set -x \
  && groupadd --gid $groupid app \
  && useradd -g app --uid $userid --shell /usr/sbin/nologin --create-home --home-dir /app app

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections.
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  # Clean the downloaded lists so that they don't increase the layer size.
  && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder image.
COPY --from=builder /app/target/release/reliost reliost

# Copy the version.json file as it's needed for the __version__ endpoint.
COPY --from=builder /app/version.json version.json

# Copy the configuration files as they are needed for the runtime.
COPY configuration configuration

# This is the default port as configured above.
EXPOSE $PORT

# Set the user to app again, this time on the final runtime image.
USER app

# Run the application.
ENTRYPOINT ["./reliost"]
