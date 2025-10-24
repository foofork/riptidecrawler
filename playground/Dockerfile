# Multi-stage build for RipTide Playground
# Disk optimization: Alpine base with aggressive cleanup

# Stage 1: Build
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies with cleanup
# Disk optimization: npm ci with cache cleanup
RUN npm ci --only=production --no-audit --no-fund && \
    npm cache clean --force

# Copy source files
COPY . .

# Build for production and cleanup in same layer to reduce disk usage
RUN npm run build && \
    rm -rf node_modules/.cache \
           /root/.npm \
           /tmp/* \
           /var/tmp/*

# Stage 2: Production
# Disk optimization: Minimal nginx alpine image
FROM nginx:alpine

# Copy built files from builder (only production assets)
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Remove unnecessary nginx files to save disk space
RUN rm -rf /usr/share/nginx/html/*.map \
           /var/cache/apk/* \
           /tmp/* \
           /var/tmp/*

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --quiet --tries=1 --spider http://localhost/ || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
