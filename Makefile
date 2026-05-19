all:
	@echo "make [clean|so|kotlin]"
clean:
	cargo clean

so: clean
	cargo ndk \
	  -t arm64-v8a \
	  -t armeabi-v7a \
	  -t x86_64 \
	  build --release
kotlin:
	cargo run --bin uniffi_bindgen -- generate \
      --library target/aarch64-linux-android/release/librelaysms_spec_payload.so \
      --language kotlin \
      --out-dir generated/