use std::{
    ffi::CStr,
    fmt,
    marker::PhantomData,
    str::Utf8Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ffi;

use crate::NonNull;

ffi_enum_wrapper! {
    pub enum TofuPolicy: ffi::gpgme_tofu_policy_t {
        None = ffi::GPGME_TOFU_POLICY_NONE,
        Auto = ffi::GPGME_TOFU_POLICY_AUTO,
        Good = ffi::GPGME_TOFU_POLICY_GOOD,
        Unknown = ffi::GPGME_TOFU_POLICY_UNKNOWN,
        Bad = ffi::GPGME_TOFU_POLICY_BAD,
        Ask = ffi::GPGME_TOFU_POLICY_ASK,
    }
}

#[derive(Copy, Clone)]
pub struct TofuInfo<'a>(NonNull<ffi::gpgme_tofu_info_t>, PhantomData<&'a ()>);

unsafe impl<'a> Send for TofuInfo<'a> {}
unsafe impl<'a> Sync for TofuInfo<'a> {}

impl<'a> TofuInfo<'a> {
    impl_wrapper!(ffi::gpgme_tofu_info_t, PhantomData);

    #[inline]
    pub fn validity(&self) -> u32 {
        unsafe { (*self.as_raw()).validity() }
    }

    #[inline]
    pub fn policy(&self) -> TofuPolicy {
        unsafe { TofuPolicy::from_raw((*self.as_raw()).policy()) }
    }

    #[inline]
    pub fn signature_count(&self) -> u64 {
        unsafe { (*self.as_raw()).signcount.into() }
    }

    #[inline]
    pub fn encrypted_count(&self) -> u64 {
        unsafe { (*self.as_raw()).encrcount.into() }
    }

    #[inline]
    pub fn first_signed(&self) -> Option<SystemTime> {
        let sign_first = unsafe { (*self.as_raw()).signfirst };
        if sign_first > 0 {
            Some(UNIX_EPOCH + Duration::from_secs(sign_first.into()))
        } else {
            None
        }
    }

    #[inline]
    pub fn last_signed(&self) -> Option<SystemTime> {
        let sign_last = unsafe { (*self.as_raw()).signlast };
        if sign_last > 0 {
            Some(UNIX_EPOCH + Duration::from_secs(sign_last.into()))
        } else {
            None
        }
    }

    #[inline]
    pub fn first_encrypted(&self) -> Option<SystemTime> {
        let encr_first = unsafe { (*self.as_raw()).encrfirst };
        if encr_first > 0 {
            Some(UNIX_EPOCH + Duration::from_secs(encr_first.into()))
        } else {
            None
        }
    }

    #[inline]
    pub fn last_encrypted(&self) -> Option<SystemTime> {
        let encr_last = unsafe { (*self.as_raw()).encrlast };
        if encr_last > 0 {
            Some(UNIX_EPOCH + Duration::from_secs(encr_last.into()))
        } else {
            None
        }
    }

    #[inline]
    pub fn description(&self) -> Result<&'a str, Option<Utf8Error>> {
        self.description_raw()
            .map_or(Err(None), |s| s.to_str().map_err(Some))
    }

    #[inline]
    pub fn description_raw(&self) -> Option<&'a CStr> {
        unsafe {
            (*self.as_raw())
                .description
                .as_ref()
                .map(|s| CStr::from_ptr(s))
        }
    }
}

impl<'a> fmt::Debug for TofuInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TofuInfo")
            .field("raw", &self.as_raw())
            .field("description", &self.description_raw())
            .field("validity", &self.validity())
            .field("policy", &self.policy())
            .field("signature_count", &self.signature_count())
            .field("first_signed", &self.first_signed())
            .field("last_signed", &self.last_signed())
            .field("encrypted_count", &self.encrypted_count())
            .field("first_encrypt", &self.first_encrypted())
            .field("last_encrypt", &self.last_encrypted())
            .finish()
    }
}
