use std::any::{self, Any};
use std::ops::Deref;

use neon_runtime::raw;
use neon_runtime::external;

use crate::context::Context;
use crate::context::internal::Env;
use crate::handle::{Managed, Handle};
use crate::result::JsResult;
use crate::types::internal::ValueInternal;
use crate::types::Value;

type BoxAny = Box<dyn Any + Send + 'static>;

#[repr(C)]
pub struct JsBox<T: Send + 'static> {
    local: raw::Local,
    // `JsBox` can not verify the lifetime. Store a raw pointer to force uses
    // to be marked unsafe.
    internal: *const T,
}

// Custom `Clone` implementation since `T` might not be `Clone`
impl<T: Send + 'static> Clone for JsBox<T> {
    fn clone(&self) -> Self {
        JsBox {
            local: self.local,
            internal: self.internal,
        }
    }
}

impl<T: Send + 'static> Copy for JsBox<T> {}

impl<T: Send + 'static> Value for JsBox<T> { }

impl<T: Send + 'static> Managed for JsBox<T> {
    fn to_raw(self) -> raw::Local {
        self.local
    }

    fn from_raw(env: Env, h: raw::Local) -> Self {
        let v = unsafe {
            external::deref::<BoxAny>(env.to_raw(), h)
                .map(|v| &*v)
        };

        let internal = v
            .and_then(|v| v.downcast_ref())
            .expect("Expected type to already be validated");

        Self {
            local: h,
            internal,
        }        
    }
}

impl<T: Send + 'static> ValueInternal for JsBox<T> {
    fn name() -> String {
        any::type_name::<Self>().to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        let v = unsafe {
            external::deref::<BoxAny>(env.to_raw(), other.to_raw())
                .map(|v| &*v)
        };

        v.map(|v| v.is::<T>()).unwrap_or(false)
    }

    fn downcast<Other: Value>(env: Env, other: Other) -> Option<Self> {
        let local = other.to_raw();
        let v = unsafe {
            external::deref::<BoxAny>(env.to_raw(), local)
                .map(|v| &*v)
        };

        v.and_then(|v| v.downcast_ref())
            .map(|internal| Self {
                local,
                internal,
            })
    }
}

impl<T: Send + 'static> JsBox<T> {
    pub fn new<'a, C>(cx: &mut C, v: T) -> JsResult<'a, JsBox<T>>
    where
        C: Context<'a>,
        T: Send + 'static,
    {
        let v = Box::new(v) as BoxAny;
        let internal = v.downcast_ref().unwrap() as *const _;
        let local = unsafe {
            external::create(cx.env().to_raw(), v)
        };

        Ok(Handle::new_internal(Self {
            local,
            internal,
        }))
    }
}

impl<'a, T: Send + 'static> Deref for JsBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.internal }
    }
}
