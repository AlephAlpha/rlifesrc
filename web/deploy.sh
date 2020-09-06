#!/bin/bash

set -e

cd $(dirname "$0")
here=$(pwd)
target=$here/../target/deploy/

echo "Here is $here"
echo "Target folder is $target"

if [ ! -d $target/.git/ ]; then
    echo
    echo "Initializing..."
    rm -rf $target
    git clone --single-branch --branch=gh-pages --depth 1 git@github.com:AlephAlpha/rlifesrc.git $target
fi

echo
echo "Building..."
cd $here
cargo web build --release

echo
echo "Copying files..."
cd $target
git clean -fdx
git rm -rf .
cp -v ../wasm32-unknown-unknown/release/*.{js,wasm} .
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
