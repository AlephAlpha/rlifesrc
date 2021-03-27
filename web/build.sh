#!/usr/bin/env bash

export DEPLOY_PATH=./dist
export TARGET_PATH=../target/wasm32-unknown-unknown/release
export STATIC_PATH=./static
export RLIFESRC_PATH=worker.js

mkdir -p ${DEPLOY_PATH}
rm -rf ${DEPLOY_PATH}/*

cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --target web --no-typescript --out-dir ${DEPLOY_PATH} ${TARGET_PATH}/main.wasm
wasm-bindgen --target no-modules --no-typescript --out-dir ${DEPLOY_PATH} ${TARGET_PATH}/worker.wasm

cp -r ${STATIC_PATH}/* ${DEPLOY_PATH}
