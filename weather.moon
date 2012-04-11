require "view"

export class Weather extends View
  new: =>
    @data = nil
    @frame = frame
    @worker = love.thread.newThread "worker", "worker.lua"
    @worker\start!

  draw: =>
    -- love.graphics.setFont(18)
    love.graphics.print("Weather", 10, 500)

  update: =>
    nil

  quit: =>
    print "Waiting for background worker..."
    @worker\set "quit", true
    @worker\wait!

