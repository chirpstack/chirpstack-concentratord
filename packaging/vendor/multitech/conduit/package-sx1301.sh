#!/bin/bash

set -e

REV="r1"

PACKAGE_NAME="chirpstack-concentratord-sx1301"
PACKAGE_VERSION=`cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "chirpstack-concentratord-sx1301").version'`
PACKAGE_DESCRIPTION=`cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "chirpstack-concentratord-sx1301").description'`
BIN_PATH="../../../../target/armv5te-unknown-linux-gnueabi/release/${PACKAGE_NAME}"
DIR=`dirname $0`
PACKAGE_DIR="${DIR}/package-sx1301"

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
Description: $PACKAGE_DESCRIPTION
EOF

cat > $PACKAGE_DIR/CONTROL/postinst << EOF
sed -i "s/ENABLED=.*/ENABLED=\"yes\"/" /etc/default/monit
update-rc.d monit defaults
/etc/init.d/monit start
/usr/bin/monit reload
EOF
chmod 755 $PACKAGE_DIR/CONTROL/postinst

cat > $PACKAGE_DIR/CONTROL/prerm << EOF
/etc/init.d/$PACKAGE_NAME-ap1 stop
/etc/init.d/$PACKAGE_NAME-ap2 stop
EOF
chmod 755 $PACKAGE_DIR/CONTROL/prerm

# This is empty, because the config files are copied on start.
cat > $PACKAGE_DIR/CONTROL/conffiles << EOF
EOF

# Files
mkdir -p $PACKAGE_DIR/opt/$PACKAGE_NAME
mkdir -p $PACKAGE_DIR/var/config/$PACKAGE_NAME/examples
mkdir -p $PACKAGE_DIR/etc/monit.d
mkdir -p $PACKAGE_DIR/etc/init.d

cp files/sx1301/$PACKAGE_NAME-ap1.init $PACKAGE_DIR/etc/init.d/$PACKAGE_NAME-ap1
cp files/sx1301/$PACKAGE_NAME-ap2.init $PACKAGE_DIR/etc/init.d/$PACKAGE_NAME-ap2

cp files/sx1301/$PACKAGE_NAME-ap1.monit $PACKAGE_DIR/var/config/$PACKAGE_NAME/examples
cp files/sx1301/$PACKAGE_NAME-ap2.monit $PACKAGE_DIR/var/config/$PACKAGE_NAME/examples

cp files/sx1301/*.toml $PACKAGE_DIR/var/config/$PACKAGE_NAME/examples
cp $BIN_PATH $PACKAGE_DIR/opt/$PACKAGE_NAME/$PACKAGE_NAME

# Package
opkg-build -o root -g root $PACKAGE_DIR

# Cleanup
rm -rf $PACKAGE_DIR
