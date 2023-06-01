#!/usr/bin/env bash
#
set -e

REV="r1"

PACKAGE_NAME="chirpstack-concentratord"
PACKAGE_VERSION=`cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "chirpstack-concentratord-sx1301").version'`
PACKAGE_DESCRIPTION=`cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "chirpstack-concentratord-sx1301").description'`
BIN_PATH="../../../../target/armv7-unknown-linux-musleabihf/release/chirpstack-concentratord-sx1301"
DIR=`dirname $0`
PACKAGE_DIR="${DIR}/package"

# Cleanup
rm -rf $PACKAGE_DIR

# CONTROL
mkdir -p $PACKAGE_DIR/CONTROL
cat > $PACKAGE_DIR/CONTROL/control << EOF
Package: $PACKAGE_NAME
Version: $PACKAGE_VERSION-$REV
Architecture: klkgw
Maintainer: Orne Brocaar <info@brocaar.com>
Priority: optional
Section: network
Source: N/A
Description: $PACKAGE_DESCRIPTION
EOF

cat > $PACKAGE_DIR/CONTROL/postinst << EOF
#!/bin/sh
/usr/bin/monit reload
EOF
chmod 755 $PACKAGE_DIR/CONTROL/postinst

cat > $PACKAGE_DIR/CONTROL/conffiles << EOF
/etc/$PACKAGE_NAME/concentratord.toml
/etc/$PACKAGE_NAME/channels.toml
EOF

# Files
mkdir -p $PACKAGE_DIR/usr/bin
mkdir -p $PACKAGE_DIR/etc/$PACKAGE_NAME/examples
mkdir -p $PACKAGE_DIR/etc/monit.d
mkdir -p $PACKAGE_DIR/etc/init.d

cp files/$PACKAGE_NAME.init $PACKAGE_DIR/etc/init.d/$PACKAGE_NAME
cp files/$PACKAGE_NAME.monit $PACKAGE_DIR/etc/monit.d/$PACKAGE_NAME
cp files/concentratord.toml $PACKAGE_DIR/etc/$PACKAGE_NAME
cp files/channels.toml $PACKAGE_DIR/etc/$PACKAGE_NAME
cp ../../../../chirpstack-concentratord-sx1301/config/channels_*.toml $PACKAGE_DIR/etc/$PACKAGE_NAME/examples

cp $BIN_PATH $PACKAGE_DIR/usr/bin/$PACKAGE_NAME

# Package
opkg-build -o root -g root $PACKAGE_DIR

# Cleanup
rm -rf $PACKAGE_DIR
