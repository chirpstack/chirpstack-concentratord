#!/bin/bash

PACKAGE_NAME="chirpstack-concentratord"
PACKAGE_VERSION=$1
REV="r1"


BIN_PATH="../../../../target/arm-unknown-linux-gnueabihf/release/chirpstack-concentratord-sx1301"
DIR=`dirname $0`
PACKAGE_DIR="${DIR}/package"

# Cleanup
rm -rf $PACKAGE_DIR

# CONTROL
mkdir -p $PACKAGE_DIR/CONTROL
cat > $PACKAGE_DIR/CONTROL/control << EOF
Package: $PACKAGE_NAME
Version: $PACKAGE_VERSION-$REV
Architecture: klk_wifc
Maintainer: Orne Brocaar <info@brocaar.com>
Priority: optional
Section: network
Source: N/A
Description: ChirpStack Concentratord
EOF

cat > $PACKAGE_DIR/CONTROL/postinst << EOF
EOF
chmod 755 $PACKAGE_DIR/CONTROL/postinst

cat > $PACKAGE_DIR/CONTROL/conffiles << EOF
EOF

# Files
mkdir -p $PACKAGE_DIR/opt/$PACKAGE_NAME
mkdir -p $PACKAGE_DIR/user/etc/$PACKAGE_NAME/examples
mkdir -p $PACKAGE_DIR/etc/monit.d
mkdir -p $PACKAGE_DIR/etc/init.d

cp files/$PACKAGE_NAME.init $PACKAGE_DIR/etc/init.d/$PACKAGE_NAME
cp files/$PACKAGE_NAME.monit $PACKAGE_DIR/etc/monit.d/$PACKAGE_NAME
cp files/*.toml $PACKAGE_DIR/user/etc/$PACKAGE_NAME/examples/
cp $BIN_PATH $PACKAGE_DIR/opt/$PACKAGE_NAME/$PACKAGE_NAME

# Package
opkg-build -o root -g root $PACKAGE_DIR

# Cleanup
rm -rf $PACKAGE_DIR
