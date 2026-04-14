FROM oven/bun:1-debian AS builder
WORKDIR /build

COPY . .
RUN bun install --frozen-lockfile

ENV SKIP_ENV_VALIDATION=true
ENV NODE_ENV=production
RUN bun run build

FROM oven/bun:1-debian
WORKDIR /app

ENV NODE_ENV=production
COPY --from=builder /build/.next/standalone ./
COPY --from=builder /build/.next/static ./.next/static
COPY --from=builder /build/public ./public

EXPOSE 3000
CMD ["bun", "run", "server.js"]
