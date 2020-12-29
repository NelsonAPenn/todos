#!/bin/bash

set -o errexit # if it messes up once, stop immediately!

# get necessary paths
repo_dir=$(pwd)
echo "$repo_dir"

cd ~
install_root="$(pwd)/.todos"
cd "$repo_dir"


if test -f "$install_root"; then
  echo "Converting to new format"
elif test -d "$install_root"; then
  # update
  echo "Updating"
else
  # install fresh

  # create todos directory
  mkdir -p "$install_root" 
  echo "[]" > "$install_root/todos"
 
  # create config.toml
  echo "\
root_directory = \"$install_root\" # don't change this
goal_color = \"1;94\"
condition_color = \"1;33\"
goal_color = \"0;37\"\
  " > "$install_root/config.toml"


fi

git pull
cargo build --release
sudo cp "target/release/todos" "/usr/local/bin"
