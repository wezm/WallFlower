require "clock"
require "weather"

love.load = ->
  export views = {
    Clock!
    Weather!
  }

love.quit = ->
  for view in *views do view\quit!
  nil

love.update = ->
  for view in *views do view\update!

love.draw = ->
  for view in *views do view\draw!

love.keypressed = (key) ->
  switch key
    when "escape"
      love.event.push "quit"
    when "f"
      -- TODO: Check if already in fullscreen: getMode!.fullscreen
      modes = love.graphics.getModes!
      table.sort modes, (a,b) -> b.width*b.height < a.width*a.height

      mode = modes[1]
      love.graphic.setMode mode.width, mode.height, true if #modes > 0
