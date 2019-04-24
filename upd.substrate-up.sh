#!/usr/bin/env bash

f=`mktemp -d`
# echo "tempdir: ${f}"; open $f

git clone https://github.com/paritytech/substrate-up $f
ls -a $f/*
cp -a $f/substrate-* ~/.cargo/bin
cp -a $f/polkadot-* ~/.cargo/bin

rm -rf $f
