# Installation

### Using inkscope fuzzer with Docker

Inkscope fuzzer can also be used entirely within a Docker container. If you don’t have it, Docker can be installed directly from [Docker’s website](https://docs.docker.com/get-docker/).

You have to build the docker image locally. From the [inkscope-fuzzer repository](https://github.com/inkscopexyz/inkscope-fuzzer), execute:

```bash
    docker build -t inkscope-fuzzer -f ./.docker/inkscope-fuzzer/Dockerfile .
```

After that, you are all set to use the fuzzer. To run it:

```bash
    docker run -v ".:/contract" inkscope-fuzzer file.contract fuzz
```

Optionally, you can add an alias to make it easier to run the fuzzer.
```bash
    alias inkscope-fuzzer-docker="docker run -v ".:/contract" inkscope-fuzzer"
    inkscope-fuzzer-docker file.contract fuzz
```

### Building from source

#### Pre-requisites

You will need the [Rust](https://www.rust-lang.org/) compiler and Cargo, the Rust package manager. The easiest way to install both is with [rustup.rs](https://rustup.rs/).

#### Building

You can install it directly from crates.io

```bash
    cargo install inkscope-fuzzer
```

> The inkscope-fuzzer executable will be ready to use in your system.  
`cargo inkscope-fuzzer`

Or, by manually building from a local copy of the [inkscope-fuzzer repository](https://github.com/inkscopexyz/inkscope-fuzzer):

```bash
    # Clone the repository
    git clone https://github.com/inkscopexyz/inkscope-fuzzer.git
    cd inkscope-fuzzer

    # Build the project
    cargo build --release
```

> The inkscope-fuzzer executable will be ready to use in the following path:  
`./target/release/inkscope-fuzzer`