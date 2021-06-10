#!/bin/bash

if [[ $(whoami) == "root" ]]; then
  echo "Warning... you ran this script as root."
  echo "Continuing in 5s. Ctrl-C if you want to stop now!"
  sleep 5
fi

# test for cargo and git

cargo=$(command -v cargo)
git=$(command -v git)

if [ ! -x "$cargo" ] || [ ! -x "$git" ] ; then
  echo "Toolchain not installed"
  exit 1
fi

# get necessary paths
repo_dir=$(pwd)
install_root="$HOME/.todos"
starting_version=$(cat "$install_root/version")

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
task_color = \"0;37\"
hide_backlog_items = true
backlog_name = \"backlog\"\
  " > "$install_root/config.toml"

fi

git pull
# ending_version=$(cat version)

# perform any necessary updates
if [[ -n "$starting_version" && "$(echo "$starting_version < 0.1"| bc)" == "1" ]]; then
  echo "\
hide_backlog_items = true
backlog_name = \"backlog\"\
  " >> "$install_root/config.toml"
fi



cargo build --release
sudo cp "target/release/todos" "/usr/local/bin"
cp "$repo_dir/version" "$install_root"

echo "Enjoy your new or updated todos CLI!"