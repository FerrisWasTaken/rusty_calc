#!/bin/bash

sudo apt-get install -y build-essential curl file git
sh -c "$(curl -fsSL https://raw.githubusercontent.com/Linuxbrew/install/master/install.sh)"
export PATH="/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin/:$PATH"
brew install gh
cargo build --release
if [ $? != 0 ]; then
    exit $?
fi
cd target/release
tar -cjf homebrew-pck.tar.bz2 rc_bin
cd ../../
VER=$(sed -ne 's/version\s?*=\s?*\"\(.*\)\"/\1/p' ./Cargo.toml)
gh release create $VER \
./target/release/homebrew-pck.tar.bz2 \
--generate-notes
git config --global user.email "wthownothing@gmail.com"
git config --global user.name "muppi090909"
git clone git@github.com:muppi090909/homebrew-core.git
cd homebrew-core
touch hello
git add --all
git commit -am "Updated"
git push origin main
