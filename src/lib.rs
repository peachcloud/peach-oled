#[macro_use]
extern crate log;
extern crate embedded_graphics;
extern crate linux_embedded_hal as hal;
extern crate ssd1306;

mod error;

use std::{
    env, process,
    result::Result,
    sync::{Arc, Mutex},
};

use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::{Font12x16, Font6x12, Font6x8, Font8x16};
use embedded_graphics::image::Image1BPP;
use embedded_graphics::prelude::*;
use hal::I2cdev;
use jsonrpc_core::{types::error::Error, IoHandler, Params, Value};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, ServerBuilder};
#[allow(unused_imports)]
use jsonrpc_test as test;
use serde::Deserialize;
use snafu::{ensure, ResultExt};
use ssd1306::prelude::*;
use ssd1306::Builder;

use crate::error::{I2CError, InvalidCoordinate, InvalidString, OledError};

//define the Graphic struct for receiving draw commands
#[derive(Debug, Deserialize)]
pub struct Graphic {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    x_coord: i32,
    y_coord: i32,
}

//define the Msg struct for receiving write commands
#[derive(Debug, Deserialize)]
pub struct Msg {
    x_coord: i32,
    y_coord: i32,
    string: String,
    font_size: String,
}

fn validate(m: &Msg) -> Result<(), OledError> {
    ensure!(
        m.string.len() <= 21,
        InvalidString {
            len: m.string.len()
        }
    );

    ensure!(
        m.x_coord >= 0,
        InvalidCoordinate {
            coord: "x".to_string(),
            range: "0-128".to_string(),
            value: m.x_coord,
        }
    );

    ensure!(
        m.x_coord < 129,
        InvalidCoordinate {
            coord: "x".to_string(),
            range: "0-128".to_string(),
            value: m.x_coord,
        }
    );

    ensure!(
        m.y_coord >= 0,
        InvalidCoordinate {
            coord: "y".to_string(),
            range: "0-47".to_string(),
            value: m.y_coord,
        }
    );

    ensure!(
        m.y_coord < 148,
        InvalidCoordinate {
            coord: "y".to_string(),
            range: "0-47".to_string(),
            value: m.y_coord,
        }
    );

    Ok(())
}

pub fn run() -> Result<(), OledError> {
    info!("Starting up.");

    debug!("Creating interface for I2C device.");
    let i2c = I2cdev::new("/dev/i2c-1").context(I2CError)?;

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    info!("Initializing the display.");
    disp.init().unwrap_or_else(|_| {
        error!("Problem initializing the OLED display.");
        process::exit(1);
    });

    debug!("Flushing the display.");
    disp.flush().unwrap_or_else(|_| {
        error!("Problem flushing the OLED display.");
        process::exit(1);
    });

    let oled = Arc::new(Mutex::new(disp));
    let oled_clone = Arc::clone(&oled);

    info!("Creating JSON-RPC I/O handler.");
    let mut io = IoHandler::default();

    io.add_method("clear", move |_| {
        let mut oled = oled_clone.lock().unwrap();
        info!("Clearing the display.");
        oled.clear();
        info!("Flushing the display.");
        oled.flush().unwrap_or_else(|_| {
            error!("Problem flushing the OLED display.");
            process::exit(1);
        });
        Ok(Value::String("success".into()))
    });

    let oled_clone = Arc::clone(&oled);

    io.add_method("draw", move |params: Params| {
        let g: Result<Graphic, Error> = params.parse();
        let g: Graphic = g?;
        // TODO: add simple byte validation function
        let mut oled = oled_clone.lock().unwrap();
        info!("Drawing image to the display.");
        let im =
            Image1BPP::new(&g.bytes, g.width, g.height).translate(Coord::new(g.x_coord, g.y_coord));
        oled.draw(im.into_iter());
        Ok(Value::String("success".into()))
    });

    let oled_clone = Arc::clone(&oled);

    io.add_method("flush", move |_| {
        let mut oled = oled_clone.lock().unwrap();
        info!("Flushing the display.");
        oled.flush().unwrap_or_else(|_| {
            error!("Problem flushing the OLED display.");
            process::exit(1);
        });
        Ok(Value::String("success".into()))
    });

    let oled_clone = Arc::clone(&oled);

    io.add_method("ping", |_: Params| Ok(Value::String("success".to_string())));

    io.add_method("write", move |params: Params| {
        info!("Received a 'write' request.");
        let m: Result<Msg, Error> = params.parse();
        let m: Msg = m?;
        validate(&m)?;

        let mut oled = oled_clone.lock().unwrap();

        info!("Writing to the display.");
        if m.font_size == "6x8" {
            oled.draw(
                Font6x8::render_str(&m.string.to_string())
                    .translate(Coord::new(m.x_coord, m.y_coord))
                    .into_iter(),
            );
        } else if m.font_size == "6x12" {
            oled.draw(
                Font6x12::render_str(&m.string.to_string())
                    .translate(Coord::new(m.x_coord, m.y_coord))
                    .into_iter(),
            );
        } else if m.font_size == "8x16" {
            oled.draw(
                Font8x16::render_str(&m.string.to_string())
                    .translate(Coord::new(m.x_coord, m.y_coord))
                    .into_iter(),
            );
        } else if m.font_size == "12x16" {
            oled.draw(
                Font12x16::render_str(&m.string.to_string())
                    .translate(Coord::new(m.x_coord, m.y_coord))
                    .into_iter(),
            );
        }

        Ok(Value::String("success".into()))
    });

    let http_server =
        env::var("PEACH_OLED_SERVER").unwrap_or_else(|_| "127.0.0.1:5112".to_string());

    info!("Starting JSON-RPC server on {}.", http_server);
    let server = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
        ]))
        .start_http(
            &http_server
                .parse()
                .expect("Invalid HTTP address and port combination"),
        )
        .expect("Unable to start RPC server");

    info!("Listening for requests.");
    server.wait();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use hal::i2cdev::linux::LinuxI2CError;
    use jsonrpc_core::ErrorCode;
    use nix::Error as NixError;
    use std::io::Error as IoError;
    use std::io::ErrorKind;

    // test to ensure correct success response
    #[test]
    fn rpc_success() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_success_response", |_| {
                Ok(Value::String("success".into()))
            });
            test::Rpc::from(io)
        };

        assert_eq!(rpc.request("rpc_success_response", &()), r#""success""#);
    }

    // test to ensure correct internal error response
    #[test]
    fn rpc_internal_error() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_internal_error", |_| Err(Error::internal_error()));
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_internal_error", &()),
            r#"{
  "code": -32603,
  "message": "Internal error"
}"#
        );
    }

    // test to ensure correct I2CError error response (io::Error variant)
    #[test]
    fn rpc_i2c_io_error() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_i2c_io_error", |_| {
                let io_err = IoError::new(ErrorKind::PermissionDenied, "oh no!");
                let source = LinuxI2CError::Io(io_err);
                Err(Error::from(OledError::I2CError { source }))
            });
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_i2c_io_error", &()),
            r#"{
  "code": -32000,
  "message": "I2C device error: oh no!"
}"#
        );
    }

    // test to ensure correct I2CError error response (nix::Error variant)
    #[test]
    fn rpc_i2c_nix_error() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_i2c_nix_error", |_| {
                let nix_err = NixError::InvalidPath;
                let source = LinuxI2CError::Nix(nix_err);
                Err(Error::from(OledError::I2CError { source }))
            });
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_i2c_nix_error", &()),
            r#"{
  "code": -32000,
  "message": "I2C device error: Invalid path"
}"#
        );
    }

    // test to ensure correct InvalidCoordinate error response
    #[test]
    fn rpc_invalid_coord() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_invalid_coord", |_| {
                Err(Error::from(OledError::InvalidCoordinate {
                    coord: "x".to_string(),
                    range: "0-128".to_string(),
                    value: 321,
                }))
            });
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_invalid_coord", &()),
            r#"{
  "code": -32001,
  "message": "Validation error: coordinate x out of range 0-128: 321"
}"#
        );
    }

    // test to ensure correct InvalidFontSize error response
    #[test]
    fn rpc_invalid_fontsize() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_invalid_fontsize", |_| {
                Err(Error::from(OledError::InvalidFontSize {
                    font: "24x32".to_string(),
                }))
            });
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_invalid_fontsize", &()),
            r#"{
  "code": -32002,
  "message": "Validation error: 24x32 is not an accepted font size. Use 6x8, 6x12, 8x16 or 12x16 instead"
}"#
        );
    }

    // test to ensure correct InvalidString error response
    #[test]
    fn rpc_invalid_string() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_invalid_string", |_| {
                Err(Error::from(OledError::InvalidString { len: 22 }))
            });
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_invalid_string", &()),
            r#"{
  "code": -32003,
  "message": "Validation error: string length 22 out of range 0-21"
}"#
        );
    }

    // test to ensure correct invalid parameters error response
    #[test]
    fn rpc_invalid_params() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_invalid_params", |_| {
                let e = Error {
                    code: ErrorCode::InvalidParams,
                    message: String::from("invalid params"),
                    data: Some(Value::String(
                        "Invalid params: invalid type: null, expected struct Msg.".into(),
                    )),
                };
                Err(Error::from(OledError::MissingParameter { e }))
            });
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_invalid_params", &()),
            r#"{
  "code": -32602,
  "message": "invalid params",
  "data": "Invalid params: invalid type: null, expected struct Msg."
}"#
        );
    }

    // test to ensure correct parse error response
    #[test]
    fn rpc_parse_error() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_parse_error", |_| {
                let e = Error {
                    code: ErrorCode::ParseError,
                    message: String::from("Parse error"),
                    data: None,
                };
                Err(Error::from(OledError::ParseError { e }))
            });
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_parse_error", &()),
            r#"{
  "code": -32700,
  "message": "Parse error"
}"#
        );
    }

}
