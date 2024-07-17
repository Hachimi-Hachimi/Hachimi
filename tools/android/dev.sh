#!/usr/bin/env bash
set -e

if [ "$RELEASE" = "1" ]; then
    BUILD_TYPE="release"
else
    BUILD_TYPE="debug"
fi

echo "-- Building"
./tools/android/build.sh

echo "-- Uploading"
adb push ./build/aarch64-linux-android/$BUILD_TYPE/libhachimi.so /sdcard/libmain.so

echo "-- Installing"
adb shell am force-stop jp.co.cygames.umamusume
adb shell su < "$(dirname "$0")/hachimi.sh"

echo "-- Launching"
adb shell am start-activity jp.co.cygames.umamusume/jp.co.cygames.umamusume_activity.UmamusumeActivity

echo "-- Logcat"
adb logcat |& grep --line-buffered Hachimi
