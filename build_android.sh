#!/usr/bin/env bash
# Builds an Android APK with cargo-apk.
#
# Requirements:
#   rustup target add armv7-linux-androideabi aarch64-linux-android
#   cargo install cargo-apk --version 0.9.6 --locked
#   ANDROID_SDK_ROOT (platforms;android-31, build-tools) and ANDROID_NDK_ROOT (r25)
set -euo pipefail
cd "$(dirname "$0")"

# Only bundle the assets the game actually loads, to keep the APK small.
rm -rf target/android-assets
mkdir -p target/android-assets/sand_beach
cp -r assets/shaders target/android-assets/
cp assets/sand_beach/vd3lecfs_2K_{Albedo,AO,Normal,Roughness}.jpg target/android-assets/sand_beach/

# Sign with a local keystore (generated on first use) unless one is provided.
if [ -z "${CARGO_APK_RELEASE_KEYSTORE:-}" ]; then
    export CARGO_APK_RELEASE_KEYSTORE="$HOME/.android/kairos-release.keystore"
    export CARGO_APK_RELEASE_KEYSTORE_PASSWORD="android"
    if [ ! -f "$CARGO_APK_RELEASE_KEYSTORE" ]; then
        mkdir -p "$(dirname "$CARGO_APK_RELEASE_KEYSTORE")"
        keytool -genkeypair -v -keystore "$CARGO_APK_RELEASE_KEYSTORE" -alias kairos \
            -keyalg RSA -keysize 2048 -validity 10000 \
            -storepass "$CARGO_APK_RELEASE_KEYSTORE_PASSWORD" -dname "CN=Kairos"
    fi
fi

cargo apk build -p adv_game --lib --release "$@"
echo "APK: target/release/apk/kairos.apk"
