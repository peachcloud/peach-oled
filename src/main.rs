extern crate linux_embedded_hal as hal;
extern crate embedded_graphics;
extern crate ssd1306;
extern crate machine_ip;

use hal::I2cdev;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, Rect};
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::coord::Coord;
use ssd1306::prelude::*;
use ssd1306::Builder;

fn main() {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    /*
    disp.draw(
        Rect::new(Coord::new(48, 16), Coord::new(120, 50))
        .with_stroke(Some(1u8.into()))
        .into_iter(),
    );
    
    disp.draw(
        Line::new(Coord::new(8, 16), Coord::new(64, 48))
        .with_stroke(Some(1u8.into()))
        .into_iter(),
    );*/

    let local_addr = machine_ip::get().unwrap();

    disp.draw(
        Font6x8::render_str(&format!("{}", "PeachCloud".to_string()))
            .translate(Coord::new(0, 0))
            .into_iter(),
    );
    
    disp.draw(
        Font6x8::render_str(&format!("IP: {}", local_addr.to_string()))
            .translate(Coord::new(0, 16))
            .into_iter(),
    );

    disp.draw(
        Circle::new(Coord::new(64, 46), 16)
            .with_stroke(Some(1u8.into()))
            .into_iter(),
    );
    
    //disp.clear();
    
    disp.flush().unwrap();
}
