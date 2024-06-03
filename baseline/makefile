NAME = baseline
BUILD_DIR = target/x86_64-fortanix-unknown-sgx/release
KEY = private.pem

BINARY = $(BUILD_DIR)/$(NAME)
TARGET = $(BUILD_DIR)/$(NAME).sgxs
SIGNATURE = $(BUILD_DIR)/$(NAME).sig

HEAP_SIZE = 0x100000000
STACK_SIZE = 0x400000
MAX_THREADS = 9

.PHONY: build run clean

$(TARGET): $(BINARY)
	ftxsgx-elf2sgxs -o $(TARGET) $(BINARY) --heap-size $(HEAP_SIZE) --stack-size $(STACK_SIZE) --threads $(MAX_THREADS)
	sgxs-sign --key $(KEY) $(TARGET) $(SIGNATURE)

$(BINARY): build

build:
	cargo build --release

run: $(TARGET)
	ftxsgx-runner $(TARGET) 2097152 30000 8

clean:
	rm -rf $(BUILD_DIR)