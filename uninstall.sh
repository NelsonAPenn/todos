#!/bin/bash
rm -rf ~/.todos

cargo=$(command -v cargo)

if [ ! -x "$cargo" ] ; then
  echo "Toolchain not installed"
  exit 1
fi

cargo uninstall todos