# testing

Testing gccrs in the wild

## Requirements

### [ftf](https://github.com/CohenArthur/ftf) >= 0.4

`cargo install ftf`

### submodules

Either clone the repository using the `--recursive` flag, or initialize the submodules afterwards
using the following command:

`git submodule update --init`

## Generate the parsing test-suite

The test-suite adaptor is a simple program in charge of generating a sensible test-suite for gccrs from rustc's test-suite, as well as making sure our testsuite is valid rust code. For now, there are two "passes" available: 

1. A parsing test-suite, where the application launches `rustc` with the `-Z parse-only` flag and keeps track of the exit code, in order to make sure that gccrs with the `-fsyntax-only` flag has the same behavior.

2. A validation test-suite, where we make sure that rustc can compile gccrs' dejagnu test-suite. This helps in ensuring that our tests are proper rust code.

Running the adaptor is time consuming: It takes roughly 5 minutes to generate the parsing test-suite on a powerful machine.

It also absolutely hammers your computer by launching `$(nproc)` instances of rustc to create the test-suite baseline.

You can run the application either in debug or release mode: As it is extremely IO-intensive, it does not benefit a lot from the extra optimizations (for now!).

You can generate a testsuite using the following arguments:

### --gccrs,-g

The gccrs executable you want to test. The command will be launched similarly to how an executable is launched from the shell, so you can either specify a relative path to an executable, an absolute path, or simply the name of an executable present in your path.

One way to do this is to copy a freshly built `rust1` from your `gccrs` local copy, and pass `--gccrs './rust1'` as an argument. If you've copied your whole build directory, the argument would look something like `--gccrs './build/gcc/rust1`.

If you have `gccrs` installed on your system, you can also simply pass `--gccrs gccrs`. Be careful in that running the testsuite with a full compiler driver will obviously be much longer.

### --rustc,-r

`rustc` executable to use and test. Similar rules apply.

### --rust_path

Path to the cloned rustc repository to extract test cases from.

### --gccrs_path

Path to the cloned gccrs repository to extract test cases from.

### --output-dir,-o

Directory to create and in which to store the adapted test cases. The directory will be created by the application.

### --yaml,-y

Path of the `ftf` test-suite file to create.

### --pass,-p

Pass to run and generate a test suite from. The currently available passes are

|Pass|Description|
|---|---|
|gccrs-parsing|Tests `gccrs`'s parser. This allows testing `gccrs` against `rustc` in parsing-mode (`-Z parse-only` and `-fsyntax-only`)|
|rustc-dejagnu|Launch `rustc` against our dejagnu testsuite. This allows validating `gccrs`'s testsuite, making sure that tests are proper rust code|
|gccrs-rustc-success|Launch `gccrs` against all successful testcases in the `rustc` testsuite|
|gccrs-rustc-success-no-std|Launch `gccrs` against all successful testcases in the `rustc` testsuite in `#[no_std]` mode|
|gccrs-rustc-success-no-core|Launch `gccrs` against all successful testcases in the `rustc` testsuite in `#[no_core]` mode|
|blake3|Launch `gccrs` on the Blake3 cryptography project|
|libcore|Launch `gccrs` on various version of the core library|
|ast-export| Make sure `gccrs` exports valid Rust code|

## Running the test-suite

If everything went smoothly, you should simply be able to run `ftf` on the generated YAML file:

`ftf -f <generated_yaml>`

## Typical first invocation

```sh
> cargo run -- \
	--gccrs './rust1' --rustc rustc \
	--gccrs-path gccrs/ --rust_path rust/ \
	--output-dir sources/ --yaml testsuite.yml \
	--pass gccrs-parsing
> ftf -f testsuite.yml -j$(nproc)
```

Running `ftf` on a single thread (default behavior if you do not pass a `-j/--jobs` argument) is not recommended as running through the whole parsing test-suite will easily take tens of minutes.

## Re-running the test-suite with an updated compiler

If you have already generated a test-suite and would simply like to run an updated version of `gccrs`, you can reuse the same YAML file.

```sh
> cp <your-gccrs-build-dir>/gcc/rust1 ./rust1
> ftf -f testsuite.yml -j$(nproc)
```
