#!/bin/sh -e

mkdir -p dist

find . -type f \( \
    -name "KCraft*.tar.gz" -o \
    -name "KCraft*.exe" -o \
    -name "KCraft*.zip" -o \
    -name "KCraft*.AppImage*" -o \
    -name "kcraft*" \
\) -not -path "./dist/*" -exec cp {} dist/ \;

# mk source tarball
mv KCraft-source KCraft-"${VERSION}"
tar czf "dist/KCraft-$VERSION.tar.gz" "KCraft-${VERSION}"

echo "-- artifacts installed in $PWD/dist"

ls -lh dist