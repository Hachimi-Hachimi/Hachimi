#!/usr/bin/env bash
set -e

SONAME=hachimi
MODID=hachimi
MODNAME=Hachimi
AUTHOR=LeadRDRK
DESC="はちみーをなめると〜"
UPDATEJSON=

get_toml_value() {
    # Takes three parameters:
    # - TOML file path ($1)
    # - section ($2)
    # - the key ($3)
    # 
    # It first gets the section using the get_section function
    # Then it finds the key within that section
    # using grep and cut.

    local file="$1"
    local section="$2"
    local key="$3"

    get_section() {
        # Function to get the section from a TOML file
        # Takes two parameters:
        # - TOML file path ($1)
        # - section name ($2)
        # 
        # It uses sed to find the section
        # A section is terminated by a line with [ in pos 0 or the end of file.

        local file="$1"
        local section="$2"

        sed -n "/^\[$section\]/,/^\[/p" "$file" | sed '$d'
    }
        
    get_section "$file" "$section" | grep "^$key " | cut -d "=" -f2- | tr -d ' "'
}

version_to_code() {
    local version=$1
    IFS='.' read -r major minor patch <<< "$version"

    major=$((10#$major))
    minor=$((10#$minor))
    patch=$((10#$patch))

    echo $((major * 10000 + minor * 100 + patch))
}

VERSION="$(get_toml_value Cargo.toml package version)"
GIT_COMMIT="$(git rev-parse --short HEAD)"
VERSION_STR="v$VERSION-$GIT_COMMIT"
VERSION_CODE="$(version_to_code "$VERSION")"
if [[ -n "$(git status --porcelain)" ]]
then
    VERSION_STR="$VERSION_STR-dirty"
fi

echo "*** Zygisk module: $MODNAME ($MODID)"
echo "*** Version: $VERSION_STR"
echo

echo "-- Building"
source ./tools/android/build.sh

echo "-- Generating module"

ZYGISK_BUILD_DIR="/tmp/zygisk-build"
clean() {
    rm -rf "$ZYGISK_BUILD_DIR"
}
copy_lib() {
    local rust_lib_arch="$1"
    local mod_lib_arch="$2"

    mkdir -p "$ZYGISK_BUILD_DIR/lib/$mod_lib_arch"
    cp -v "build/$rust_lib_arch/$BUILD_TYPE/lib$SONAME.so" "$ZYGISK_BUILD_DIR/lib/$mod_lib_arch/lib$SONAME.so"
}

clean

cp -r -v ./tools/android/zygisk-template "$ZYGISK_BUILD_DIR"
copy_lib aarch64-linux-android arm64-v8a
copy_lib armv7-linux-androideabi armeabi-v7a
copy_lib i686-linux-android x86
copy_lib x86_64-linux-android x86_64

cat << EOF > "$ZYGISK_BUILD_DIR/module.prop"
id=$MODID
name=$MODNAME
version=$VERSION_STR
versionCode=$VERSION_CODE
author=$AUTHOR
description=$DESC
EOF

if [[ -n "$UPDATEJSON" ]]
then
    echo "updateJson=$UPDATEJSON" > "$ZYGISK_BUILD_DIR/module.prop"
fi

generate_sha256() {
    local file="$1"
    local hash_file="$file.sha256"
    local hash=($(sha256sum "$file"))

    echo "$hash" > "$hash_file"
    echo "$hash" "$file"
}

for f in $(find "$ZYGISK_BUILD_DIR" -type f)
do
    generate_sha256 "$f"
done

echo "-- Zipping"

ZIP_FILENAME="zygisk-$MODID-$VERSION_STR-$BUILD_TYPE.zip"
ZIP_FILE="$(realpath build)/$ZIP_FILENAME"

pushd "$ZYGISK_BUILD_DIR"
zip -FSr6 "$ZIP_FILE" .
popd

echo "-- Module built: $ZIP_FILENAME"