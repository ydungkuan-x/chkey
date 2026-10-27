//! COM Class Factory for TSF Text Service.

use windows::core::{GUID, IUnknown, Interface, Result, implement};
use windows::Win32::Foundation::{BOOL, E_FAIL};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};
use windows::Win32::UI::TextServices::ITfTextInputProcessor;

use super::text_service::TsfTextService;

/// COM Class Factory for creating CHKey TsfTextService instances.
#[implement(IClassFactory)]
pub struct TsfClassFactory;

impl IClassFactory_Impl for TsfClassFactory_Impl {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut std::ffi::c_void,
    ) -> Result<()> {
        if punkouter.is_some() {
            return Err(windows::core::Error::from(E_FAIL));
        }
        let service: ITfTextInputProcessor = TsfTextService::new().into();
        unsafe {
            service.query(riid, ppvobject).ok()
        }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}
