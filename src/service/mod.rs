use crate::{
    command_request::RequestData, CommandRequest, CommandResponse, KvError, MemTable, Storage,
};
use std::sync::Arc;
use tracing::debug;

mod command_service;

/// The abstract of handle
pub trait CommandService {
    /// Handle Command, return Response
    fn execute(self, store: &impl Storage) -> CommandResponse;
}

/// Notify events (immutable)
pub trait Notify<Arg> {
    fn notify(&self, arg: &Arg);
}

/// Notify events (mutable)
pub trait NotifyMut<Arg> {
    fn notify(&self, arg: &mut Arg);
}

impl<Arg> Notify<Arg> for Vec<fn(&Arg)> {
    #[inline]
    fn notify(&self, arg: &Arg) {
        for f in self {
            f(arg)
        }
    }
}

impl<Arg> NotifyMut<Arg> for Vec<fn(&mut Arg)> {
    #[inline]
    fn notify(&self, arg: &mut Arg) {
        for f in self {
            f(arg)
        }
    }
}

/// The struct of Service
pub struct Service<Store = MemTable> {
    inner: Arc<ServiceInner<Store>>,
}

impl<Store> Clone for Service<Store> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// The struct of Service inner
pub struct ServiceInner<Store> {
    store: Store,
    on_received: Vec<fn(&CommandRequest)>,
    on_executed: Vec<fn(&CommandResponse)>,
    on_before_send: Vec<fn(&mut CommandResponse)>,
    on_after_send: Vec<fn()>,
}

impl<Store: Storage> ServiceInner<Store> {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            on_received: Vec::new(),
            on_executed: Vec::new(),
            on_before_send: Vec::new(),
            on_after_send: Vec::new(),
        }
    }

    pub fn fn_received(mut self, f: fn(&CommandRequest)) -> Self {
        self.on_received.push(f);
        self
    }

    pub fn fn_executed(mut self, f: fn(&CommandResponse)) -> Self {
        self.on_executed.push(f);
        self
    }

    pub fn fn_before_send(mut self, f: fn(&mut CommandResponse)) -> Self {
        self.on_before_send.push(f);
        self
    }

    pub fn fn_after_send(mut self, f: fn()) -> Self {
        self.on_after_send.push(f);
        self
    }
}

impl<Store: Storage> From<ServiceInner<Store>> for Service<Store> {
    fn from(inner: ServiceInner<Store>) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl<Store: Storage> Service<Store> {
    pub fn execute(&self, cmd: CommandRequest) -> CommandResponse {
        debug!("Got request: {:?}", cmd);
        self.inner.on_received.notify(&cmd);
        let mut res = dispatch(cmd, &self.inner.store);
        debug!("Executed response: {:?}", res);
        self.inner.on_executed.notify(&res);
        self.inner.on_before_send.notify(&mut res);
        if !self.inner.on_before_send.is_empty() {
            debug!("Modified response: {:?}", res);
        }
        res
    }
}

pub fn dispatch(cmd: CommandRequest, store: &impl Storage) -> CommandResponse {
    match cmd.request_data {
        Some(RequestData::Hget(parma)) => parma.execute(store),
        Some(RequestData::Hgetall(param)) => param.execute(store),
        Some(RequestData::Hset(param)) => param.execute(store),
        None => KvError::InvalidCommand("Request has no data".into()).into(),
        _ => KvError::Internal("Not implemented".into()).into(),
    }
}

#[cfg(test)]
mod tests {
    use http::StatusCode;
    use std::thread;
    use tracing::info;

    use super::*;
    use crate::{MemTable, Value};

    #[test]
    fn service_should_work() {
        let service: Service = ServiceInner::new(MemTable::default()).into();
        let cloned = service.clone();

        let handle = thread::spawn(move || {
            let res = cloned.execute(CommandRequest::new_hset("t1", "k1", "v1".into()));
            assert_res_ok(res, &[Value::default()], &[]);
        });
        handle.join().unwrap();

        let res = service.execute(CommandRequest::new_hget("t1", "k1"));
        assert_res_ok(res, &["v1".into()], &[]);
    }

    #[test]
    fn event_registration_should_work() {
        fn b(cmd: &CommandRequest) {
            info!("Got {:?}", cmd);
        }
        fn c(res: &CommandResponse) {
            info!("{:?}", res);
        }
        fn d(res: &mut CommandResponse) {
            res.status = StatusCode::CREATED.as_u16() as _;
        }
        fn e() {
            info!("Data is sent");
        }

        let service: Service = ServiceInner::new(MemTable::default())
            .fn_received(|_: &CommandRequest| {})
            .fn_received(b)
            .fn_executed(c)
            .fn_before_send(d)
            .fn_after_send(e)
            .into();

        let res = service.execute(CommandRequest::new_hset("t1", "k1", "v1".into()));
        assert_eq!(res.status, StatusCode::CREATED.as_u16() as _);
        assert_eq!(res.message, "");
        assert_eq!(res.values, vec![Value::default()]);
    }
}

#[cfg(test)]
use crate::{Kvpair, Value};

// Return the success test
#[cfg(test)]
pub fn assert_res_ok(mut res: CommandResponse, values: &[Value], pairs: &[Kvpair]) {
    res.pairs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(res.status, 200);
    assert_eq!(res.message, "");
    assert_eq!(res.values, values);
    assert_eq!(res.pairs, pairs);
}

// Return the failure test
#[cfg(test)]
pub fn assert_res_error(res: CommandResponse, code: u32, msg: &str) {
    assert_eq!(res.status, code);
    assert!(res.message.contains(msg));
    assert_eq!(res.values, &[]);
    assert_eq!(res.pairs, &[]);
}
