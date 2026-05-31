#!/bin/sh -e

ROOTDIR="$PWD"
BUILDDIR="${BUILDDIR:-$ROOTDIR/build}"
ARTIFACTS_DIR="$ROOTDIR/artifacts"
INSTALL="$PWD/install"

SHARUN="https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/refs/heads/main/useful-tools/quick-sharun.sh"

cmake --install "$BUILDDIR" --prefix "${INSTALL}/usr"

# variables to be used on quick-sharun and uruntime2appimage
export ICON="$ROOTDIR/program_info/org.kcraft.KCraft.svg"
export DESKTOP="$BUILDDIR/program_info/org.kcraft.KCraft.desktop"
export OPTIMIZE_LAUNCH=1
export DEPLOY_OPENGL=0
export DEPLOY_VULKAN=0
export ADD_HOOKS=""
export OUTPATH="$ARTIFACTS_DIR"
export OUTNAME="KCraft-Linux-$ARCH-$VERSION.AppImage"
UPINFO="gh-releases-zsync|KCraft|KCraft|latest|KCraft-Linux-${ARCH}-*.AppImage.zsync"

export UPINFO

# deploy
curl -L --retry 30 "$SHARUN" -o quick-sharun
chmod a+x quick-sharun
./quick-sharun "${INSTALL}/usr/bin/kcraft" "${INSTALL}/usr/share/"

# MAKE APPIMAGE WITH URUNTIME
echo "-- Generating AppImage..."
./quick-sharun --make-appimage

echo "Linux package created: $OUTPATH/$OUTNAME"