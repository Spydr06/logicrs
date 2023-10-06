# Installation

Building is and installation is done via Cargo from source.

## Overview
  - [Installation](#installation)
  - [Overview](#overview)
  - [Compatibility](#compatibility)
  - [Dependencies](#dependencies)
  - [Building](#building)
  - [Installation](#installation-1)

## Compatibility

**Supported Operating Systems:**
- [x] Linux
- [x] Windows 10/11
- [x] MacOS

## Obtaining LogicRs

LogicRs can be obtained by simply cloning this repository or dowloading the `.zip` file on [GitHub](https://github.com/spydr06/LogicRs.git)
```console
$ git clone https://github.com/spydr06/logicrs.git --recursive
```
Prebuilt Binaries can be found on the `releases` section of this repository.

## Dependencies

**Libraries:**
- `gtk4`
- `libcairo`
- `libadwaita`

On UNIX, these dependencies mostly will be taken care of by Cargo and your distribution's package manager.
On Windows, you will need to install an unix-like environment like MSYS64/MINGW64 and take care of the dependencies yourself.

**Linux**

For rpm-based distributions (Fedora, Red Hat, etc.):

```bash
$ sudo dnf install gtk4-devel cairo-devel libadwaita-devel
```

For Ubuntu/Debian-based distributions (requires Ubuntu 23.04 or newer):

```bash
$ sudo apt install build-essential libadwaita-1-dev libcairo2-dev libgtk-4-dev
```

For Ubuntu/Debian-based distributions that do not ship with `libadwaita` version `1.2` yet (22.10 and below):

```bash
$ sudo apt install build-essential git libcairo2-dev libgtk-4-dev meson valac sassc gettext
$ git clone https://gitlab.gnome.org/GNOME/libadwaita.git -b libadwaita-1-2 --depth=1
$ cd libadwaita
$ meson . _build -Dgtk_doc=false -Dtests=false -Dexamples=false
```

then, execute with root permission:
```console
# ninja -C _build install
```

**MacOS**

On MacOS, use brew to install these dependencies.

```zsh
$ brew install cairo
$ brew install gtk4
$ brew install libadwaita
```

## Building

Building LogicRs is very easy by just running one command in the main directory of this repository:
```console
$ cargo build
```

To run the compiled program, use `cargo run`.

## Installation

Global installation can be done using this command:

```console
$ cargo install --path /path/to/this/repository
```

Or, if you don't want to clone this repository:

```console
$ cargo install --git https://github.com/Spydr06/logicrs
```
