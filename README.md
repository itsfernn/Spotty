<h1 align="center"> Spotty </h1>
<h4 align="center"> A libadwaita/GTK4-based Spotify client </h4>

<!-- TODO: Add showcase screenshot -->

Enjoy listening to your favorite Spotify content with **Spotty**: a libadwaita/GTK4-based Spotify client designed for GNOME!

Spotty is a fork of [Riff](https://github.com/Diegovsky/riff) (which itself was a fork of [Spot](https://github.com/xou816/spot)). It removes the native audio backend dependencies by delegating playback to a Spotify Connect-compatible daemon such as [spotifyd](https://github.com/Spotifyd/spotifyd), simplifying the architecture and enabling features like Spotify Connect out of the box.

> [!NOTE]
> **Spotty requires a premium Spotify account and a Spotify Connect daemon (e.g. spotifyd) running!**

## Installing

### From source

Requires Rust (stable), **GTK4**, **libadwaita**, and **blueprint-compiler**.

With meson:

```
meson setup target -Dbuildtype=debug -Doffline=false --prefix="$HOME/.local"
ninja install -C target
meson test -C target --verbose
```

This will install a `.desktop` file and the `spotty` executable in `~/.local/bin`.

To build an optimized release build, use `-Dbuildtype=release` instead.

### With GNOME Builder and flatpak

Open the project in GNOME Builder and configure with the `dev.itsfernn.Spotty` JSON. Then build.

## Usage notes

### Credentials

It is recommended to install a libsecret compliant keyring application, such as [GNOME Keyring](https://wiki.gnome.org/action/show/Projects/GnomeKeyring) (aka seahorse). This will allow saving your password securely between launches.

### Daemon setup

Spotty requires a Spotify Connect daemon running (e.g., [spotifyd](https://github.com/Spotifyd/spotifyd)). Once the daemon is connected, Spotty will appear as a remote control for it.

## Features

- playback control (play/pause, prev/next, seeking, shuffle, repeat)
- selection mode: easily browse and select multiple tracks to queue them
- browse your saved albums and playlists
- search albums and artists
- view an artist's releases
- view users' playlists
- view album info
- credentials management with Secret Service
- MPRIS integration
- playlist management (creation and edition)
- liked tracks

## License

Licensed under the [MIT License](LICENSE). This project is a fork of [Riff](https://github.com/Diegovsky/riff) which is in turn a fork of [Spot](https://github.com/xou816/spot).
