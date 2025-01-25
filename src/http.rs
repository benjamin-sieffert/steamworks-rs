use super::*;

use sys::HTTPRequestHandle;

struct HttpRequestCompleted {
    pub local_handle: HTTPRequestHandle,
    pub succ: bool,
    pub status: usize,
    pub body_size: usize,
}

unsafe impl Callback for HttpRequestCompleted {
    const ID: i32 = sys::HTTPRequestCompleted_t__bindgen_ty_1::k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::HTTPRequestCompleted_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = *(raw as *mut sys::HTTPRequestCompleted_t);
        Self {
            local_handle: status.m_hRequest,
            succ: status.m_bRequestSuccessful,
            status: status.m_eStatusCode as _,
            body_size: status.m_unBodySize as _,
        }
    }
}
