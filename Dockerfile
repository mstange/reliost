# First, we need to build and cache our dependencies so it doesn't have to
# rebuild them every time we build this Dockerfile.
# See https://github.com/LukeMathWalker/cargo-chef for more information.
FROM lukemathwalker/cargo-chef:latest-rust-slim-bullseye as chef
WORKDIR /app

# Let's start with preparing the dependencies.
FROM chef as planner
# Copy the project files into the planner.
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

# Continue with building our project.
FROM chef as builder

# Copy the recipe.json file from the planner stage.
COPY --from=planner /app/recipe.json recipe.json

# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .

# Build our project.
RUN cargo build --release --bin reliost

# This is the final image that will be used in production.
FROM debian:bullseye-slim AS runtime
WORKDIR /app
ENV PORT=8080
# Set the app environment to production.
ENV APP_ENVIRONMENT="production"

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

# Copy the configuration files as they are needed for the runtime.
COPY configuration configuration

# This is the default port as configured above.
EXPOSE $PORT

# Run the application.
ENTRYPOINT ["./reliost"]
