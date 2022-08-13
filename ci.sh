#!/bin/bash

sudo apt install -y build-essential curl file git
sh -c "$(curl -fsSL https://raw.githubusercontent.com/Linuxbrew/install/master/install.sh)"
echo 'export PATH="/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin/:$PATH"' >> $BASH_ENV
echo 'export MANPATH="/home/linuxbrew/.linuxbrew/share/man:$MANPATH"' >> $BASH_ENV
echo 'export INFOPATH="/home/linuxbrew/.linuxbrew/share/info:$INFOPATH"' >> $BASH_ENV
brew install gh
cargo build --release
cd target/release
tar cjf homebrew-pck.tar.bz2 rc_bin
gh release create latest homebrew-pck.tar.bz2
