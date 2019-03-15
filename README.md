## peach-oled

OLED microservice module for PeachCloud. Write to a 128x64 OLED display with SDD1306 driver (I2C) using [JSON-RPC](https://www.jsonrpc.org/specification) over http.

![Close-up, black-and-white photo of an Adafruit 128x64 1.3" OLED Bonnet. The circuit board features a 5-way joystick on the left side, two push-buttons on the right side (labelled #5 and #6), and a central OLED display. The display shows text reading: "PeachCloud" on the first line and "IP: 192.168.0.8" on the third line. A circle is displayed beneath the two lines of text and is horizontally-centered".](docs/images/peachcloud_oled.jpg)

### Setup

Clone this repo:

`git clone https://github.com/peachcloud/peach-oled.git`

Move into the repo and compile:

`cd peach-oled`  
`cargo build`

Run the binary:

`./target/debug/peach-oled`

### Licensing

AGPL-3.0
