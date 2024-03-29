#!/usr/bin/env bash
set -e

if [[ -z "$ANDROID_NDK_ROOT" ]]; then
    echo "ANDROID_NDK_ROOT must be set"
    exit 1
fi

if [ "$RELEASE" = "1" ]; then
    CARGOARGS="$CARGOARGS --release"
    BUILD_TYPE="release"
else
    BUILD_TYPE="debug"
fi

TOOLCHAIN_DIR="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64"
SYSROOT="$TOOLCHAIN_DIR/sysroot"

export AR="$TOOLCHAIN_DIR/bin/llvm-ar"
export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$SYSROOT"
export RUSTFLAGS="-C link-args=-static-libstdc++ -C link-args=-lc++abi"

mkdir -p build

export CC="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang"
export CXX="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang++"
cargo build --target=aarch64-linux-android --target-dir=build/arm64-v8a $CARGOARGS

export CC="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi24-clang"
export CXX="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi24-clang++"
cargo build --target=armv7-linux-androideabi --target-dir=build/armeabi-v7a $CARGOARGS

pushd build

cp "arm64-v8a/aarch64-linux-android/$BUILD_TYPE/libhachimi.so" libmain-arm64-v8a.so
cp "armeabi-v7a/armv7-linux-androideabi/$BUILD_TYPE/libhachimi.so" libmain-armeabi-v7a.so

ARM64_V8A_SHA1=($(sha1sum libmain-arm64-v8a.so))
ARMEABI_V7A_SHA1=($(sha1sum libmain-armeabi-v7a.so))

cat << EOF > sha1.json
{
    "libmain-arm64-v8a.so": "$ARM64_V8A_SHA1",
    "libmain-armeabi-v7a.so": "$ARMEABI_V7A_SHA1"
}
EOF

popd
