#!/usr/bin/env bash
set -e

echo "-- Building"
./tools/android/build.sh

echo "-- Uploading"
adb push ./build/arm64-v8a/aarch64-linux-android/debug/libhachimi.so /sdcard/libmain.so

echo "-- Installing"
adb shell am force-stop jp.co.cygames.umamusume
adb shell su -c 'bash /sdcard/hachimi.sh'

echo "-- Launching"
adb shell monkey -p jp.co.cygames.umamusume 1

echo "-- Logcat"
adb logcat |& grep --line-buffered Hachimi
