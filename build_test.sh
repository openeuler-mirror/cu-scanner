#!/bin/bash

echo "Clean the build dir"
cargo clean

echo "Create build tmp dir"
TMP_DIR="${HOME}/tmp/cu-scanner-build"
mkdir -p ${TMP_DIR}/{SOURCES,SPECS}

echo "make source tar"
tar -czvf ${TMP_DIR}/SOURCES/cu-scanner.tar.gz ./*

echo "make spec"
cp cu-scanner.spec ${TMP_DIR}/SPECS

echo "Start build"
pushd ${TMP_DIR} 2>/dev/null 1>/dev/null
rpmbuild -ba --define "_topdir $(pwd)" SPECS/cu-scanner.spec
popd 2>/dev/null 1>/dev/null
