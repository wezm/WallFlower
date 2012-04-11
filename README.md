WallFlower
==========

The aim of this project is to implement a kind of live photo frame. In
my mind it is driven by a small, silent computer like an [alix3d3] and
hangs on the wall giving up to date info about things I'm interested
in. These things might include:

* Time & date
* Current weather
* Weather forecast
* Sunrise, sunset times
* Headlines from RSS feeds
* Reminders
* Photos
* Music player
* Twitter status and/or mentions

[alix3d3]: http://pcengines.ch/alix3d3.htm

Building & Running
------------------

The project is implemented in [MoonScript][moonscript] and uses the
[LÖVE][love] framework. You will need both of these installed to compile
the moon files to lua, then run them with love.

[moonscript]: http://moonscript.org/
[love]: https://love2d.org/

Compile the files:

    make

Run the app (*couldn't resist*):

    make love

License
-------

The project is licensed under the [zlib/libpng licence][license] (the
same as LÖVE). See the [LICENSE][license-file] file for full details.

[license-file]: https://github.com/wezm/WallFlower/blob/master/LICENSE.md
[license]: http://www.opensource.org/licenses/zlib-license.php
