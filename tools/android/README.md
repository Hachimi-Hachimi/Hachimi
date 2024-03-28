# Android Dev Tools
These scripts are intended for development on Linux.

- `build.sh`: Helper script to build binaries for both `arm64-v8a` and `armeabi-v7a`
- `dev.sh`: Run the build script, deploy the binaries to the current adb device and run logcat. Device must be rooted. **Push the `hachimi.sh` script to `/sdcard/hachimi.sh` first before running this script. Also, it assumes that you have an arm64 device.**
- `hachimi.sh`: **This is intended to be executed on an Android device.** Helper script to deploy the binaries during development.

`build.sh` and `dev.sh` requires the `ANDROID_NDK_ROOT` environment variable to be set.