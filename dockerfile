# Stage 1: Build
FROM rust:1.88 AS builder  
WORKDIR /rax-ftp-client
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install utilities for generating sample files
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create project directory structure
RUN mkdir -p /app/rax-ftp-client/client_root

# Copy binary and config from builder stage
COPY --from=builder /rax-ftp-client/target/release/rax-ftp-client /app/rax-ftp-client/
COPY --from=builder /rax-ftp-client/config.toml /app/rax-ftp-client/

# Create default directories in client_root
RUN mkdir -p /app/rax-ftp-client/client_root/uploads \
    /app/rax-ftp-client/client_root/downloads \
    /app/rax-ftp-client/client_root/samples

# Create sample text files for uploading
RUN echo "Hello from RAX FTP Client!" > /app/rax-ftp-client/client_root/samples/hello.txt && \
    echo "This file will be uploaded to the server." > /app/rax-ftp-client/client_root/samples/upload_me.txt && \
    echo "Client test document for FTP transfer testing." > /app/rax-ftp-client/client_root/samples/client_doc.txt

# Create a sample log file
RUN printf "2025-01-01 10:00:00 - Client started\n2025-01-01 10:00:01 - Connected to server\n2025-01-01 10:00:02 - Authentication successful\n" > /app/rax-ftp-client/client_root/samples/client.log

# Create a sample configuration file
RUN printf "# Client Configuration\nmode=active\ntimeout=30\nretries=3\n" > /app/rax-ftp-client/client_root/samples/settings.conf

# Create a sample CSV for upload testing
RUN printf "Product,Price,Stock\nLaptop,999.99,15\nMouse,29.99,50\nKeyboard,79.99,30\n" > /app/rax-ftp-client/client_root/samples/inventory.csv

# Download a small sample PDF
RUN curl -L "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf" \
    -o /app/rax-ftp-client/client_root/samples/report.pdf 2>/dev/null || \
    echo "%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /Resources 4 0 R /MediaBox [0 0 612 792] /Contents 5 0 R >>\nendobj\n4 0 obj\n<< /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >>\nendobj\n5 0 obj\n<< /Length 44 >>\nstream\nBT /F1 12 Tf 100 700 Td (Client Report) Tj ET\nendstream\nendobj\nxref\n0 6\ntrailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n0\n%%EOF" > /app/rax-ftp-client/client_root/samples/report.pdf

# Create a small sample audio file
RUN printf "\xFF\xFB\x90\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" > /app/rax-ftp-client/client_root/samples/sound.mp3

# Create info files in directories
RUN echo "Place files here to upload to server" > /app/rax-ftp-client/client_root/uploads/readme.txt && \
    echo "Downloaded files from server will appear here" > /app/rax-ftp-client/client_root/downloads/readme.txt

# Volume mount for persistent client files
VOLUME ["/app/rax-ftp-client/client_root"]

# Container environment overrides
ENV RAX_FTP_HOST=172.20.0.10
ENV RAX_FTP_LOCAL_DIRECTORY=/app/rax-ftp-client/client_root

# Run FTP client
CMD ["./rax-ftp-client/rax-ftp-client"]