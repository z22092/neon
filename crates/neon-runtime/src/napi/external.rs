use std::mem::MaybeUninit;

use raw::{Env, Local};

use nodejs_sys as napi;

extern "C" fn finalize_external<T: Send + 'static>(
    _env: napi::napi_env,
    data: *mut std::ffi::c_void,
    _hint: *mut std::ffi::c_void,
) {
    unsafe {
        Box::<T>::from_raw(data as *mut _);
    }
}

pub unsafe fn deref<T: Send + 'static>(
    env: Env,
    local: Local,
) -> Option<*const T> {
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_typeof(
        env,
        local,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    let result = result.assume_init();

    if result != napi::napi_valuetype::napi_external {
        return None;
    }

    let mut result = MaybeUninit::uninit();
    let status = napi::napi_get_value_external(
        env,
        local,
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    Some(result.assume_init() as *const _)
}

pub unsafe fn create<T: Send + 'static>(env: Env, v: T) -> Local {
    let v = Box::new(v);
    let mut result = MaybeUninit::uninit();

    let status = napi::napi_create_external(
        env,
        Box::into_raw(v) as *mut _,
        Some(finalize_external::<T>),
        std::ptr::null_mut(),
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    result.assume_init()
}
