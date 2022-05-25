#!/bin/bash

project_dir=$(dirname "$0")
curdir=$(pwd)

cd "$project_dir"
cargo build --release
cp "$project_dir/target/release/goose" "$project_dir"
cd "$curdir"

