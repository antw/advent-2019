#!/usr/bin/env bash

set -e

#dirs=($(find . -maxdepth 1 -type d \( ! -name . \) | grep "\d\d-"))
dirs=($(find . -maxdepth 1 -type d \( ! -name . \)))

#cd intcode
#cargo test
#
#cd ..

for dir in "${dirs[@]}"; do
  cd "$dir"

  if [ -f Cargo.toml ]; then
    cargo test
  fi;

  cd ..
done
