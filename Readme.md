<h1 align="center">
    ✨ Shader art using webgpu  ✨
</h1>

This is the implementation of [An introduction to Shader Art Coding ](https://www.youtube.com/watch?v=f4s1h2YETNY) in Rust using webgpu.

<p align="center">
  <img src="assets/demo.gif" alt="demo" />
</p>

## 🔌 Setup

You need:

- [Rust](https://www.rust-lang.org/) compiler and [Cargo package manager](https://doc.rust-lang.org/cargo/)
- One of the [supported backends](https://github.com/gfx-rs/wgpu#supported-platforms) by wgpu crate.

## 🚀 Getting started

```
$ git clone https://github.com/pythops/shader-art-rs
$ cd shader-art-rs/
$ cargo run
```

## ⚙️ Configuration

### Save as gif

```
$ cargo run -- --save animation.gif
```

### Speed up the animation

You can speed up the animation or the generated gif.

```
$ cargo run -- --speed <Speed factor: u8>
```

### GIF resolution

```
$ cargo run -- --save animation.gif --resolution <widthxheight>
```

the default resolution is `512x512`

## 🙏 Acknowledgments

Thanks to [@sorth](https://github.com/sotrh) for the amazing tutorial [learn-wgpu](https://github.com/sotrh/learn-wgpu)

## ⚖️ License

AGPLv3
