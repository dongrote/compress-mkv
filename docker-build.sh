#!/bin/sh
RESIDENT_DIRECTORY=$(dirname $(realpath "$0"))
cd "$RESIDENT_DIRECTORY"
docker build . -t compress-mkv
