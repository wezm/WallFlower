require "view"
json = require "dkjson"

export class Weather extends View
  new: =>
    @data = nil
    @frame = frame
    @worker = love.thread.newThread "worker", "worker.lua"
    @worker\start!

  draw: =>
    status = "Updating..."
    -- love.graphics.setFont(18)
    if @data
      status = string.format "%.1f°C", @data.current.temperature_out

    love.graphics.print(status, 10, 500)
      

  update: =>
    new_data = @worker\get "weather"
    if new_data
      @data = json.decode(new_data)

  quit: =>
    print "Waiting for background worker..."
    @worker\set "quit", true
    @worker\wait!

