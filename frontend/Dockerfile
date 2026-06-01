# syntax=docker/dockerfile:1

FROM oven/bun:1.3.12-debian AS builder
WORKDIR /build

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY . .
ENV SKIP_ENV_VALIDATION=true
ENV NODE_ENV=production

ARG VITE_BACKEND_URL=https://api.myrtle.moe
ARG VITE_SITE_URL=https://myrtle.moe
ENV VITE_BACKEND_URL=$VITE_BACKEND_URL
ENV VITE_SITE_URL=$VITE_SITE_URL

RUN bun run build

FROM oven/bun:1.3.12-debian AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV PORT=3000
ENV HOST=0.0.0.0

COPY --from=builder /build/.output ./.output

USER bun

EXPOSE 3000
CMD ["bun", "./.output/server/index.mjs"]
