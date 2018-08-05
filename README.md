WallFlower
==========

The aim of this project is to implement a kind of live photo frame. In my mind
it is driven by a small, silent computer like a [Raspberry Pi] or [Rock64]. It
hangs on the wall giving up to date info about things I'm interested in. These
things might include:

* Time & date
* Current weather
* Weather forecast
* Sunrise, sunset times
* Headlines from RSS feeds
* Reminders
* Photos
* Music player
* Twitter status and/or mentions

[Raspberry Pi]: https://www.raspberrypi.org/
[Rock64]: https://www.pine64.org/?page_id=7147

Building & Running
------------------

The project is implemented in [Rust] and uses the [Piston] framework. Rust
version 1.28.0 or newer is required to compile the application.

[Rust]: http://rust-lang.org/
[Piston]: http://piston.rs/

Compile:

    cargo build --release

Run:

    cargo run --release
