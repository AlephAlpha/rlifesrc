#!/bin/bash

set -e

cd $(dirname "$0")
cargo web deploy --release
cp ./target/deploy/* .deploy_git/
cp ./target/wasm32-unknown-unknown/release/worker.js .deploy_git/
cp ./target/wasm32-unknown-unknown/release/worker.wasm .deploy_git/
cd .deploy_git
git add -A
git commit -m "Update Github Pages"
git push origin HEAD:gh-pages
cd ..
