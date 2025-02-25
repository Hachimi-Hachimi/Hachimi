#!/usr/bin/env bash
set -e

APK_EXTRACT_DIR=/tmp/hachimi-apk-extract
TMP_BASE_APK=/tmp/hachimi-base.apk
TMP_CONFIG_APK=/tmp/hachimi-config.apk
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
    rm -f "$TMP_BASE_APK" "$TMP_CONFIG_APK"
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

echo "-- Extracting config APK"
rm -rf "$APK_EXTRACT_DIR"
unzip "$3" -d "$APK_EXTRACT_DIR"

if [ -d "$APK_ARM64_LIB_DIR" ]; then
    BUILD_DIR=./build/aarch64-linux-android
    APK_LIB_DIR="$APK_ARM64_LIB_DIR"
elif [ -d "$APK_ARM_LIB_DIR" ]; then
    BUILD_DIR=./build/armv7-linux-androideabi
    APK_LIB_DIR="$APK_ARM_LIB_DIR"
else
    echo "-- Failed to detect config architecture!"
    exit 1
fi

echo "-- Detected lib dir: $APK_LIB_DIR"

if [ ! -f "$APK_LIB_DIR/libmain_orig.so" ]; then
    echo "-- Copying libmain_orig.so"
    cp "$APK_LIB_DIR/libmain.so" "$APK_LIB_DIR/libmain_orig.so"
fi

echo "-- Copying Hachimi"
cp "$BUILD_DIR/$BUILD_TYPE/libhachimi.so" "$APK_LIB_DIR/libmain.so"

echo "-- Repacking config APK"
pushd "$APK_EXTRACT_DIR"
zip -r6 "$TMP_CONFIG_APK" .
popd

echo "-- Signing APKs (you'll be asked for a password twice)"
echo "(Password is securep@ssw0rd816-n if you're using UmaPatcher's keystore)"
cp "$2" "$TMP_BASE_APK"
"$APKSIGNER" sign --ks "$1" "$TMP_BASE_APK"
"$APKSIGNER" sign --ks "$1" "$TMP_CONFIG_APK"

echo "-- Installing"
adb shell am force-stop "$PACKAGE_NAME"
adb install-multiple "$TMP_BASE_APK" "$TMP_CONFIG_APK"

clean

echo "-- Launching"
adb shell am start-activity "$PACKAGE_NAME/$ACTIVITY_NAME"

echo "-- Logcat"
adb logcat |& grep --line-buffered Hachimi
