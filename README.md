# testing

Testing gccrs in the wild

## Requirements

### ftf >= 0.3

`cargo install --git https://github.com/cohenarthur/ftf`

### submodules

Either clone the repository using the `--recursive` flag, or initialize the submodules afterwards
using the following command:

`git submodule update --init`

## Generate the parsing test-suite

The test-suite adaptor is located in `rustc-testsuite-adaptor`. It is a simple program in charge of generating a sensible test-suite for gccrs from rustc's test-suite. For now, the only generation available is the generation of a parsing test-suite: The application launches `rustc` with the `-Z parse-only` flag and keeps track of the exit code, in order to make sure that gccrs with the `-fsyntax-only` flag has the same behavior.

Running the adaptor is time consuming: It takes roughly 5 minutes to generate the parsing test-suite on my machine.

Currently, the program simply launches the `rustc` installed on your system, which is an issue.

It also absolutely hammers your computer by launching $(nproc) instances of rustc to create the test-suite baseline.

You can run the application either in debug or release mode: As it is extremely IO-intensive, it does not benefit a lot from the extra optimizations (for now!).

You can generate a testsuite using the following arguments:

### --compiler,-c

The compiler executable you want to test. The command will be launched similarly to how an executable is launched from the shell, so you can either specify a relative path to an executable, an absolute path, or simply the name of an executable present in your path.

One way to do this is to copy a freshly built `rust1` from your `gccrs` local copy, and pass `--compiler './rust1'` as an argument. If you've copied your whole build directory, the argument would look something like `--compiler './build/gcc/rust1`.

If you have `gccrs` installed on your system, you can also simply pass `--compiler gccrs`. Be careful in that running the testsuite with a full compiler driver will obviously be much longer.

### --rustc,-r

Path to the cloned rustc repository to extract test cases from.

### --output-dir,-o

Directory to create and in which to store the adapted test cases. The directory will be created by the application.

### --yaml,-y

Path of the `ftf` test-suite file to create.

## Running the test-suite

If everything went smoothly, you should simply be able to run `ftf` on the generated YAML file:

`ftf -f <generated_yaml>`

## Typical first invocation

```sh
> cargo run --manifest-path rustc-testsuite-adaptor/Cargo.toml -- \
	--compiler './rust1' --output-dir sources/ --rustc rust --yaml testsuite.yml
> # the --manifest-path argument is to run the adaptor from the root of this repository
> ftf -f testsuite.yml -j$(nproc)
```

Running `ftf` on a single thread (default behavior if you do not pass a `-j/--jobs` argument) is not recommended as running through the whole parsing test-suite will easily take tens of minutes.

## Re-running the test-suite with an updated compiler

If you have already generated a test-suite and would simply like to run an updated version of `gccrs`, you can reuse the same YAML file.

```sh
> cp <your-gccrs-build-dir>/gcc/rust1 ./rust1
> ftf -f testsuite.yml -j$(nproc)
```
