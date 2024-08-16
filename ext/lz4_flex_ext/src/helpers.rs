use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    ptr::null_mut,
};

use magnus::{rb_sys::AsRawValue, RString, TryConvert, Value};
use rb_sys::{
    rb_gc_guard, rb_str_locktmp, rb_str_resize, rb_str_unlocktmp, rb_thread_call_without_gvl,
    RSTRING_LEN, RSTRING_PTR,
};

#[derive(Debug)]
pub struct LockedRString<'a>(RString, PhantomData<&'a ()>);

impl<'a> LockedRString<'a> {
    fn new(string: RString) -> Self {
        unsafe { rb_str_locktmp(string.as_raw()) };

        Self(string, PhantomData)
    }

    pub(crate) fn as_slice(&self) -> &'a [u8] {
        // Safety: The string is locked, so it's safe to transmute it to a slice, but we want the
        // slice to match the lifetime of the LockedRString.
        unsafe { transmute(self.0.as_slice()) }
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl Drop for LockedRString<'_> {
    fn drop(&mut self) {
        unsafe { rb_str_unlocktmp(self.0.as_raw()) };
    }
}

impl TryConvert for LockedRString<'_> {
    fn try_convert(val: Value) -> Result<Self, magnus::Error> {
        TryConvert::try_convert(val).map(Self::new)
    }
}

#[derive(Debug)]
pub struct RStringMut<'a>(RString, PhantomData<&'a ()>);

impl<'a> RStringMut<'a> {
    pub(crate) fn buf_new(size: usize) -> Self {
        let string = RString::buf_new(size);
        Self(string, PhantomData)
    }

    pub(crate) fn as_mut_slice(&mut self) -> &'a mut [u8] {
        let raw_value = self.0.as_raw();

        unsafe {
            std::slice::from_raw_parts_mut(
                RSTRING_PTR(raw_value) as *mut u8,
                RSTRING_LEN(raw_value) as _,
            )
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn resize(&self, size: usize) {
        unsafe { rb_str_resize(self.0.as_raw(), size as _) };
    }

    pub(crate) fn into_inner(self) -> RString {
        self.0
    }
}

impl Drop for RStringMut<'_> {
    fn drop(&mut self) {
        // Keep the value alive while we hold this RString
        let _ = rb_gc_guard!(self.0.as_raw());
    }
}

pub(crate) fn nogvl_if_large<F, R>(input_len: usize, mut func: F) -> R
where
    F: FnMut() -> R,
    R: Sized,
{
    const MAX_INPUT_LEN: usize = 512;

    if input_len > MAX_INPUT_LEN {
        nogvl(func)
    } else {
        func()
    }
}

// https://docs.rs/rb-sys/0.9.97/rb_sys/bindings/uncategorized/fn.rb_thread_call_without_gvl.html
pub(crate) fn nogvl<F, R>(mut func: F) -> R
where
    F: FnMut() -> R,
    R: Sized,
{
    unsafe extern "C" fn call_without_gvl<F, R>(arg: *mut c_void) -> *mut c_void
    where
        F: FnMut() -> R,
        R: Sized,
    {
        let arg = arg as *mut (&mut F, &mut MaybeUninit<R>);
        // Safety: we know that the pointer is valid since we initialized it in nogvl
        let (func, result) = unsafe { &mut *arg };
        result.write(func());

        null_mut()
    }

    let result = MaybeUninit::uninit(); //location in memory that has not yet been initalized (init'd in call_without_gvl)
    let arg_ptr = &(&mut func, &result) as *const _ as *mut c_void; //tuple of func and result (raw + mutable) pointers

    unsafe {
        // https://github.com/ruby/ruby/blob/ruby_3_3/thread.c#L1550-L1634
        rb_thread_call_without_gvl(Some(call_without_gvl::<F, R>), arg_ptr, None, null_mut());
        result.assume_init()
    }
}
