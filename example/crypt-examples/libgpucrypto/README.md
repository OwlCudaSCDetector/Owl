# libgpucrypto

<span style="color:red">🔥Special thanks to [lwakefield](https://github.com/lwakefield), who not only went out of his way to find libgpucrypto's source code, but also uploaded it for anyone to use</span>

This project is not actively maintained, but go ahead and open issues and I (or someone else) might get around to looking into it. I forked the [libgpucrypto](https://github.com/lwakefield/libgpucrypto) library and fixed it just enough to build fine. I don't intend on doing much else unless I need to. Originally, the project required g++ 4.4 or older and CUDA Toolkit 4.0. After my patches, it can now build with the latest g++ and CUDA Toolkit but still requires OpenSSL 1.0.2 or older. I might end up needing a newer version of OpenSSL, at which point I will add support for that. Unfortunately, it will take much more work than it did to update the project to the latest g++ and CUDA Toolkit.

If you want to add support for newer versions of OpenSSL, you can open up an issue here and I'll give you a rundown of what needs to be done

I was able to build the project successfully with these specs:\
Linux Mint 20.2\
GTX 1050\
OpenSSL 1.0.2f\
CUDA Toolkit v11.4\
g++ 9.3.0

Requirements:\
Linux (Untested on Windows)\
CUDA Toolkit\
OpenSSL 1.0.2 or lower\
make\
g++ or equivalent

### How to build:
1. Open up `Makefile.in` and modify the following:\
    a. Set `OPENSSL_DIR` to your OpenSSL install location. If you leave it blank the system default will be used.\
    b. Set `CUDA_TOOLKIT_DIR` to your CUDA Toolkit directory (e.g. `/usr/local/cuda-11.4`)\
    c. Set `COMPUTE_CAPABILITY` to the compute capability of your GPU. For example my GTX 1050 supports up to compute capability 6.1, so I put `61`
2. Run `make` to build the project
3. Make sure to link `libgpucrypto` and its dependencies to your project _in the proper order_. For example: `g++ myfile.cpp -L/usr/local/ssl/lib64 -L/path/to/libgpucrypto/lib -L/usr/local/cuda-11.4/lib64 -libgpucrypto -lcrypto -lcudart`. 

Because of a quirk in gcc/g++, dependencies have to be listed in a specific order. Say my program depends on two libraries, `libx` and `liby`. Each of these libraries has their own dependencies: `libx` depends on `liba`, and `liby` depends on `libb`.

The dependency tree would look like this: 
```
-- myprogram.cpp
---- libx
-------- liba
---- liby
-------- libb
```
See how `libx` and `liby` are both on the first level on the tree? And `liba` and `libb` are both on the second level of the tree?\
That's exactly how we'll order our command: `g++ myprogram.cpp -lx -ly -la -lb`\
Notice how all the dependencies on the first level are listed first, then all the dependencies on the second level are listed second and so on.