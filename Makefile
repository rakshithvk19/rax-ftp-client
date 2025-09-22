# Run the FTP server
client:
	RUST_LOG=info cargo run

# Clean build artifacts
clean:
	cargo clean

## Run clippy
clippy:
	cargo clippy &> clippy.txt

# Default target
.PHONY: client debug trace build release clean