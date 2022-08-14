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
FILE=`realpath homebrew-pck.tar.bz2`
cd ../../
VER=$(sed -ne 's/version\s?*=\s?*\"\(.*\)\"/\1/p' ./Cargo.toml)
gh release create $VER \
$FILE \
--generate-notes
git config --global user.email "$GH_EMAIL"
git config --global user.name "$GH_USR"
git clone git@github.com:muppi090909/homebrew-core.git
cd homebrew-core/Formula
sed -ie "s/\(\t*\)version\(.*\)/\1version \"$VER\"/" rusty_calc.rb
sed -ie \
"s/\(\t*\)url\s.*/\1url \"https:\/\/github.com\/muppi090909\/rusty_calc\/releases\/download\/$VER\/homebrew-pck.tar.bz2\"/" rusty_calc.rb
sed -ie \
"s//"
sed -ie \
 "s/\(\t*\)sha256 .*/\1sha256 \"`shasum -a 256 $FILE | awk '{ print $1 }'`\"/" rusty_calc.rb
cd ..
git add --all
git commit -am "Updated"
git push origin main
