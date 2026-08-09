FROM oven/bun:1 AS builder
WORKDIR /app
ENV NODE_ENV=development

COPY package.json ./
COPY packages packages
COPY apps/web-client apps/web-client

RUN bun install

WORKDIR /app/apps/web-client
RUN bun run build

FROM oven/bun:1-slim
WORKDIR /app
ENV NODE_ENV=production

COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/packages ./packages
COPY --from=builder /app/apps/web-client ./apps/web-client

WORKDIR /app/apps/web-client
EXPOSE 3000
CMD ["bun", "./build/index.js"]
