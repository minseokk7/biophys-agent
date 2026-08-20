#!/bin/bash
set -e

# 윈도우에 이미 설치된 Android SDK/NDK를 WSL에서 직접 마운트하여 사용
export ANDROID_HOME=/mnt/c/Users/minse/AppData/Local/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973
export JAVA_HOME=/usr/lib/jvm/default-java
export PATH=$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:/root/.cargo/bin:$PATH
export CI=true

echo "======================================"
echo "🔧 NDK 경로: $NDK_HOME"
echo "🔧 JAVA_HOME: $JAVA_HOME"
echo "======================================"

# Rust 안드로이드 타겟 추가
source /root/.cargo/env
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android 2>/dev/null || true

cd /mnt/c/Users/minse/Documents/antigravity/noble-babbage/biophys-agent

echo "🚀 APK 빌드 시작..."
pnpm tauri android build --apk

echo "✅ APK 빌드 완료!"
