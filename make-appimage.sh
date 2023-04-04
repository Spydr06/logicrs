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
if [ -e $APP_DIR ]; then
    rm -rf $APP_DIR
fi

mkdir -p $APP_DIR/usr/{bin,lib,share/icons}
cp $BIN $APP_DIR/usr/bin/logicrs

# Copy binary
sed -i -e 's#/usr#././#g' $APP_DIR/usr/bin/logicrs

# Collect libraries
mkdir -p $APP_DIR/usr/lib

LIB_PATHS="/usr/lib64"
LIBRARIES=($(ldd $BIN | awk '{print $1;}'))
declare -A EXCLUDE_LIBS=(
    [libgcc_s.so.1]=1
    [linux-vdso.so.1]=1
)

for lib in "${LIBRARIES[@]}"; do
    LIB_FILE=($(ldconfig -p | grep $lib | tr ' ' '\n' | grep /))

    # if we encounter a excluded lib or a system lib located in /lib64, continue
    if [[ ${EXCLUDE_LIBS[$lib]} ]] || [[ $LIB_FILE =~ ^/lib64/* ]]; then
        echo " - Excluding $LIB_FILE"
        continue
    fi

    echo " + Including $LIB_FILE"
    cp ${LIB_FILE[0]} "$APP_DIR/usr/lib/$lib"
done

# Copy extra libs required
PIXBUF_LOADERS_DIR="gdk-pixbuf-2.0/2.10.0"
PIXBUF_SVG_LOADER="$PIXBUF_LOADERS_DIR/loaders/libpixbufloader-svg.so"
EXTRA_LIB_DIR='/usr/lib64'
declare -a EXTRA_LIBS=(
    $PIXBUF_SVG_LOADER
    "librsvg-2.so.2"
)

for lib in "${EXTRA_LIBS[@]}"; do
    LIB_FILE="$EXTRA_LIB_DIR/$lib"

    if [ ! -f $LIB_FILE ]; then
        echo "Could not find $LIB_FILE in $EXTRA_LIB_DIR"
        continue
    fi

    mkdir -p "$APP_DIR/usr/lib/$(dirname $lib)"

    echo " + Including $LIB_FILE"
    cp $LIB_FILE "$APP_DIR/usr/lib/$lib"
done

# generate pixbuf loader cache

PIXBUF_LOADERS_CACHE="$PIXBUF_LOADERS_DIR/loaders.cache"
echo " @ Generating $PIXBUF_LOADERS_CACHE"
gdk-pixbuf-query-loaders "$APP_DIR/usr/lib/$PIXBUF_SVG_LOADER" > "$APP_DIR/usr/lib/$PIXBUF_LOADERS_CACHE"

# Patch binaries
pushd $APP_DIR/usr/lib
    find . -type f -exec sed -i -e 's#/usr#././#g' {} \;
popd

#
# Generate and copy icons
#

DIR_ICON=".DirIcon"

SOURCE_ICONS_DIR="style/icons"
RELEASE_ICON_FILE="$SOURCE_ICONS_DIR/hicolor/scalable/apps/com.spydr06.logicrs.svg"
DEBUG_ICON_FILE="$SOURCE_ICONS_DIR/hicolor/scalable/apps/com.spydr06.logicrs.Devel.svg"
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

echo " + Copying icons to $ICONS_DIR"
cp -r $SOURCE_ICONS_DIR $ICONS_DIR/..

#
# Copy theme files
#

ADW_THEME_INDEX="/usr/share/icons/Adwaita/index.theme"
if [ ! -f $ADW_THEME_INDEX ]; then
    echo "Error getting $ADW_THEME_INDEX, there might be missing icons."
fi
cp $ADW_THEME_INDEX $ICONS_DIR/Adwaita

ADW_THEME_CACHE="/usr/share/icons/Adwaita/icon-theme.cache"
if [ ! -f $ADW_ICON_THEM ]; then
    echo "Error getting $ADW_THEME_CACHE, there might be missing icons."
fi
cp $ADW_THEME_CACHE $ICONS_DIR/Adwaita

GLIB_SCHEMA="/usr/share/glib-2.0/schemas/gschemas.compiled"
if [ ! -f $GLIB_SCHEMA ]; then
    echo "Error getting $GLIB_SCHEMA, there might be missing icons."
fi
mkdir -p $APP_DIR/$(dirname $GLIB_SCHEMA)
cp $GLIB_SCHEMA "$APP_DIR/$(dirname $GLIB_SCHEMA)"
echo "<> cp $GLIB_SCHEMA to $APP_DIR/$(dirname $GLIB_SCHEMA)"

#
# Prepare AppDir
#

SNIPPETS_DIR="snippets"

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
