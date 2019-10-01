#!/bin/bash

set -e

here=$(pwd)
cd $(dirname "$0")
echo "Building..."
cargo web build --release

echo "Copying files..."
cp ../target/wasm32-unknown-unknown/release/*.{js,wasm} .deploy_git/
cp -r static/* .deploy_git/

if [ $1 != "-c" ]; then
    echo "Deploying..."
    cd .deploy_git
    git add -A
    git commit -m "网页版更新：$(date)"
    git push origin HEAD:gh-pages
fi

cd $here
echo "Done!"
