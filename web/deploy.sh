#!/bin/bash

set -e

cd $(dirname "$0")
here=$(pwd)
build=$here/../target/wasm32-unknown-unknown/release/
deploy=$here/../target/deploy/

echo "Here is $here"
echo "Built in $build"
echo "Deployed in $deploy"

echo
echo "Initializing..."
if [ ! -d $deploy/.git/ ]; then
    rm -rf $deploy
    git clone --single-branch --branch=gh-pages --depth 1 git@github.com:AlephAlpha/rlifesrc.git $deploy
fi
cd $deploy
git clean -fdx
git rm -rf .
git reset

echo
echo "Building..."
cd $here
cargo build --release --target wasm32-unknown-unknown --bin main
wasm-bindgen --target web --no-typescript --out-dir $deploy $build/main.wasm
cargo build --release --target wasm32-unknown-unknown --bin worker
wasm-bindgen --target no-modules --no-typescript --out-dir $deploy $build/worker.wasm

echo
echo "Copying files..."
cd $deploy
cp -vr $here/static/* .

while getopts ":d" o; do
    case "${o}" in
    d)
        echo
        echo "Deploying..."
        git add -A
        git commit -m "网页版更新：$(date)"
        git push origin gh-pages
        ;;
    *) ;;
    esac
done
shift $((OPTIND - 1))

echo
echo "Done!"
