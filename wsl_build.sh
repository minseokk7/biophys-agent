
#!/bin/bash
set -e
export ANDROID_HOME=/root/android-sdk
export NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973
export PATH=$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:/root/.cargo/bin:$PATH
export CI=true

cd /mnt/c/Users/minse/Documents/antigravity/noble-babbage/biophys-agent
rm -rf node_modules
pnpm install
pnpm tauri android build --apk

