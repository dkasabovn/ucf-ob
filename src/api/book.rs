// returns prices separated by commas

use std::ops::{Deref, DerefMut};
use fast_book::comm::urcp::{
    read_response_vec, write_request, LevelViewRequest, OBReqType, OBRequest, OBRespType,
    PriceViewResponse,
};
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use actix_web::web::Data;

pub fn get_book_data(stream: Data<Mutex<UnixStream>>, oid: u16) -> Option<PriceViewResponse> {
    let mut stream = stream.lock().unwrap();
    let mut inner = stream.deref_mut();

    write_request(
        &mut inner,
        &OBReqType::LEVELVIEW,
        &OBRequest {
            level_view: LevelViewRequest { ob_id: oid },
        },
    ).unwrap();

    let response_vec = read_response_vec(&mut inner).unwrap();

    for response in response_vec {
        match response.typ {
            OBRespType::LEVELVIEW => {
                unsafe {
                    let array = response.resp.view.prices;
                    return Some(PriceViewResponse { prices: array });
                }
            }
            _ => {}
        }
    }

    None
}
