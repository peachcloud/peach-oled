#[macro_use]
extern crate log;
extern crate embedded_graphics;
extern crate linux_embedded_hal as hal;
extern crate ssd1306;

mod error;

use std::{
    process,
    result::Result,
    sync::{Arc, Mutex},
};

use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::{Font12x16, Font6x12, Font6x8, Font8x16};
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

//define the Msg struct for receiving display write commands
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

    io.add_method("write", move |params: Params| {
        info!("Received a 'write' request.");
        let m: Result<Msg, Error> = params.parse();
        let m: Msg = m?;
        validate(&m)?;

        let mut oled = oled_clone.lock().unwrap();

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

    info!("Creating JSON-RPC server.");
    let server = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
        ]))
        .start_http(&"127.0.0.1:3031".parse().unwrap())
        .expect("Unable to start RPC server");

    info!("Listening for requests.");
    server.wait();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use jsonrpc_core::ErrorCode;

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
            io.add_method("rpc_internal_err", |_| Err(Error::internal_error()));
            test::Rpc::from(io)
        };

        assert_eq!(
            rpc.request("rpc_internal_err", &()),
            r#"{
  "code": -32603,
  "message": "Internal error"
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
}
