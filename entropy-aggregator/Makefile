# Makefile for building SGX enclave for entropy aggregator

# SGX SDK settings
SGX_SDK ?= /opt/intel/sgxsdk
SGX_MODE ?= SIM
SGX_ARCH ?= x64
SGX_DEBUG ?= 1

# Environment setup
include $(SGX_SDK)/environment

# Compiler settings
SGX_COMMON_CFLAGS := -m64
ifeq ($(SGX_DEBUG), 1)
    SGX_COMMON_CFLAGS += -O0 -g
else
    SGX_COMMON_CFLAGS += -O2
endif

# EDL (Enclave Definition Language) file
EDL_FILE := src/tee/sgx/entropy_aggregator.edl

# Enclave settings
ENCLAVE_NAME := sgx_enclave
ENCLAVE_DIR := $(CURDIR)/src/tee/sgx
ENCLAVE_SRC := $(ENCLAVE_DIR)/enclave.rs
ENCLAVE_UNTRUSTED_SRC := $(ENCLAVE_DIR)/untrusted.rs

# Output files
ENCLAVE_LIB := $(ENCLAVE_NAME).so
ENCLAVE_SIGNED := $(ENCLAVE_NAME).signed.so
ENCLAVE_UNSIGNED := $(ENCLAVE_NAME)_unsigned.so
ENCLAVE_CONFIG := $(ENCLAVE_DIR)/enclave.config.xml

# Build directories
BUILD_DIR := build
ENCLAVE_BUILD_DIR := $(BUILD_DIR)/enclave

# Compiler paths
SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx-edger8r
SGX_SIGN := $(SGX_SDK)/bin/x64/sgx-sign

# Default target
.PHONY: all
all: enclave

# Build the SGX enclave
.PHONY: enclave
enclave: $(ENCLAVE_SIGNED)

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(ENCLAVE_BUILD_DIR):
	mkdir -p $(ENCLAVE_BUILD_DIR)

# Generate EDL headers and bridges
$(ENCLAVE_BUILD_DIR)/enclave_t.c: $(EDL_FILE) | $(ENCLAVE_BUILD_DIR)
	$(SGX_EDGER8R) --trusted $(EDL_FILE) --search-path $(SGX_SDK)/include --trusted-dir $(ENCLAVE_BUILD_DIR)

$(ENCLAVE_BUILD_DIR)/enclave_u.c: $(EDL_FILE) | $(ENCLAVE_BUILD_DIR)
	$(SGX_EDGER8R) --untrusted $(EDL_FILE) --search-path $(SGX_SDK)/include --untrusted-dir $(ENCLAVE_BUILD_DIR)

# Build the unsigned enclave
$(ENCLAVE_UNSIGNED): $(ENCLAVE_BUILD_DIR)/enclave_t.c | $(ENCLAVE_BUILD_DIR)
	@echo "Building unsigned enclave..."
	# In a real implementation, this would compile the enclave using the SGX Rust SDK
	# For now, we'll just create a placeholder to show the expected structure
	@mkdir -p $(dir $@)
	@touch $@

# Sign the enclave
$(ENCLAVE_SIGNED): $(ENCLAVE_UNSIGNED)
	@echo "Signing enclave..."
	# In a real implementation, this would use the SGX signing tool
	# For now, we'll just copy the unsigned file as signed for testing
	@cp $(ENCLAVE_UNSIGNED) $(ENCLAVE_SIGNED)

# Clean build artifacts
.PHONY: clean
clean:
	rm -rf $(BUILD_DIR)
	rm -f $(ENCLAVE_UNSIGNED) $(ENCLAVE_SIGNED)

# Run tests with SGX feature
.PHONY: test-sgx
test-sgx:
	cargo test --features=sgx

# Run with SGX feature
.PHONY: run-sgx
run-sgx:
	cargo run --features=sgx

# Build with SGX feature
.PHONY: build-sgx
build-sgx:
	cargo build --features=sgx

# Print configuration
.PHONY: info
info:
	@echo "SGX SDK: $(SGX_SDK)"
	@echo "SGX Mode: $(SGX_MODE)"
	@echo "SGX Debug: $(SGX_DEBUG)"
	@echo "Enclave Name: $(ENCLAVE_NAME)"
	@echo "Build Directory: $(BUILD_DIR)"