#!/bin/bash

set -e

cd $(dirname "$0")
here=$(pwd)
bin=$here/../target/wasm32-unknown-unknown/release/
target=$here/../target/deploy/

echo "Here is $here"
echo "Target folder is $target"

echo
echo "Initializing..."
if [ ! -d $target/.git/ ]; then
    rm -rf $target
    git clone --single-branch --branch=gh-pages --depth 1 git@github.com:AlephAlpha/rlifesrc.git $target
fi
cd $target
git clean -fdx
git rm -rf .

echo
echo "Building..."
cd $here
cargo build --release --target wasm32-unknown-unknown --bin main
wasm-bindgen --target web --no-typescript --out-dir $target $bin/main.wasm
cargo build --release --target wasm32-unknown-unknown --bin worker
wasm-bindgen --target no-modules --no-typescript --out-dir $target $bin/worker.wasm

echo
echo "Copying files..."
cd $target
cp -vr $here/static/* .
git add -A

while getopts ":d" o; do
    case "${o}" in
    d)
        echo
        echo "Deploying..."
        git commit -m "网页版更新：$(date)"
        git push origin gh-pages
        ;;
    *) ;;
    esac
done
shift $((OPTIND - 1))

echo
echo "Done!"
