extern crate linux_embedded_hal as hal;
extern crate embedded_graphics;
extern crate ssd1306;
extern crate machine_ip;
extern crate failure;
extern crate validator;
#[macro_use]
extern crate validator_derive;

use failure::Fail;
use hal::I2cdev;
use embedded_graphics::prelude::*;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::coord::Coord;
use ssd1306::prelude::*;
use ssd1306::Builder;
use jsonrpc_http_server::jsonrpc_core::types::error::Error;
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use validator::{Validate, ValidationErrors};

//define the Msg struct for receiving display write commands
#[derive(Debug, Validate, Deserialize)]
struct Msg {
    #[validate(range(min = "0", max = "128", message = "x_coord not in range 0-128"))]
    x_coord: i32,
    #[validate(range(min = "0", max = "57", message = "y_coord not in range 0-57"))]
    y_coord: i32,
    #[validate(length(max = "21", message = "string length > 21 characters"))]
    string: String,
}

#[derive(Debug, Fail)]
pub enum WriteError {
    #[fail(display = "validation error")]
    Invalid { e: ValidationErrors },

    #[fail(display = "missing expected parameters")]
    MissingParams {e: Error},
}

impl From<WriteError> for Error {
    fn from(err: WriteError) -> Self {
        match &err {
            WriteError::Invalid {e} => {
                let err_clone = e.clone();
                // extract error from ValidationErrors
                let field_errs = err_clone.field_errors();
                let checks = vec!["x_coord", "y_coord", "string"];
                // check source of validation err
                for &error in &checks {
                    let validation_err = field_errs.get(&error);
                    if validation_err.is_some() {
                        let validation_err = validation_err.unwrap();
                        let err_msg = &validation_err[0].message;
                        let em = err_msg.clone();
                        let em = em.expect("failed to unwrap error msg");
                        return Error {
                            code: ErrorCode::ServerError(1),
                            message: "validation error".into(),
                            data: Some(format!("{}", em).into()),
                        };
                    }
                }
                Error {
                    code: ErrorCode::ServerError(1),
                    message: "validation error".into(),
                    data: Some(format!("{:?}", e).into()),
                }
            }
            WriteError::MissingParams {e} => Error {
                code: ErrorCode::ServerError(-32602),
                message: "invalid params".into(),
                data: Some(format!("{}", e.message).into()),
            },
            err => Error {
                code: ErrorCode::InternalError,
                message: "internal error".into(),
                data: Some(format!("{:?}", err).into()),
            },
        }
    }
}

fn main() {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let oled = Arc::new(Mutex::new(disp));
    let oled_clone = Arc::clone(&oled);
    let mut io = IoHandler::default();

    io.add_method("write", move |params: Params| {
        // parse parameters and match on result
        let m: Result<Msg> = params.parse();
        match m {
            // if result contains parameters, unwrap
            Ok(_) => {
                let m: Msg = m.unwrap();
                match m.validate() {
                    Ok(_) => {
                        let mut oled = oled_clone.lock().unwrap();
                        oled.draw(
                            Font6x8::render_str(&format!("{}", &m.string))
                                .translate(Coord::new(m.x_coord, m.y_coord))
                                .into_iter(),
                        );
                        oled.flush().unwrap();
                        Ok(Value::String("success".into()))
                    }
                    Err(e) => Err(Error::from(WriteError::Invalid {e})),
                }
            }
            Err(e) => Err(Error::from(WriteError::MissingParams {e})),
        }
    });

    let oled_clone = Arc::clone(&oled);

    io.add_method("clear", move |_| {
        let mut oled = oled_clone.lock().unwrap();
        let _ = oled.clear();
        oled.flush().unwrap();
        Ok(Value::String("success".into()))
    });

    let server = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
        ]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server");

    server.wait();
}
