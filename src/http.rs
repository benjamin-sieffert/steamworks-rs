use std::ffi::CString;

use super::*;

use sys::{EHTTPMethod, HTTPRequestHandle, SteamAPICall_t};

/// Access to the steam http interface
pub struct Http<Manager> {
    pub(crate) http: *mut sys::ISteamHTTP,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Http<Manager> {
    /// https://partner.steamgames.com/doc/api/ISteamHTTP#CreateHTTPRequest
    pub fn create_http_request(
        &self,
        method: EHTTPMethod,
        url: &str,
    ) -> Result<HttpRequest, SteamError> {
        let c_url = CString::new(url).map_err(|_| SteamError::InvalidParameter)?;
        let handle;
        unsafe {
            handle = sys::SteamAPI_ISteamHTTP_CreateHTTPRequest(self.http, method, c_url.as_ptr());
        }
        if handle == sys::INVALID_HTTPREQUEST_HANDLE {
            Err(SteamError::HttpRequestUrlEmpty)
        } else {
            Ok(HttpRequest(handle))
        }
    }

    pub fn create_get_request(&self, url: &str) -> Result<HttpRequest, SteamError> {
        self.create_http_request(EHTTPMethod::k_EHTTPMethodGET, url)
    }

    pub fn create_post_request(
        &self,
        url: &str,
        content_type: &'static str,
        body: &mut [u8],
    ) -> Result<HttpRequest, SteamError> {
        let c_cnt = CString::new(content_type).map_err(|_| SteamError::InvalidParameter)?;

        let r = self.create_http_request(EHTTPMethod::k_EHTTPMethodPOST, url)?;

        unsafe {
            let succ = sys::SteamAPI_ISteamHTTP_SetHTTPRequestRawPostBody(
                self.http,
                r.0,
                c_cnt.as_ptr(),
                body.as_mut_ptr(),
                body.len() as _,
            );

            if !succ {
                return Err(SteamError::HttpSetBodyError);
            }
        }

        Ok(r)
    }

    pub fn send_http_request<F>(&self, handle: HttpRequest, cb: F)
    where
        F: FnOnce(Result<HttpRequestCompleted, SteamError>) + 'static + Send,
    {
        unsafe {
            let mut api_call = 0;
            let succ = sys::SteamAPI_ISteamHTTP_SendHTTPRequest(self.http, handle.0, &mut api_call);

            if !succ {
                return cb(Err(SteamError::InvalidParameter));
            }

            register_call_result::<sys::HTTPRequestCompleted_t, _, _>(
                &self._inner,
                api_call,
                HttpRequestCompleted::ID, // Not sure if correct, but is also unused by register_call_result body.
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(v.into())
                    })
                },
            );
        }
    }
}

pub struct HttpRequest(HTTPRequestHandle);

struct HttpRequestCompleted {
    pub local_handle: HTTPRequestHandle,
    pub succ: bool,
    pub status: usize,
    pub body_size: usize,
}

impl From<&sys::HTTPRequestCompleted_t> for HttpRequestCompleted {
    fn from(status: &sys::HTTPRequestCompleted_t) -> Self {
        Self {
            local_handle: status.m_hRequest,
            succ: status.m_bRequestSuccessful,
            status: status.m_eStatusCode as _,
            body_size: status.m_unBodySize as _,
        }
    }
}

unsafe impl Callback for HttpRequestCompleted {
    const ID: i32 = sys::HTTPRequestCompleted_t__bindgen_ty_1::k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::HTTPRequestCompleted_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = *(raw as *mut sys::HTTPRequestCompleted_t);
        (&status).into()
    }
}
