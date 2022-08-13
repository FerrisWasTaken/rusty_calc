#!/bin/bash

#sudo apt-get install -y build-essential curl file git
#sh -c "$(curl -fsSL https://raw.githubusercontent.com/Linuxbrew/install/master/install.sh)"
#echo 'export PATH="/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin/:$PATH"' >> $BASH_ENV
#echo 'export MANPATH="/home/linuxbrew/.linuxbrew/share/man:$MANPATH"' >> $BASH_ENV
#echo 'export INFOPATH="/home/linuxbrew/.linuxbrew/share/info:$INFOPATH"' >> $BASH_ENV
#brew install gh
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
