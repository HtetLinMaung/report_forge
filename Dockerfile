# Use a Rust base image
FROM rust:slim-buster as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the current directory contents into the container at /app
COPY . /usr/src/app

# Build for release
RUN cargo build --release

# Start a new build stage
FROM debian:12-slim

# Install Node.js, npm, and required dependencies for Puppeteer
RUN apt-get update && \
    apt-get install -y wget gnupg ca-certificates libdrm2 libgbm1 && \
    wget -qO - https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs && \
    apt-get install -y libx11-xcb1 libxcomposite1 libxdamage1 libxi6 libxext6 libxtst6 libnss3 libcups2 libxss1 libxrandr2 libasound2 libpangocairo-1.0-0 libatk1.0-0 libatk-bridge2.0-0 libgtk-3-0 && \
    apt-get install -y fonts-liberation fonts-noto-cjk fonts-noto-color-emoji fonts-noto && \
    apt-get clean && rm -rf /var/lib/apt/lists/* 



# Install sitetopdf globally
RUN npm install -g sitetopdf

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/report_forge /usr/local/bin/

# Copy the static directory
# COPY ./static /static

# Set the command to run your application
CMD ["report_forge"]