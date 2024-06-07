# Owl

Owl: Differential-based Side-Channel Leakage Detection for CUDA Applications

## Installation

### Build

```shell
$ git clone https://github.com/OwlCudaSCDetector/Owl.git owl
$ cd owl && make
```

### Docker

```
$ docker build -t owl:1.0 .
$ docker run --gpus all --rm -it owl:1.0 bash
```

## Usage

When complete the **build** phase, we can start test CUDA applications. Use `randaccess` in the `example` directory as a use case.

For quick test, We assume that the **root directory** of the tool is `/root/owl`:
```shell
$ cd example/cuda-examples
$ make
$ make test
```

`src/owl-wrapper` is a wrapper for Intel pins and NVbits. `src/owl-wrapper ${command}` can be used to trace CUDA program execution.
`src/owl_analyzer/target/release/owl_analyzer` is the core analyzer.


## License

Owl has a MIT license, as found in the LICENSE file.