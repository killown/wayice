# Wayice

Wayice is a sleek and efficient Wayland compositor built using Rust and Smithay. It offers a modern, fast, and lightweight window management experience, leveraging the safety and performance benefits of Rust for smooth rendering and enhanced security.

## Dependencies

You'll need to install the following dependencies (note, that those package
names may vary depending on your OS and linux distribution):

- `libwayland`
- `libxkbcommon`

#### These are needed for the "Udev/DRM backend"

- `libudev`
- `libinput`
- `libgbm`
- [`libseat`](https://git.sr.ht/~kennylevinsen/seatd)

If you want to enable X11 support (to run X11 applications within wayice),
then you'll need to install the following packages as well:
    - `xwayland`

## Build and run

You can run it with cargo after having cloned this repository:

```
cd wayice;

cargo run -- --{backend}
```

The currently available backends are:

- `--x11`: start wayice as an X11 client. This allows you to run the compositor inside an X11 session or any compositor supporting XWayland. Should be preferred over the winit backend where possible.
- `--winit`: start wayice as a [Winit](https://github.com/tomaka/winit) application. This allows you to run it
  inside of an other X11 or Wayland session.
- `--tty-udev`: start wayice in a tty with udev support. This is the "traditional" launch of a Wayland
  compositor. Note that this requires you to start wayice as root if your system does not have logind
  available.

