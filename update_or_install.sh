#!/bin/bash

set -o errexit # if it messes up once, stop immediately!

# test for cargo and git

cargo=$(which cargo)
git=$(which git)

if [ ! -x "$cargo" ] || [ ! -x "$git" ] ; then
  echo "Toolchain not installed"
  exit 1
fi

# get necessary paths
repo_dir=$(pwd)

cd ~
install_root="$(pwd)/.todos"
cd "$repo_dir"


if test -d "$install_root"; then
  echo "Updating"
else
  # install fresh

  # create todos directory
  mkdir -p "$install_root" 
  echo "{\"nodes\":[]}" > "$install_root/todos"
 
  # create config.toml
  echo "\
goal_color = \"1;94\"
condition_color = \"1;33\"
task_color = \"0;37\"\
  " > "$install_root/config.toml"

fi

git pull
cargo build --release
sudo cp "target/release/todos" "/usr/local/bin"
cp "$repo_dir/version" "$install_root"
