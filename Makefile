SRC = view.lua clock.lua weather.lua worker.lua conf.lua main.lua

all: $(SRC)

love: $(SRC)
	love .

%.lua: %.moon
	moonc -p $< > $@

clean:
	rm -f $(SRC)
