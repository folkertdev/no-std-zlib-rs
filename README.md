# `no_std` example using `zlib-rs`

Currently configured for the `nrf52840`. For other hardware, change the `memory.x` to the right layout and change the `probe-rs run` command in the `.cargo/config.toml`.

Currently uses a specific commit until a version with `no_std` support is published to `crates.io`.

This is a proof of concept. In particular the "allocator" used in this example is not robust.


## Example Output

```
> cargo run
     Running `probe-rs run --allow-erase-all --always-print-stacktrace --catch-reset --catch-hardfault --chip nRF52840_xxAA target/thumbv7em-none-eabihf/release/no-std-zlib-rs`
      Erasing ✔ [00:00:01] [#########################################################################] 68.00 KiB/68.00 KiB @ 39.56 KiB/s (eta 0s )
  Programming ✔ [00:00:01] [#########################################################################] 68.00 KiB/68.00 KiB @ 61.49 KiB/s (eta 0s )    Finished in 2.838s
<lvl> done with init
└─ no_std_zlib_rs::do_the_thing @ src/main.rs:101
<lvl> deflated [40, 21, 203, 72, 205, 201, 201, 87, 40, 207, 47, 202, 73, 1, 0, 26, 11, 4, 93]
└─ no_std_zlib_rs::do_the_thing @ src/main.rs:118
<lvl> It worked, we got our input back out!
└─ no_std_zlib_rs::do_the_thing @ src/main.rs:168
```

## Notes

This is a pretty hefty dependency. In part that is because this example performs both compression and decompression.

```
> cargo bloat --release --crates
    Finished `release` profile [optimized + debuginfo] target(s) in 0.02s
    Analyzing target/thumbv7em-none-eabihf/release/no-std-zlib-rs

File  .text    Size Crate
4.2%  62.6% 25.5KiB zlib_rs
1.4%  20.7%  8.4KiB std
0.8%  11.7%  4.8KiB no_std_zlib_rs
0.2%   2.6%  1.1KiB [Unknown]
0.1%   1.2%    486B defmt_rtt
0.1%   0.8%    334B defmt
0.0%   0.2%     68B cortex_m_rt
0.0%   0.1%     40B cortex_m
6.7% 100.0% 40.7KiB .text section size, the file size is 607.3KiB
```
