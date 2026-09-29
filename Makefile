LIBRARY_NAME = librelaysms_spec_payload
XCFRAMEWORK_NAME = relaySMS_spec_payload

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
	cargo build --release --target=aarch64-apple-ios-sim
	cargo build --release --target=aarch64-apple-ios

swift: so_ios
	# Ensure directory structure exists
	mkdir -p generated/bindings/headers

	# Generate Swift bindings and C header files
	cargo run --bin uniffi_bindgen generate \
	   --library target/aarch64-apple-ios/release/$(LIBRARY_NAME).dylib \
	   --language swift \
	   --out-dir generated/bindings

	# Move FFI header to headers directory
	cp generated/bindings/*FFI.h generated/bindings/headers/

	# Create a modulemap file for Xcode
	echo "module relaysms_spec_payloadFFI {" > generated/bindings/headers/module.modulemap
	echo "    header \"relaysms_spec_payloadFFI.h\"" >> generated/bindings/headers/module.modulemap
	echo "    export *" >> generated/bindings/headers/module.modulemap
	echo "}" >> generated/bindings/headers/module.modulemap

xcode: swift
	rm -rf generated/ios/$(XCFRAMEWORK_NAME).xcframework
	xcodebuild -create-xcframework \
	   -library target/aarch64-apple-ios-sim/release/$(LIBRARY_NAME).a -headers generated/bindings/headers \
	   -library target/aarch64-apple-ios/release/$(LIBRARY_NAME).a -headers generated/bindings/headers \
	   -output "generated/ios/$(XCFRAMEWORK_NAME).xcframework"