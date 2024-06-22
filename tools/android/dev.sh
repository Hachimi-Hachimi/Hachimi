#!/usr/bin/env bash
set -e

echo "-- Building"
./tools/android/build.sh

echo "-- Uploading"
adb push ./build/arm64-v8a/aarch64-linux-android/debug/libhachimi.so /sdcard/libmain.so

echo "-- Installing"
adb shell am force-stop jp.co.cygames.umamusume
adb shell su < "$(dirname "$0")/hachimi.sh"

echo "-- Launching"
adb shell am start-activity jp.co.cygames.umamusume/jp.co.cygames.umamusume_activity.UmamusumeActivity

echo "-- Logcat"
adb logcat |& grep --line-buffered Hachimi
