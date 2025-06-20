# Android Dev Tools
These scripts are intended for development on Linux and macOS.

- `build.sh`: Helper script to build binaries for all architectures.
- `build_zygisk.sh`: Helper script to build a Zygisk module for all architectures.
- `dev.sh`: Run the build script, deploy the binaries to the current adb device and run logcat. Device must be arm64 and rooted.
- `hachimi.sh`: **This is intended to be executed on an Android device.** Helper script to deploy the binaries during development. Used by `dev.sh`
- `dev_nr.sh`: Run the build script, patch the app, install it to the current adb device and run logcat. No rooting needed. Needs a keystore file for signing (can be exported from UmaPatcher, convert to JKS or PKCS#12 before use). Usage: `dev_nr.sh <KEYSTORE> <BASE_APK> <ARCH_CONFIG_APK>`
- `dev_nr_no_split.sh`: Same as `dev_nr.sh`, but takes a single APK file.

`build.sh` and `dev.sh` requires the `ANDROID_NDK_ROOT` environment variable to be set.