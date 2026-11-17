# Cross-compiles the Dune Client launcher for Windows x64 from Linux (MinGW, no Windows SDK).
# Build:  docker build -f docker/windows-gnu.Dockerfile -t dune-launcher-win .
# Run:    scripts/build-windows.sh   (uses `docker run --cpus=2`)
FROM debian:trixie-slim
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl build-essential pkg-config cmake nasm \
      gcc-mingw-w64-x86-64-posix g++-mingw-w64-x86-64-posix binutils-mingw-w64-x86-64 \
      zip python3 git wine64 \
 && rm -rf /var/lib/apt/lists/* \
 && update-alternatives --set x86_64-w64-mingw32-gcc /usr/bin/x86_64-w64-mingw32-gcc-posix \
 && update-alternatives --set x86_64-w64-mingw32-g++ /usr/bin/x86_64-w64-mingw32-g++-posix
# fxc stand-in: Microsoft's fxc.exe is not redistributable and the shader step of gpui-fast-windows only
# runs on a Windows host, so third_party/gpui-fast-windows runs GPUI_FXC_PATH on Linux too (wine +
# d3dcompiler_47, wine's own, built on vkd3d-shader).
COPY fxc-wine/d3dc.c /tmp/d3dc.c
COPY fxc-wine/fxc /usr/local/bin/fxc
RUN x86_64-w64-mingw32-gcc -O2 -o /usr/local/lib/d3dc.exe /tmp/d3dc.c -ld3dcompiler -static-libgcc && rm /tmp/d3dc.c
ENV GPUI_FXC_PATH=/usr/local/bin/fxc
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.100.0 \
 && rustup target add x86_64-pc-windows-gnu
WORKDIR /src
