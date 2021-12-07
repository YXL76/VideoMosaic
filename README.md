# VideoMosaic

Generate your video/photo mosaic.

![example-image](docs/example-image.webp)
![example-video](docs/example-video.webp)

## Installation

### Prebuilt binaries

Watch the [releases](https://github.com/YXL76/VideoMosaic/releases) page.

### Build from source

#### Requirements

- [Rust](https://www.rust-lang.org/) nightly
- [FFmpeg](https://www.ffmpeg.org/)

#### Build

```shell
git clone https://github.com/YXL76/VideoMosaic.git
cd VideoMosaic
cargo build --release
```

## Interface

- [GUI](gui)
- [CLI](cli)

## Usage

```shell
cargo run -- --help

Usage: video_mosaic [-t] [-a] [<command>] [<args>]

Video Mosaic.

Options:
  -t, --text-multithreading
                    if enabled, spread text workload in multiple threads when
                    multiple cores are available. By default, it is disabled.
  -a, --antialiasing
                    if set to true, the renderer will try to perform
                    antialiasing for some primitives.
  --help            display usage information

Commands:
  cli               CLI subcommand.
```

```shell
cargo run -- cli --help

Usage: video_mosaic cli <target> [-k <keyword...>] [-n <num>] [-l <library...>] [-s <size>] [--k <k>] [-h] [--calc-unit <calc-unit>] [--color-space <color-space>] [--dist-algo <dist-algo>] [--filter <filter>] [--quad-iter <quad-iter>] [--overlay <overlay>]

CLI subcommand.

Options:
  -k, --keyword     keywords to crawl the images
  -n, --num         the number of images that need to be crawled
  -l, --library     the path of the libraries
  -s, --size        the size of the block
  --k               k-means (k)
  -h, --hamerly     use Hamerly’s K-Means Clustering Algorithm
  --calc-unit       calculation unit (average, pixel, k_means)
  --color-space     color space (rgb, hsv, cielab)
  --dist-algo       distance algorithm (euclidean, ciede2000)
  --filter          filter (nearest, triangle, catmullRom, gaussian, lanczos3)
  --quad-iter       the number of iterations of the quadrant
  --overlay         overlay image and set the bottom image's alpha channel
  --help            display usage information
```

## Reference

- [Photographic mosaic](https://en.wikipedia.org/wiki/Photographic_mosaic)
- [Mazaika](https://www.mazaika.com/index.html)

## Acknowledgements

- [grokify/gosaic](https://github.com/grokify/gosaic)
- [okaneco/kmeans-colors](https://github.com/okaneco/kmeans-colors)
