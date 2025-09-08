FROM node:22-slim AS frontend-builder

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN corepack enable
COPY ./frontend /app
WORKDIR /app

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install
RUN pnpm run build

FROM rust:1.89.0-slim-trixie AS backend-builder

# Install gcc-aarch64-linux-gnu even if not cross compiling(reuse docker build cache from x86)
RUN apt update -y \
    && apt install -y musl-tools pkg-config make perl gcc-aarch64-linux-gnu

RUN --mount=type=cache,target=~/.cargo/bin/
RUN --mount=type=cache,target=~/.cargo/registry/index/
RUN --mount=type=cache,target=~/.cargo/registry/cache/
RUN --mount=type=cache,target=~/.cargo/git/db/

ARG ARCH=x86_64

RUN rustup target add ${ARCH}-unknown-linux-musl

WORKDIR /prompts
COPY ./prompts .

WORKDIR /compiler
COPY ./backend .

WORKDIR /compiler
RUN --mount=type=cache,target=target

RUN cargo install --target ${ARCH}-unknown-linux-musl --path .

FROM scratch
WORKDIR /static
WORKDIR /data
WORKDIR /

COPY --from=frontend-builder /app/build /static
COPY --from=backend-builder /usr/local/cargo/bin/backend /

ENV STATIC_DIR="/static"
ENV DATABASE_URL="sqlite://data/db.sqlite?mode=rwc"
ENV BIND_ADDR="0.0.0.0:80"

EXPOSE 80

VOLUME ["/data"]

CMD ["/backend"]
