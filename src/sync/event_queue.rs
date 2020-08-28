use neon_runtime::raw::Env;
use neon_runtime::tsfn::{CallMode, Status, ThreadsafeFunction};

use context::{Context, TaskContext};
use result::JsResult;
use types::Value;

type Callback = Box<dyn FnOnce(Env) + Send + 'static>;

pub struct EventQueue {
    tsfn: ThreadsafeFunction<Callback>,
}

impl EventQueue {
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> Self {
        let tsfn = unsafe {
            ThreadsafeFunction::new(
                cx.env().to_raw(),
                Self::callback,
            )
        };

        Self { tsfn }
    }

    pub fn send<F, T>(&self, f: F)
    where
        F: FnOnce(TaskContext) -> JsResult<T> + Send + 'static,
        T: Value,
    {
        let callback = Box::new(move |env| {
            let env = unsafe { std::mem::transmute(env) };

            TaskContext::with_context(env, move |cx| {
                let _ = f(cx);
            });
        });

        assert_eq!(
            self.tsfn.call(callback, CallMode::napi_tsfn_blocking),
            Status::napi_ok,
        );
    }

    fn callback(env: Env, callback: Callback) {
        callback(env);
    }
}
