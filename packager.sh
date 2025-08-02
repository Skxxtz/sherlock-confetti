#!/bin/bash

read -p "Current version: " version
rm -rf ~/.tmp/confetti-release/
mkdir -p ~/.tmp/confetti-release/
cargo build --release
cp target/release/confetti ~/.tmp/confetti-release/
cp LICENSE ~/.tmp/confetti-release/LICENSE

cd ~/.tmp/confetti-release/
tar -czf confetti-v${version}-bin-linux-x86_64.tar.gz confetti LICENSE
