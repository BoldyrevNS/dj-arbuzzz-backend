# Build stage
FROM node:25-alpine as builder

# Install pnpm
RUN npm install -g pnpm@latest

# Create app directory
WORKDIR /app

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
RUN pnpm run build

# Runtime stage
FROM node:25-alpine

# Install pnpm
RUN npm install -g pnpm@latest

# Create app directory
WORKDIR /app

# Copy built application from builder
COPY --from=builder /app/.output /app/.output
COPY --from=builder /app/package.json /app/package.json

# Expose port
EXPOSE 3000

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=3000
ENV NODE_ENV=production

# Run the application
CMD ["node", ".output/server/index.mjs"]
