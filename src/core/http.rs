use std::sync::{atomic::{self, AtomicBool}, Arc};

use arc_swap::ArcSwap;
use serde::de::DeserializeOwned;

use super::Error;

pub struct AsyncRequest<T: Send + Sync> {
    request: ureq::Request,
    map_fn: fn(ureq::Response) -> Result<T, Error>,
    running: AtomicBool,
    pub result: ArcSwap<Option<Result<T, Error>>>
}

impl<T: Send + Sync + 'static> AsyncRequest<T> {
    pub fn new(request: ureq::Request, map_fn: fn(ureq::Response) -> Result<T, Error>) -> Self {
        AsyncRequest {
            request,
            map_fn,
            running: AtomicBool::new(false),
            result: ArcSwap::default()
        }
    }

    pub fn call(self: Arc<Self>) {
        self.result.store(Arc::new(None));
        self.running.store(true, atomic::Ordering::Release);
        std::thread::spawn(move || {
            let res = match self.request.clone().call() {
                Ok(v) => (self.map_fn)(v),
                Err(e) => Err(Error::from(e))
            };
            self.result.store(Arc::new(Some(res)));
            self.running.store(false, atomic::Ordering::Release);
        });
    }

    pub fn running(&self) -> bool {
        self.running.load(atomic::Ordering::Acquire)
    }
}

impl<T: Send + Sync + 'static + DeserializeOwned> AsyncRequest<T> {
    pub fn with_json_response(request: ureq::Request) -> AsyncRequest<T> {
        AsyncRequest::new(request, |res|
            Ok(serde_json::from_str(&res.into_string()?)?)
        )
    }
}

pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, Error> {
    let res = ureq::get(url).call()?;
    Ok(serde_json::from_str(&res.into_string()?)?)
}