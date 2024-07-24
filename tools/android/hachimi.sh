#!/usr/bin/env bash
set -e

if [ -z "$PACKAGE_NAME" ]
then
    echo "No PACKAGE_NAME set, bye-bye!"
    exit 1
fi

APP_DIR=$(dirname $(pm path $PACKAGE_NAME | head -n 1 | cut -c 9-))
LIB_PATH="$APP_DIR/lib/arm64"
LIBMAIN_PATH="$LIB_PATH/libmain.so"
LIBMAIN_ORIG_PATH="$LIB_PATH/libmain_orig.so"

if [ ! -d "$LIB_PATH" ]
then
    echo "Lib directory doesn't exist, bye-bye!"
    exit 1
fi

if [ ! -f "$LIBMAIN_ORIG_PATH" ]
then
    cp "$LIBMAIN_PATH" "$LIBMAIN_ORIG_PATH"
fi

mv /sdcard/libmain.so "$LIBMAIN_PATH"
chmod 0755 "$LIBMAIN_PATH"
chown system:system "$LIBMAIN_PATH"
