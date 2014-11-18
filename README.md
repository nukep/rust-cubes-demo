This is a simple tech demo I wrote so I could learn Rust.
It utilizes OpenGL 3 and SDL2.

## Build instructions
You need the SDL2 binaries on your system, as this is a non-Rust dependency.

* Windows: Download SDL2.dll [here](https://www.libsdl.org/download-2.0.php) and place it on the project's root.
* Ubuntu/Debian: `sudo apt-get install libsdl2-dev`

Then run cargo on the project's root:

`cargo build --release`

## My impression of Rust

The experience I had with Rust was mostly pleasant.
It's a serious breath of fresh air, having done similar projects in C++.
The only caveat for me so far is the language's infancy and frequent changes
in Rust nightly. This will obviously go away over time, when Rust reaches 1.0 in
a few months.

I recommend Rust for _anybody_ who is familiar with C or C++.
