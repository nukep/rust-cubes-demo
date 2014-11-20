This is a simple tech demo I wrote so I could learn Rust.
It utilizes OpenGL 3 and SDL2.

## Controls

Use your mouse to hover over and explode subcubes.

* Left click: Explode (subdivide) hovered subcube
* Right click: Rearrange all subcubes into their original positions
* Space: Hurl all subcubes outward
* "O" key: Toggle outlines
* "R" key: Reset to a single subcube

## Screenshots

![Screenshot 1](screenshots/screenshot1.png)
![Screenshot 2](screenshots/screenshot2.png)

## Build instructions
It is recommended you [install Rust nightly](http://www.rust-lang.org/install.html)
and Cargo on your machine.
An easy way to do this on Linux and OS X is by entering the following:

```bash
    curl -s https://static.rust-lang.org/rustup.sh | sudo sh
```

You need the SDL2 binaries on your system, as this is a non-Rust dependency.

* Windows: Download SDL2.dll [here](https://www.libsdl.org/download-2.0.php) and place it on the project's root.
* Ubuntu/Debian: `sudo apt-get install libsdl2-dev`
* Mac OS X: `brew install sdl2` (make sure you have [Homebrew](http://brew.sh/) installed)

Then run cargo on the project's root:

`cargo build --release`

## My impression of Rust

The experience I had with Rust was mostly pleasant.
It's a serious breath of fresh air, having done similar projects in C++.
The only caveat for me so far is the language's infancy and frequent changes
in Rust nightly. This will obviously go away over time, when Rust reaches 1.0 in
a few months.

I recommend Rust for _anybody_ who is familiar with C or C++.
