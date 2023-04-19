#!/bin/bash
rm -rf ~/.todos

executable="/usr/local/bin/todos"
if test -f "$executable"; then
  sudo rm "$executable"
fi