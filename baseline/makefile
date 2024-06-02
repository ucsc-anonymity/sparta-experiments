HEAP_SIZE = 0x100000000
STACK_SIZE = 0x400000
MAX_THREADS = 9

BUILD_DIR = target/x86_64-fortanix-unknown-sgx/release

BINARY = $(BUILD_DIR)/baseline
TARGET = $(BUILD_DIR)/baseline.sgxs
SIGNATURE = $(BUILD_DIR)/baseline.sig

.PHONY: clean run

$(TARGET): $(BINARY)
	ftxsgx-elf2sgxs -o $(TARGET) $(BINARY) --heap-size $(HEAP_SIZE) --stack-size $(STACK_SIZE) --threads $(MAX_THREADS)
	sgxs-sign --key private.pem $(TARGET) $(SIGNATURE)

$(BINARY):
	cargo build --release

run: $(TARGET)
	ftxsgx-runner $(TARGET) 3 7 8

clean:
	rm -rf $(BUILD_DIR)