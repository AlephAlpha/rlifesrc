#!/bin/bash

set -e

here=$(pwd)
cd $(dirname "$0")
echo "Building..."
cargo web build --release

echo "Copying files..."
cp ../target/wasm32-unknown-unknown/release/*.{js,wasm} .deploy/
cp -r static/* .deploy/

while getopts ":d" o; do
    case "${o}" in
    d)
        echo "Deploying..."
        cd .deploy
        git add -A
        git commit -m "网页版更新：$(date)"
        git push origin HEAD:gh-pages
        ;;
    *) ;;
    esac
done
shift $((OPTIND - 1))

cd $here
echo "Done!"
