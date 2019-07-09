#!/bin/bash

set -e

cd $(dirname "$0")
cargo web build --release
cp target/wasm32-unknown-unknown/release/*.{js,wasm} .deploy_git/
cp static/* .deploy_git/
cd .deploy_git
git add -A
git commit -m "Update Github Pages"
git push origin HEAD:gh-pages
cd ..
