// returns prices separated by commas

use actix_web::error::ErrorBadRequest;
use actix_web::web::Data;
use actix_web::Error;
use fast_book::comm::urcp::{
    read_response, read_response_vec, write_request, LevelViewRequest, OBReqType, OBRequest,
    OBRespType, PriceViewResponse,
};
use std::ops::{Deref, DerefMut};
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

pub fn get_book_data(stream: &mut UnixStream, oid: u16) -> Result<PriceViewResponse, Error> {
    write_request(
        stream,
        &OBReqType::LEVELVIEW,
        &OBRequest {
            level_view: LevelViewRequest { ob_id: oid },
        },
    )?;

    let response = read_response(stream)?;

    match response.typ {
        OBRespType::LEVELVIEW => unsafe {
            let array = response.resp.view.prices;
            return Ok(PriceViewResponse { prices: array });
        },
        _ => Err(ErrorBadRequest("Couldn't get level view")),
    }
}
