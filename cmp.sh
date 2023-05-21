#!/usr/bin/env bash

orig=$1
compressed="$orig.cmp"
recovered="recovered-$orig"

cli() {
    ./target/release/cmpr "$@"
}

cargo build --release
cli --version

echo "compressing [$orig] into [$compressed]..."
cli -a lzw --stats compress -o "$compressed" "$orig"

echo "decompressing [$compressed] into [$recovered]..."
cli -a lzw --stats decompress -o "$recovered" "$compressed"

if diff "$orig" "$recovered" &> /dev/null; then
    echo "ok"
else
    echo "files differ"
fi

rm "$compressed"
rm "$recovered"
