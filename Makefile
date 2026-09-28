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

so_ios: clean
	cargo build \
		--release \
		--target=aarch64-apple-ios-sim

	cargo build \
		--release \
		--target=aarch64-apple-ios

swift: so_ios
	cargo run --bin uniffi_bindgen generate \
		--library target/aarch64-apple-ios-sim/release/librelaysms_spec_payload.dylib \
		--language swift \
		--out-dir generated/bindings/sim/

	cargo run --bin uniffi_bindgen generate \
		--library target/aarch64-apple-ios/release/librelaysms_spec_payload.dylib \
		--language swift \
		--out-dir generated/bindings

xcode: swift
	xcodebuild -create-xcframework \
		-library target/aarch64-apple-ios-sim/release/librelaysms_spec_payload.a -headers generated/bindings/sim/ \
		-library target/aarch64-apple-ios/release/librelaysms_spec_payload.a -headers generated/bindings \
		-output "generated/ios/RelaySMS_spec_payload.xcframework"