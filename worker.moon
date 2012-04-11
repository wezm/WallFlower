require "love.timer"

io = require "io"
http = require "socket.http"
ltn12 = require "ltn12"

next_update = love.timer.getTime!
thread = love.thread.getThread!

while true
  now = love.timer.getTime!

  if now > next_update
    print "Fetching weather"
    next_update = now + 30 -- redo in 30s

    -- This will block so only sleep in the alternate case
    rsp, status, auth = http.request "http://tweetnest.local/"

    if status == 200
      thread\set "weather", rsp
    -- http.request
    --  url: "http://tweetnest.local/"
    --  sink: ltn12.sink.file io.stdout
  else
    quit = thread\get "quit"
    if quit then return
    love.timer.sleep 0.5
