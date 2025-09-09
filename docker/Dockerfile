# support cross compiling only on x86_64 host
# you should manifest resulting image for both x86_64 and aarch64(incorrect
FROM --platform=$BUILDPLATFORM node:22-slim AS frontend-builder

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN corepack enable
COPY ./frontend /app
WORKDIR /app

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install
RUN pnpm run build

FROM --platform=$BUILDPLATFORM rust:1.89.0-slim-trixie AS backend-builder

RUN apt update -y \
    && apt install -y musl-tools pkg-config make perl curl xz-utils

WORKDIR /opt/zig
RUN curl -LO https://download.opensuse.org/repositories/home:/clayrisser:/sid/Debian_Unstable/amd64/zig_0.13.0-1_amd64.deb && \
    apt install -y ./zig_0.13.0-1_amd64.deb

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
COPY ./backend .

WORKDIR /compiler
RUN --mount=type=cache,target=target

RUN cargo zigbuild -r --target x86_64-unknown-linux-musl --target aarch64-unknown-linux-musl && \
    mkdir -p /app/linux && \
    cp target/aarch64-unknown-linux-musl/release/backend /app/linux/arm64 && \
    cp target/x86_64-unknown-linux-musl/release/backend /app/linux/amd64

FROM scratch

ARG TARGETPLATFORM

WORKDIR /static
WORKDIR /data
WORKDIR /

COPY --from=frontend-builder /app/build /static
COPY --from=backend-builder /app/${TARGETPLATFORM} /backend

ENV STATIC_DIR="/static"
ENV DATABASE_URL="sqlite://data/db.sqlite?mode=rwc"
ENV BIND_ADDR="0.0.0.0:80"

EXPOSE 80

VOLUME ["/data"]

CMD ["/backend"]
