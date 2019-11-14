#!/bin/sh

uri="$1"
dest="$2"

/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --headless --print-to-pdf="$2" "$1"
