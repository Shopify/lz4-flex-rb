use std::{
    cell::Cell,
    ffi::c_void,
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    ptr::null_mut,
};

use magnus::{
    encoding::{EncodingCapable, Index},
    exception::type_error,
    rb_sys::AsRawValue,
    value::ReprValue,
    Error, RString, Ruby, TryConvert, Value,
};
use rb_sys::{
    rb_gc_guard, rb_str_locktmp, rb_str_modify_expand, rb_str_set_len, rb_str_unlocktmp,
    rb_thread_call_without_gvl, RSTRING_PTR,
};

use crate::base_error_class;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub(crate) enum Encoding {
    Utf8 = 0,
    Binary = 1,
    UsAscii = 2,
}

impl Encoding {
    pub(crate) fn encindex(self) -> Index {
        let ruby = unsafe { Ruby::get_unchecked() };

        match self {
            Self::Utf8 => ruby.utf8_encindex(),
            Self::Binary => ruby.ascii8bit_encindex(),
            Self::UsAscii => ruby.usascii_encindex(),
        }
    }

    pub(crate) fn from_u8(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::Utf8),
            1 => Ok(Self::Binary),
            2 => Ok(Self::UsAscii),
            _ => Err(Error::new(
                base_error_class(),
                "unsupported encoding for string, please use Lz4Flex.compress_block instead"
                    .to_owned(),
            )),
        }
    }
}

impl TryFrom<RString> for Encoding {
    type Error = Error;

    fn try_from(string: RString) -> Result<Self, Self::Error> {
        let ruby = unsafe { Ruby::get_unchecked() };

        let enc = string.enc_get();

        if enc == ruby.utf8_encindex() {
            return Ok(Encoding::Utf8);
        }

        if enc == ruby.ascii8bit_encindex() {
            return Ok(Encoding::Binary);
        }

        if enc == ruby.usascii_encindex() {
            return Ok(Encoding::UsAscii);
        }

        Err(Error::new(
            base_error_class(),
            "unsupported encoding for string, please use Lz4Flex.compress_block instead".to_owned(),
        ))
    }
}

#[derive(Debug)]
pub struct LockedRString<'a>(pub(crate) RString, PhantomData<&'a ()>);

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

    pub(crate) fn encoding(&self) -> Result<Encoding, Error> {
        Encoding::try_from(self.0)
    }
}

impl Drop for LockedRString<'_> {
    fn drop(&mut self) {
        unsafe { rb_str_unlocktmp(self.0.as_raw()) };
        // Keep the value alive while we hold this RString
        let _ = rb_gc_guard!(self.0.as_raw());
    }
}

impl TryConvert for LockedRString<'_> {
    fn try_convert(val: Value) -> Result<Self, magnus::Error> {
        let rstring: RString = RString::from_value(val).ok_or(magnus::Error::new(
            type_error(),
            format!("expected String, got {}", val.class()),
        ))?;
        Ok(Self::new(rstring))
    }
}

#[derive(Debug)]
pub struct RStringMut<'a> {
    inner: RString,
    capa: Cell<usize>,
    // Used to safely borrow the underlying rstring buffer by associating it with the lifetime of
    // the RStringMut
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> RStringMut<'a> {
    pub(crate) fn buf_new(size: usize) -> Self {
        let string = RString::buf_new(size);
        Self {
            inner: string,
            capa: Cell::new(size),
            _lifetime: PhantomData,
        }
    }

    pub(crate) fn as_mut_slice(&mut self) -> &'a mut [u8] {
        let raw_value = self.inner.as_raw();

        unsafe {
            std::slice::from_raw_parts_mut(RSTRING_PTR(raw_value) as *mut u8, self.capa.get())
        }
    }

    pub(crate) fn capacity(&self) -> usize {
        self.capa.get()
    }

    pub(crate) fn expand(&self, size: usize) {
        if size <= self.capacity() {
            return;
        }

        unsafe { rb_str_modify_expand(self.inner.as_raw(), size as _) };
    }

    pub(crate) fn set_len(&self, len: usize) {
        unsafe { rb_str_set_len(self.inner.as_raw(), len as _) }
    }

    pub(crate) fn into_inner(self) -> RString {
        self.inner
    }
}

impl Drop for RStringMut<'_> {
    fn drop(&mut self) {
        // Keep the value alive while we hold this RString
        let _ = rb_gc_guard!(self.inner.as_raw());
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
