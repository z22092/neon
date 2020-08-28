use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use neon_runtime::reference;

use context::Context;
use handle::Handle;
use object::Object;

#[derive(Clone)]
struct NapiRef(*mut c_void);

pub struct Persistent<T> {
    internal: NapiRef,
    _phantom: PhantomData<T>,
}

unsafe impl<T> Send for Persistent<T> {}
unsafe impl<T> Sync for Persistent<T> {}

impl<T: Object> Persistent<T> {
    pub fn new<'a, C: Context<'a>>(cx: &mut C, value: Handle<T>) -> Self {
        let env = cx.env().to_raw();
        let internal = unsafe {
            reference::new(env, value.to_raw())
        };

        Self {
            internal: NapiRef(internal as *mut _),
            _phantom: PhantomData,
        }
    }

    pub fn drop<'a, C: Context<'a>>(self, cx: &mut C) {
        let env = cx.env().to_raw();
        let internal = ManuallyDrop::new(self).internal.0 as *mut _;

        unsafe {
            reference::unreference(env, internal);
        }
    }

    pub fn deref<'a, C: Context<'a>>(self, cx: &mut C) -> Handle<'a, T> {
        let env = cx.env();
        let internal = ManuallyDrop::new(self).internal.0 as *mut _;

        let local = unsafe {
            reference::get(env.to_raw(), internal)
        };

        unsafe {
            reference::unreference(env.to_raw(), internal);
        }

        Handle::new_internal(T::from_raw(env, local))
    }

    pub fn clone<'a, C: Context<'a>>(&self, cx: &mut C) -> Self {
        let env = cx.env();
        let internal = self.internal.0 as *mut _;

        unsafe {
            reference::reference(env.to_raw(), internal);
        };

        Self {
            internal: self.internal.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for Persistent<T> {
    fn drop(&mut self) {
        // Destructors are called during stack unwinding, prevent a double
        // panic and instead prefer to leak.
        if !std::thread::panicking() {
            // TODO: Link to documentation here
            panic!("Must call `drop` or `deref` on a `Persistent`.");
        }
    }
}
