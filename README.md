<div align="center">

[![License][license-badge]](LICENSE)
[![CI Status][ci-badge]][ci]
[![Conda Platform][conda-badge]][conda-url]
[![Conda Downloads][conda-downloads-badge]][conda-url]
[![Project Chat][chat-badge]][chat-url]
[![Pixi Badge][pixi-badge]][pixi-url]

[license-badge]: https://img.shields.io/github/license/pavelzw/pixi-install-to-prefix?style=flat-square
[ci-badge]: https://img.shields.io/github/actions/workflow/status/pavelzw/pixi-install-to-prefix/ci.yml?style=flat-square&branch=main
[ci]: https://github.com/pavelzw/pixi-install-to-prefix/actions/
[conda-badge]: https://img.shields.io/conda/vn/conda-forge/pixi-install-to-prefix?style=flat-square
[conda-downloads-badge]: https://img.shields.io/conda/dn/conda-forge/pixi-install-to-prefix?style=flat-square
[conda-url]: https://prefix.dev/channels/conda-forge/packages/pixi-install-to-prefix
[chat-badge]: https://img.shields.io/discord/1082332781146800168.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2&style=flat-square
[chat-url]: https://discord.gg/kKV8ZxyzY4
[pixi-badge]: https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/prefix-dev/pixi/main/assets/badge/v0.json&style=flat-square
[pixi-url]: https://pixi.sh

</div>

## 🗂 Table of Contents

- [Introduction](#-introduction)
- [Installation](#-installation)
- [Usage](#-usage)

## 📖 Introduction

[Pixi](https://pixi.sh) installs your environments to `.pixi/envs/<env-name>` by default.
If you want to install your environment to an arbitrary location on your system, you can use `pixi-install-to-prefix`.

## 💿 Installation

You can install `pixi-install-to-prefix` using `pixi`:

```bash
pixi global install pixi-install-to-prefix
```

Or using `cargo`:

```bash
cargo install --locked --git https://github.com/pavelzw/pixi-install-to-prefix.git
```

Or by downloading our pre-built binaries from the [releases page](https://github.com/pavelzw/pixi-install-to-prefix/releases).

Instead of installing `pixi-install-to-prefix` globally, you can also use `pixi exec` to run `pixi-install-to-prefix` in a temporary environment:

```bash
pixi exec pixi-install-to-prefix ./my-environment
```

## 🎯 Usage

```text
Usage: pixi-install-to-prefix [OPTIONS] <PREFIX>

Arguments:
  <PREFIX>  The path to the prefix where you want to install the environment

Options:
  -l, --lockfile <LOCKFILE>        The path to the pixi lockfile [default: pixi.lock]
  -e, --environment <ENVIRONMENT>  The name of the pixi environment to install [default: default]
  -p, --platform <PLATFORM>        The platform you want to install for [default: osx-arm64]
  -c, --config <CONFIG>            The path to the pixi config file. By default, no config file is used
  -v, --verbose...                 Increase logging verbosity
  -q, --quiet...                   Decrease logging verbosity
  -h, --help                       Print help
```

### Mirror and S3 middleware

You can use mirror middleware by creating a configuration file as described in the [pixi documentation](https://pixi.sh/latest/reference/pixi_configuration/#mirror-configuration) and referencing it using `--config`.

```toml
[mirrors]
"https://conda.anaconda.org/conda-forge" = ["https://my.artifactory/conda-forge"]
```

If you are using [S3 in pixi](https://pixi.sh/latest/deployment/s3/), you can also add the appropriate S3 config in your config file and reference it.

```toml
[s3-options.my-s3-bucket]
endpoint-url = "https://s3.eu-central-1.amazonaws.com"
region = "eu-central-1"
force-path-style = false
```
