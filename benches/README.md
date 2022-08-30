# Benchmarks

We can use the benchmark suite to generate macro benchmark and micro benchmark reports that describe the performance of the dfm_checks library.

The macro benchmark reports we can generate describe the application level performance of dfm_checks in terms of overall runtime. They are useful in assessing how changes in parameters or changes in implementation affect the performance of dfm_checks.

The micro benchmark reports we can generate are flamegraphs that describe which areas of code consume the most CPU time. They are useful in identifying performance hotspots in our code, and they can help guide decisions about where effort should be spent optimizing code.

## Dependencies

### Flamegraph

[Flamegraph](https://github.com/flamegraph-rs/flamegraph) is a Cargo tool that we can use to profile the benchmark suite and generate a flamegraph. Use Cargo to install Flamegraph.

```
cargo install flamegraph
```

### dTrace or perf

Flamegraph uses an OS-specific profiler to collect information about the benchmark suite as it executes. When running on Linux Flamegraph uses the perf profiler. On all other operating systems Flamegraph uses the dTrace profiler.

dTrace should already be installed on macOS, so no additional setup is required.

On Ubuntu (x86) install perf.

```
sudo apt install linux-tools-common linux-tools-generic linux-tools-`uname -r`
```

`linux-tools-` is appended with `` `uname -r` `` to ensure that you install the correct version of perf for your operating system's kernel.

### gnuplot

To generate plots in the macro benchmark reports you need to have [gnuplot](http://www.gnuplot.info/) installed.

On macOS you can install gnuplot using Homebrew.

`brew install gnuplot`

## Running benchmarks

This command runs the benchmark suite and the profiler.

```
cargo flamegraph -o target/bench_checks_flamegraph.svg --bench bench_checks -- --bench
```

Macro benchmark reports are written to `target/criterion/`. To view the report open `target/criterion/report/index.html` in a web browser.

An SVG flamegraph is written to `target/bench_checks_flamegraph.svg`. You can open the SVG in a web browser.

## Troubleshooting

### macOS system integrity protection

When running Flamegraph on macOS you may run into the following error:

```
dtrace: system integrity protection is on, some features will not be available
dtrace: failed to initialize dtrace: DTrace requires additional privileges
```

macOS's "system integrety protection" prevents profiling with dTrace. The security ramifications of this are beyond my understanding, but a simple workaround appears to be to run Flamegraph using `sudo`:

```
sudo cargo flamegraph -o target/bench_checks_flamegraph.svg --bench bench_checks -- --bench
```

### Ubuntu (x86) mangled names

When running Flamegraph on Ubuntu using perf, the function names in the generated SVG flamegraph appear mangled. So far I have not figured out how to generate the SVG flamegraph with unmangled function names.