#!/bin/bash

PACKAGE_NAME="chirpstack-concentratord-2g4"
PACKAGE_VERSION=$1
REV="r1"


BIN_PATH="../../../../target/armv5te-unknown-linux-gnueabi/release/chirpstack-concentratord-2g4"
DIR=`dirname $0`
PACKAGE_DIR="${DIR}/package-2g4"

# Cleanup
rm -rf $PACKAGE_DIR

# CONTROL
mkdir -p $PACKAGE_DIR/CONTROL
cat > $PACKAGE_DIR/CONTROL/control << EOF
Package: $PACKAGE_NAME
Version: $PACKAGE_VERSION-$REV
Architecture: arm926ejste
Maintainer: Orne Brocaar <info@brocaar.com>
Priority: optional
Section: network
Source: N/A
Description: ChirpStack Concentratord 2g4
EOF

cat > $PACKAGE_DIR/CONTROL/postinst << EOF
EOF
chmod 755 $PACKAGE_DIR/CONTROL/postinst

cat > $PACKAGE_DIR/CONTROL/conffiles << EOF
EOF

# Files
mkdir -p $PACKAGE_DIR/opt/$PACKAGE_NAME
mkdir -p $PACKAGE_DIR/var/config/$PACKAGE_NAME/examples
mkdir -p $PACKAGE_DIR/etc/init.d

cp files/2g4/$PACKAGE_NAME-ap1.init $PACKAGE_DIR/etc/init.d/$PACKAGE_NAME-ap1
cp files/2g4/$PACKAGE_NAME-ap2.init $PACKAGE_DIR/etc/init.d/$PACKAGE_NAME-ap2
cp files/2g4/*.toml $PACKAGE_DIR/var/config/$PACKAGE_NAME/examples/
cp $BIN_PATH $PACKAGE_DIR/opt/$PACKAGE_NAME/$PACKAGE_NAME

# Package
opkg-build -o root -g root $PACKAGE_DIR

# Cleanup
rm -rf $PACKAGE_DIR
