ARG userid=10001
ARG groupid=10001

FROM debian:bookworm-slim

# VERSION must be passed as a build arg, e.g. --build-arg VERSION=0.1.0
ARG VERSION
ARG userid
ARG groupid

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends curl ca-certificates xz-utils openssl \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

RUN set -x \
  && groupadd --gid $groupid app \
  && useradd -g app --uid $userid --shell /usr/sbin/nologin --create-home --home-dir /app app

WORKDIR /app

RUN curl -LsSf \
    "https://github.com/mstange/reliost/releases/download/v${VERSION}/reliost-v${VERSION}-x86_64-unknown-linux-gnu.tar.xz" \
  | tar -xJ --strip-components=1

COPY configuration configuration

ENV PORT=8080
ENV APP_ENVIRONMENT="production"
EXPOSE $PORT

USER app
ENTRYPOINT ["./reliost"]
