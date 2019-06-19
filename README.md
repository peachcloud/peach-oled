# peach-oled

[![Build Status](https://travis-ci.com/peachcloud/peach-oled.svg?branch=master)](https://travis-ci.com/peachcloud/peach-oled)

OLED microservice module for PeachCloud. Write to a 128x64 OLED display with SDD1306 driver (I2C) using [JSON-RPC](https://www.jsonrpc.org/specification) over http.

![Close-up, black-and-white photo of an Adafruit 128x64 1.3" OLED Bonnet. The circuit board features a 5-way joystick on the left side, two push-buttons on the right side (labelled #5 and #6), and a central OLED display. The display shows text reading: "PeachCloud" on the first line and "IP: 192.168.0.8" on the third line. A circle is displayed beneath the two lines of text and is horizontally-centered".](docs/images/peachcloud_oled.jpg)

### JSON-RPC API

| Method | Parameters | Description |
| --- | --- | --- |
| `write` | `x_coord`, `y_coord`, `string`, `font_size` | Write message to display buffer for given co-ordinates using given font size |
| `draw` | `bytes`, `width`, `height`, `x_coord`, `y_coord` | Draw graphic to display buffer for given byte array, dimensions and co-ordinates |
| `clear` | | Clear the display buffer |
| `flush` | | Flush the display |

| Font Sizes |
| --- |
| `6x8` |
| `6x12` |
| `8x16` |
| `12x16` |

### Setup

Clone this repo:

`git clone https://github.com/peachcloud/peach-oled.git`

Move into the repo and compile:

`cd peach-oled`  
`cargo build`

Run the binary:

`./target/debug/peach-oled`

Logging is made available with `env_logger`:

`RUST_LOG=info ./target/debug/peach-oled`

_Other logging levels include debug, warn and error._

### Example Usage

**Write Text to the OLED Display**

With microservice running, open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "write", "params" : {"x_coord": 0, "y_coord": 0, "string": "Welcome to PeachCloud!", "font_size": "6x8" }, "id":1 }' 127.0.0.1:3031`

Server responds with:

`{"jsonrpc":"2.0","result":success","id":1}`

OLED will remain blank because no flush command has been issued.

Write to the second line of the display:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "write", "params" : {"x_coord": 0, "y_coord": 8, "string": "Born in cypherspace", "font_size": "6x12" }, "id":1 }' 127.0.0.1:3031`

Flush the display:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "flush", "id":1 }' 127.0.0.1:3031`

OLED display shows:

`Welcome to PeachCloud!`  
`Born in cypherspace`

Validation checks are performed for all three parameters: `x_coord`, `y_coord` and `string`. An appropriate error is returned if the validation checks are not satisfied:

`{"jsonrpc":"2.0","error":{"code":1,"message":"Validation error: coordinate x out of range 0-128: 129."},"id":1}`

`{"jsonrpc":"2.0","error":{"code":1,"message":"validation error","data":"y_coord not in range 0-57"},"id":1}`

`{"jsonrpc":"2.0","error":{"code":1,"message":"Validation error: string length 47 out of range 0-21."},"id":1}`

An error is returned if one or all of the expected parameters are not supplied:

`{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: missing field `font_size`."},"id":1}`

-----

**Draw Graphic to the OLED Display**

With microservice running, open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "draw", "params" : {"bytes": [30, 0, 33, 0, 64, 128, 128, 64, 140, 64, 140, 64, 128, 64, 64, 128, 33, 0, 30, 0], "width": 10, "height": 10, "x_coord": 32, "y_coord": 32}, "id":1 }' 127.0.0.1:3031`

Server responds with:

`{"jsonrpc":"2.0","result":success","id":1}`

OLED will remain blank because no flush command has been issued.

Flush the display:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "flush", "id":1 }' 127.0.0.1:3031`

OLED display shows a 10x10 graphic of a dot inside a circle.

No validation checks are currently performed on the parameters of the `draw` RPC, aside from type-checks when the parameters are parsed.

-----

**Clear the Display**

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "clear", "id":1 }' 127.0.0.1:3031`

Server responds with:

`{"jsonrpc":"2,0","result":"success","id":1}`

### Licensing

AGPL-3.0
