# Stage 1: Build
FROM rust:1.88 AS builder  
WORKDIR /rax-ftp-client
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Create project directory structure
RUN mkdir -p /app/rax-ftp-client/client_root

# Copy binary and config from builder stage
COPY --from=builder /rax-ftp-client/target/release/rax-ftp-client /app/rax-ftp-client/
COPY --from=builder /rax-ftp-client/config.toml /app/rax-ftp-client/

# Volume mount for persistent client files
VOLUME ["/app/rax-ftp-client/client_root"]

# Container environment overrides
ENV RAX_FTP_HOST=rax-ftp-server
ENV RAX_FTP_LOCAL_DIRECTORY=/app/rax-ftp-client/client_root

# Run FTP client
CMD ["./rax-ftp-client/rax-ftp-client"]