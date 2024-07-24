#!/usr/bin/env bash
set -e

if [ "$RELEASE" = "1" ]; then
    BUILD_TYPE="release"
else
    BUILD_TYPE="debug"
fi

case "$1" in
    "" | "JP")
        PACKAGE_NAME="jp.co.cygames.umamusume"
        ACTIVITY_NAME="jp.co.cygames.umamusume_activity.UmamusumeActivity"
        ;;

    "TW_GP")
        PACKAGE_NAME="com.komoe.kmumamusumegp"
        ACTIVITY_NAME="jp.co.cygames.umamusume_activity.UmamusumeActivity"
        ;;

    "TW_MC")
        PACKAGE_NAME="com.komoe.umamusumeofficial"
        ACTIVITY_NAME="jp.co.cygames.umamusume_activity.UmamusumeActivity"
        ;;

    "KR")
        PACKAGE_NAME="com.kakaogames.umamusume"
        ACTIVITY_NAME="kr.co.kakaogames.umamusume_activity.UmamusumeActivity"
        ;;

    *)
        echo "Invalid region specified: $1"
        exit 1
        ;;
esac

echo "-- Building"
./tools/android/build.sh

echo "-- Uploading"
adb push ./build/aarch64-linux-android/$BUILD_TYPE/libhachimi.so /sdcard/libmain.so

echo "-- Installing"
adb shell am force-stop "$PACKAGE_NAME"
adb shell "PACKAGE_NAME=$PACKAGE_NAME" su < "$(dirname "$0")/hachimi.sh"

echo "-- Launching"
adb shell am start-activity "$PACKAGE_NAME/$ACTIVITY_NAME"

echo "-- Logcat"
adb logcat |& grep --line-buffered Hachimi
