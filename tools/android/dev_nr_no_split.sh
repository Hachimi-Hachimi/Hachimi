#!/usr/bin/env bash
set -e

APK_EXTRACT_DIR=/tmp/hachimi-apk-extract
TMP_BASE_APK=/tmp/hachimi-base.apk
APK_ARM64_LIB_DIR="$APK_EXTRACT_DIR/lib/arm64-v8a"
APK_ARM_LIB_DIR="$APK_EXTRACT_DIR/lib/armeabi-v7a"

if [ -z "$PACKAGE_NAME" ]; then
    PACKAGE_NAME="jp.co.cygames.umamusume"
fi
if [ -z "$ACTIVITY_NAME" ]; then
    ACTIVITY_NAME="jp.co.cygames.umamusume_activity.UmamusumeActivity"
fi

clean() {
    echo "-- Cleaning up"
    rm -rf "$APK_EXTRACT_DIR"
    rm -f "$TMP_BASE_APK"
}

if [ "$1" = "clean" ]; then
    clean
    exit
fi

if [ "$RELEASE" = "1" ]; then
    BUILD_TYPE="release"
else
    BUILD_TYPE="debug"
fi

if [ ! -f "$1" ]; then
    echo "Keystore doesn't exist, byebye!"
    exit 1
fi

if [ ! -f "$2" ]; then
    echo "Base APK doesn't exist, byebye!"
    exit 1
fi

if [ -z "$APKSIGNER" ]; then
    echo "APKSIGNER must be set"
    exit 1
fi

echo "-- Building"
./tools/android/build.sh

clean

echo "-- Extracting APK"
rm -rf "$APK_EXTRACT_DIR"
unzip "$2" -d "$APK_EXTRACT_DIR"

if [ -d "$APK_ARM64_LIB_DIR" ]; then
    if [ ! -f "$APK_ARM64_LIB_DIR/libmain_orig.so" ]; then
        echo "-- [arm64] Copying libmain_orig.so"
        cp "$APK_ARM64_LIB_DIR/libmain.so" "$APK_ARM64_LIB_DIR/libmain_orig.so"
    fi

    echo "-- [arm64] Copying Hachimi"
    cp "./build/aarch64-linux-android/$BUILD_TYPE/libhachimi.so" "$APK_ARM64_LIB_DIR/libmain.so"
fi

if [ -d "$APK_ARM_LIB_DIR" ]; then
    if [ ! -f "$APK_ARM_LIB_DIR/libmain_orig.so" ]; then
        echo "-- [armv7] Copying libmain_orig.so"
        cp "$APK_ARM_LIB_DIR/libmain.so" "$APK_ARM_LIB_DIR/libmain_orig.so"
    fi

    echo "-- [armv7] Copying Hachimi"
    cp "./build/armv7-linux-androideabi/$BUILD_TYPE/libhachimi.so" "$APK_ARM_LIB_DIR/libmain.so"
fi

echo "-- Repacking APK"
pushd "$APK_EXTRACT_DIR"
zip -r6 "$TMP_BASE_APK" .
zip -Z store "$TMP_BASE_APK" resources.arsc
popd

echo "-- Signing APK"
echo "(Password is securep@ssw0rd816-n if you're using UmaPatcher's keystore)"
"$APKSIGNER" sign --ks "$1" "$TMP_BASE_APK"

echo "-- Installing"
adb shell am force-stop "$PACKAGE_NAME"
adb install "$TMP_BASE_APK"

clean

echo "-- Launching"
adb shell am start-activity "$PACKAGE_NAME/$ACTIVITY_NAME"

echo "-- Logcat"
adb logcat |& grep --line-buffered Hachimi
