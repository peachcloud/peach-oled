## peach-oled

OLED microservice module for PeachCloud. Write to a 128x64 OLED display with SDD1306 driver (I2C) using [JSON-RPC](https://www.jsonrpc.org/specification) over http.

![Close-up, black-and-white photo of an Adafruit 128x64 1.3" OLED Bonnet. The circuit board features a 5-way joystick on the left side, two push-buttons on the right side (labelled #5 and #6), and a central OLED display. The display shows text reading: "PeachCloud" on the first line and "IP: 192.168.0.8" on the third line. A circle is displayed beneath the two lines of text and is horizontally-centered".](docs/images/peachcloud_oled.jpg)

### JSON-RPC API

| Method | Parameters | Description |
| --- | --- | --- |
| `write` | `x_coord`, `y_coord`, `string`, `font_size` | Write message to display at given co-ordinates using given font size |
| `clear` | | Clear the display |

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

### Example Usage

**Write Text to the OLED Display**

With microservice running, open a second terminal window and use `curl` to call server methods:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "write", "params" : {"x_coord": 0, "y_coord": 0, "string": "Welcome to PeachCloud!", "font_size": "6x8" }, "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2.0","result":success","id":1}`

OLED display shows:

`Welcome to PeachCloud!`

Write to the second line of the display:

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "write", "params" : {"x_coord": 0, "y_coord": 8, "string": "Born in cypherspace", "font_size": "6x12" }, "id":1 }' 127.0.0.1:3030`

OLED display shows:

`Welcome to PeachCloud!`  
`Born in cypherspace`

Validation checks are performed for all three parameters: `x_coord`, `y_coord` and `string`. An appropriate error is returned if the validation checks are not satisfied:

`{"jsonrpc":"2.0","error":{"code":1,"message":"validation error","data":"x_coord not in range 0-128"},"id":1}`

`{"jsonrpc":"2.0","error":{"code":1,"message":"validation error","data":"y_coord not in range 0-57"},"id":1}`

`{"jsonrpc":"2.0","error":{"code":1,"message":"validation error","data":"string length > 21 characters"},"id":1}`

An error is returned if one or all of the expected parameters are not supplied:

`{"jsonrpc":"2.0","error":{"code":-32602,"message":"invalid params","data":"Invalid params: missing field `string`."},"id":1}`

-----

**Clear the Display**

`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "clear", "id":1 }' 127.0.0.1:3030`

Server responds with:

`{"jsonrpc":"2,0","result":"success","id":1}`

### Licensing

AGPL-3.0
