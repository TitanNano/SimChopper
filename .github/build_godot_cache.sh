#!/usr/bin/env bash

# set -x

project_dir=$(dirname $(dirname $0))

cd $project_dir

mkdir -p .godot

cache_size=$(du -s .godot/)

godot --editor --headless --verbose &

(while [[ "a" == "a" ]]; do du -s .godot/ && sleep 5; done) &

# initial sleep to let editor startup
sleep 10

new_cache_size=$(du -s .godot/)

while [[ "$cache_size" != "$new_cache_size" ]]; do
  sleep 15
  cache_size="$new_cache_size"
  new_cache_size=$(du -s .godot/)
done

godot_job=$(jobs -p)

kill -s TERM $godot_job
