all:
	@echo "make [clean|so|kotlin]"
clean:
	cargo clean
	rm -rf generated/*

so: clean
	cargo ndk \
	  -t arm64-v8a \
	  -t armeabi-v7a \
	  -t x86_64 \
	  build --release
kotlin: so
	cargo run --bin uniffi_bindgen -- generate \
	  --no-format \
      --library target/aarch64-linux-android/release/librelaysms_spec_payload.so \
      --language kotlin \
      --out-dir generated/