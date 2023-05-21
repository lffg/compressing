Naive implementation of the [LZW] and [Huffman] compression algorithms.

To run, install the [Rust toolchain][rust-toolchain]. Cargo may be used to
compile the source.

Example,

```
$ cargo run

Usage: cmpr [OPTIONS] -a <ALGORITHM> <COMMAND>

Commands:
  compress
  decompress
  help        Print this message or the help of the given subcommand(s)

Options:
  -a <ALGORITHM>      The algorithm to use for compress or decompress [possible values: lzw, huffman]
      --stats         Whether the program should show statistics
  -h, --help          Print help
  -V, --version       Print version
```

Compress a file using the LZW algorithm (assuming `cargo build --release`):

```
$ ./target/release/cmpr -a lzw --stats compress -o Cargo.lock.lzw Cargo.lock
done.
    in 1 ms
    saved 35.10%
```

Decompress the same file:

```
$ ./target/release/cmpr -a lzw --stats decompress -o recovered-Cargo.lock Cargo.lock.lzw
done.
    in 0 ms
```

The script `cmp.sh` may be used to test the compression algorithm by
compressing, decompressing and comparing with the original file. E.g.,

```
$ ./cmp.sh Cargo.lock
    Finished release [optimized] target(s) in 0.06s
cmpr 0.1.0
compressing [Cargo.lock] into [Cargo.lock.cmp]...
done.
    in 1 ms
    saved 35.10%
decompressing [Cargo.lock.cmp] into [recovered-Cargo.lock]...
done.
    in 2 ms
ok
```

[LZW]: https://en.wikipedia.org/wiki/Lempel%E2%80%93Ziv%E2%80%93Welch
[Huffman]: https://en.wikipedia.org/wiki/Huffman_coding
[rust-toolchain]: https://rustup.rs/
