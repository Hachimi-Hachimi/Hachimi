#!/usr/bin/env bash
set -e

case "$OSTYPE" in
    darwin*)  OS="darwin" ;; 
    linux*)   OS="linux" ;;
    *)
        echo "Unknown OSTYPE: $OSTYPE"
        exit 1
        ;;
esac

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

TOOLCHAIN_DIR="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/$OS-x86_64"
SYSROOT="$TOOLCHAIN_DIR/sysroot"

export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$SYSROOT"
export RUSTFLAGS="-C link-args=-static-libstdc++ -C link-args=-lc++abi"

export CC_aarch64_linux_android="$TOOLCHAIN_DIR/bin/aarch64-linux-android24-clang"
export CXX_aarch64_linux_android="$TOOLCHAIN_DIR/bin/aarch64-linux-android24-clang++"
export AR_aarch64_linux_android="$TOOLCHAIN_DIR/bin/llvm-ar"

export CC_armv7_linux_androideabi="$TOOLCHAIN_DIR/bin/armv7a-linux-androideabi24-clang"
export CXX_armv7_linux_androideabi="$TOOLCHAIN_DIR/bin/armv7a-linux-androideabi24-clang++"
export AR_armv7_linux_androideabi="$TOOLCHAIN_DIR/bin/llvm-ar"

mkdir -p build
cargo build --target=aarch64-linux-android --target-dir=build $CARGOARGS
cargo build --target=armv7-linux-androideabi --target-dir=build $CARGOARGS

pushd build

cp "aarch64-linux-android/$BUILD_TYPE/libhachimi.so" libmain-arm64-v8a.so
cp "armv7-linux-androideabi/$BUILD_TYPE/libhachimi.so" libmain-armeabi-v7a.so

ARM64_V8A_SHA1=($(sha1sum libmain-arm64-v8a.so))
ARMEABI_V7A_SHA1=($(sha1sum libmain-armeabi-v7a.so))

cat << EOF > sha1.json
{
    "libmain-arm64-v8a.so": "$ARM64_V8A_SHA1",
    "libmain-armeabi-v7a.so": "$ARMEABI_V7A_SHA1"
}
EOF

popd
