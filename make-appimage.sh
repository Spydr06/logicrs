#!/usr/bin/env bash

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

pushd $SCRIPT_DIR

#
# Build Project
#

BUILD_DIR="target"
BIN="$BUILD_DIR/release/logicrs"

cargo build --release

if [ ! -f $BIN ]; then
    echo "Release build was not successful."
    exit 1
fi

#
# Create AppDir
#

APP_DIR="$BUILD_DIR/appimage/logicrs.AppDir"
if [ -d $APP_DIR ]; then
    rm -rf $APP_DIR
fi

mkdir -p $APP_DIR/usr/{bin,lib,share/icons}
cp $BIN $APP_DIR/usr/bin/logicrs

# Copy binary
sed -i -e 's#/usr#././#g' $APP_DIR/usr/bin/logicrs

# Collect libraries
mkdir -p $APP_DIR/usr/lib

LIB_PATH="/usr/lib64"
IFS=' ' read -ra LIBRARIES <<< $(pkg-config --libs gtk4 libadwaita-1)
for i in "${LIBRARIES[@]}"; do
    LIB_FILE="lib${i:2}.so"
    if [ ! -f $LIB_PATH/$LIB_FILE ]; then
        echo "could not find \`$LIB_FILE\` in \`$LIB_PATH\`"
    fi
    cp $LIB_PATH/$LIB_FILE $APP_DIR/usr/lib/$LIB_FILE
done

# Patch binaries
pushd $APP_DIR/usr/lib
    find . -type f -exec sed -i -e 's#/usr#././#g' {} \;
popd

#
# Generate Icons
#

DIR_ICON=".DirIcon"

RELEASE_ICON_FILE="style/icons/hicolor/com.spydr06.logicrs.svg"
DEBUG_ICON_FILE="style/icons/hicolor/com.spydr06.logicrs.Devel.svg"
ICON_FILE=$DEBUG_ICON_FILE

inkscape -z -w 256 -h 256 "$ICON_FILE" -o $APP_DIR/$DIR_ICON.png
if [ ! -f $APP_DIR/$DIR_ICON.png ]; then
    echo "Icon generation was not successful."
    exit 1
fi
mv $APP_DIR/$DIR_ICON.png $APP_DIR/$DIR_ICON

cp $ICON_FILE $APP_DIR/logicrs.svg

ICONS_DIR=$APP_DIR/usr/share/icons
mkdir -p $ICONS_DIR
cp $ICON_FILE $ICONS_DIR/logicrs.svg

SNIPPETS_DIR="snippets"

#
# Prepare AppDir
#

# Copy desktop file
DESKTOP_FILE="com.spydr06.logicrs.desktop"
cp $SNIPPETS_DIR/$DESKTOP_FILE $APP_DIR/$DESKTOP_FILE

# Create build directory
BUILD_DIR="target/appimage"
mkdir -p $BUILD_DIR

#
# Generate AppImage
#

RECIPE_FILE="$SNIPPETS_DIR/AppImageBuilder.yml"
appimage-builder --recipe $RECIPE_FILE --appdir $APP_DIR --build-dir $BUILD_DIR

popd
