require "view"

export class Clock extends View
  new: =>
    @inset = 10
    @timefont = love.graphics.newFont "League Gothic.otf", 200
    @datefont = love.graphics.newFont "League Gothic.otf", 100

  draw: =>
    love.graphics.setFont @timefont 
    love.graphics.printf os.date("%I:%M:%S"), @inset, @inset, love.graphics.getWidth! - 2 * @inset

    love.graphics.setFont @datefont
    love.graphics.printf os.date("%a %d %b %Y"), @inset, @inset + @timefont\getHeight!, love.graphics.getWidth! - 2 * @inset

