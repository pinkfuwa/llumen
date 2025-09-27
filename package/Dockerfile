# support cross compiling only on x86_64 host
# you should manifest resulting image for both x86_64 and aarch64
FROM --platform=$BUILDPLATFORM node:22-slim AS frontend-builder

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
ENV CI=true

RUN corepack enable
COPY ./frontend /app
WORKDIR /app

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

FROM --platform=$BUILDPLATFORM rust:1.90.0-slim-trixie AS backend-builder

RUN apt update -y \
    && apt install -y musl-tools pkg-config make perl curl xz-utils

COPY ./package/install-zig.sh /usr/local/bin/install-zig.sh
RUN chmod +x /usr/local/bin/install-zig.sh
RUN install-zig.sh

ENV PATH="/opt/zig:$PATH"

WORKDIR /

RUN --mount=type=cache,target=~/.cargo/bin/
RUN --mount=type=cache,target=~/.cargo/registry/index/
RUN --mount=type=cache,target=~/.cargo/registry/cache/
RUN --mount=type=cache,target=~/.cargo/git/db/

RUN cargo install --locked cargo-zigbuild

RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

WORKDIR /prompts
COPY ./prompts .

WORKDIR /compiler

WORKDIR /compiler
RUN --mount=type=cache,id=target,target=target \
    cargo zigbuild -r --target x86_64-unknown-linux-musl --target aarch64-unknown-linux-musl && \
    mkdir -p /app/linux && \
    cp target/aarch64-unknown-linux-musl/release/backend /app/linux/arm64 && \
    cp target/x86_64-unknown-linux-musl/release/backend /app/linux/amd64

FROM alpine AS ca

FROM scratch

ARG TARGETPLATFORM

LABEL org.opencontainers.image.title="llumen"
LABEL org.opencontainers.image.description="A lightweight, self-hostable LLM chat application."
LABEL org.opencontainers.image.url="https://github.com/pinkfuwa/llumen"
LABEL org.opencontainers.image.source="https://github.com/pinkfuwa/llumen"
LABEL org.opencontainers.image.authors="eason0729, kaiyohugo"
LABEL org.opencontainers.image.vendor="llumen"

WORKDIR /static
WORKDIR /data
WORKDIR /

COPY --from=ca /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=frontend-builder /app/build /static
COPY --from=backend-builder /app/${TARGETPLATFORM} /backend

ENV STATIC_DIR="/static"
ENV DATABASE_URL="sqlite://data/db.sqlite?mode=rwc"
ENV BLOB_URL="/data/blobs.redb"
ENV BIND_ADDR="0.0.0.0:80"
ENV RUST_LOG=none,backend=debug

EXPOSE 80

VOLUME ["/data"]

CMD ["/backend"]
