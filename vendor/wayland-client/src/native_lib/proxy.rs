use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wayland_commons::utils::UserData;
use wayland_commons::wire::ArgumentType;
use wayland_commons::MessageGroup;
use {Interface, Proxy};

use super::EventQueueInner;

use wayland_sys::client::*;
use wayland_sys::common::*;

pub struct ProxyInternal {
    alive: AtomicBool,
    user_data: UserData,
}

impl ProxyInternal {
    pub fn new(user_data: UserData) -> ProxyInternal {
        ProxyInternal {
            alive: AtomicBool::new(true),
            user_data,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ProxyInner {
    internal: Option<Arc<ProxyInternal>>,
    ptr: *mut wl_proxy,
    is_wrapper: bool,
}

unsafe impl Send for ProxyInner {}
unsafe impl Sync for ProxyInner {}

impl ProxyInner {
    pub(crate) fn is_alive(&self) -> bool {
        self.internal
            .as_ref()
            .map(|i| i.alive.load(Ordering::Acquire))
            .unwrap_or(true)
    }

    pub(crate) fn is_external(&self) -> bool {
        self.internal.is_none()
    }

    pub(crate) fn version(&self) -> u32 {
        if !self.is_alive() {
            return 0;
        }
        unsafe { ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_get_version, self.ptr) as u32 }
    }

    pub(crate) fn id(&self) -> u32 {
        if !self.is_alive() {
            return 0;
        }
        unsafe { ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_get_id, self.ptr) }
    }

    pub(crate) fn get_user_data<UD: 'static>(&self) -> Option<&UD> {
        if let Some(ref inner) = self.internal {
            inner.user_data.get::<UD>()
        } else {
            None
        }
    }

    pub(crate) fn send<I: Interface>(&self, msg: I::Request) {
        if let Some(ref internal) = self.internal {
            // object is managed
            if !internal.alive.load(Ordering::Acquire) {
                // don't send message to dead objects !
                return;
            }
        }
        let destructor = msg.is_destructor();
        msg.as_raw_c_in(|opcode, args| unsafe {
            ffi_dispatch!(
                WAYLAND_CLIENT_HANDLE,
                wl_proxy_marshal_array,
                self.ptr,
                opcode,
                args.as_ptr() as *mut _
            );
        });

        if destructor {
            // we need to destroy the proxy now
            if let Some(ref internal) = self.internal {
                internal.alive.store(false, Ordering::Release);
            }
            unsafe {
                ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_destroy, self.ptr);
            }
        }
    }

    pub(crate) fn send_constructor<I, J>(
        &self,
        msg: I::Request,
        version: Option<u32>,
    ) -> Result<NewProxyInner, ()>
    where
        I: Interface,
        J: Interface,
    {
        if let Some(ref internal) = self.internal {
            // object is managed
            if !internal.alive.load(Ordering::Acquire) {
                // don't send message to dead objects !
                return Err(());
            }
        }
        let destructor = msg.is_destructor();

        let opcode = msg.opcode();

        // sanity check
        let mut nid_idx = I::Request::MESSAGES[opcode as usize]
            .signature
            .iter()
            .position(|&t| t == ArgumentType::NewId)
            .expect("Trying to use 'send_constructor' with a message not creating any object.");

        if let Some(o) = I::Request::child(opcode, 1, &()) {
            if !o.is_interface::<J>() {
                panic!("Trying to use 'send_constructor' with the wrong return type. Required interface {} but the message creates interface {}")
            }
        } else {
            // there is no target interface in the protocol, this is a generic object-creating
            // function (likely wl_registry.bind), the newid arg will thus expand to (str, u32, obj)
            nid_idx += 2;
        }

        let version = version.unwrap_or_else(|| self.version());

        let ptr = msg.as_raw_c_in(|opcode, args| unsafe {
            assert!(
                args[nid_idx].o.is_null(),
                "Trying to use 'send_constructor' with a non-placeholder object."
            );
            ffi_dispatch!(
                WAYLAND_CLIENT_HANDLE,
                wl_proxy_marshal_array_constructor_versioned,
                self.ptr,
                opcode,
                args.as_mut_ptr(),
                J::c_interface(),
                version
            )
        });

        if destructor {
            // we need to destroy the proxy now
            if let Some(ref internal) = self.internal {
                internal.alive.store(false, Ordering::Release);
            }
            unsafe {
                ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_destroy, self.ptr);
            }
        }

        Ok(unsafe { NewProxyInner::from_c_ptr(ptr) })
    }

    pub(crate) fn equals(&self, other: &ProxyInner) -> bool {
        if !self.is_alive() {
            return false;
        }
        match (&self.internal, &other.internal) {
            (&Some(ref my_inner), &Some(ref other_inner)) => Arc::ptr_eq(my_inner, other_inner),
            (&None, &None) => self.ptr == other.ptr,
            _ => false,
        }
    }

    pub(crate) fn make_wrapper(&self, queue: &EventQueueInner) -> Result<ProxyInner, ()> {
        if !self.is_external() && !self.is_alive() {
            return Err(());
        }

        let wrapper_ptr;
        unsafe {
            wrapper_ptr = ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_create_wrapper, self.ptr);
            queue.assign_proxy(wrapper_ptr);
        }

        Ok(ProxyInner {
            internal: self.internal.clone(),
            ptr: wrapper_ptr,
            is_wrapper: true,
        })
    }

    pub(crate) fn child<I: Interface>(&self) -> NewProxyInner {
        let ptr =
            unsafe { ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_create, self.ptr, I::c_interface()) };
        NewProxyInner { ptr: ptr }
    }

    pub(crate) fn c_ptr(&self) -> *mut wl_proxy {
        self.ptr
    }

    pub(crate) unsafe fn from_c_ptr<I: Interface>(ptr: *mut wl_proxy) -> Self {
        if ptr.is_null() {
            return ProxyInner {
                internal: Some(Arc::new(ProxyInternal {
                    alive: AtomicBool::new(false),
                    user_data: UserData::empty(),
                })),
                ptr: ptr,
                is_wrapper: false,
            };
        }

        let is_managed = {
            ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_get_listener, ptr)
                == &::wayland_sys::RUST_MANAGED as *const u8 as *const _
        };
        let internal = if is_managed {
            let user_data =
                ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_get_user_data, ptr) as *mut ProxyUserData<I>;
            Some((*user_data).internal.clone())
        } else {
            None
        };
        ProxyInner {
            internal: internal,
            ptr: ptr,
            is_wrapper: false,
        }
    }

    pub(crate) unsafe fn from_c_display_wrapper(d: *mut wl_proxy) -> ProxyInner {
        ProxyInner {
            internal: None,
            ptr: d,
            is_wrapper: true,
        }
    }

    pub(crate) fn child_placeholder(&self) -> ProxyInner {
        ProxyInner {
            internal: Some(Arc::new(ProxyInternal {
                alive: AtomicBool::new(false),
                user_data: UserData::empty(),
            })),
            ptr: ::std::ptr::null_mut(),
            is_wrapper: false,
        }
    }
}

pub(crate) struct NewProxyInner {
    ptr: *mut wl_proxy,
}

impl NewProxyInner {
    pub(crate) unsafe fn implement<I: Interface, F>(
        self,
        implementation: F,
        user_data: UserData,
    ) -> ProxyInner
    where
        F: FnMut(I::Event, Proxy<I>) + 'static,
    {
        let new_user_data = Box::new(ProxyUserData::new(implementation, user_data));
        let internal = new_user_data.internal.clone();

        ffi_dispatch!(
            WAYLAND_CLIENT_HANDLE,
            wl_proxy_add_dispatcher,
            self.ptr,
            proxy_dispatcher::<I>,
            &::wayland_sys::RUST_MANAGED as *const _ as *const _,
            Box::into_raw(new_user_data) as *mut _
        );

        ProxyInner {
            internal: Some(internal),
            ptr: self.ptr,
            is_wrapper: false,
        }
    }

    pub(crate) fn c_ptr(&self) -> *mut wl_proxy {
        self.ptr
    }

    pub(crate) unsafe fn from_c_ptr(ptr: *mut wl_proxy) -> NewProxyInner {
        NewProxyInner { ptr: ptr }
    }
}

struct ProxyUserData<I: Interface> {
    internal: Arc<ProxyInternal>,
    implem: Option<Box<FnMut(I::Event, Proxy<I>)>>,
}

impl<I: Interface> ProxyUserData<I> {
    fn new<F>(implem: F, user_data: UserData) -> ProxyUserData<I>
    where
        F: FnMut(I::Event, Proxy<I>) + 'static,
    {
        ProxyUserData {
            internal: Arc::new(ProxyInternal::new(user_data)),
            implem: Some(Box::new(implem)),
        }
    }
}

unsafe extern "C" fn proxy_dispatcher<I: Interface>(
    _implem: *const c_void,
    proxy: *mut c_void,
    opcode: u32,
    _msg: *const wl_message,
    args: *const wl_argument,
) -> c_int
where
    I: Interface,
{
    let proxy = proxy as *mut wl_proxy;

    // We don't need to worry about panic-safeness, because if there is a panic,
    // we'll abort the process, so no access to corrupted data is possible.
    let ret = ::std::panic::catch_unwind(move || {
        // parse the message:
        let msg = I::Event::from_raw_c(proxy as *mut _, opcode, args)?;
        let must_destroy = msg.is_destructor();
        // create the proxy object
        let proxy_obj = ::Proxy::<I>::from_c_ptr(proxy);
        // retrieve the impl
        let user_data = ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_get_user_data, proxy);
        {
            let user_data = &mut *(user_data as *mut ProxyUserData<I>);
            let implem = user_data.implem.as_mut().unwrap();
            if must_destroy {
                user_data.internal.alive.store(false, Ordering::Release);
                ffi_dispatch!(WAYLAND_CLIENT_HANDLE, wl_proxy_destroy, proxy);
            }
            // call the impl
            implem(msg, proxy_obj);
        }
        if must_destroy {
            // final cleanup
            let _ = Box::from_raw(user_data as *mut ProxyUserData<I>);
        }
        Ok(())
    });
    // check the return status
    match ret {
        Ok(Ok(())) => return 0,
        Ok(Err(())) => {
            eprintln!(
                "[wayland-client error] Attempted to dispatch unknown opcode {} for {}, aborting.",
                opcode,
                I::NAME
            );
            ::libc::abort();
        }
        Err(_) => {
            eprintln!("[wayland-client error] A handler for {} panicked.", I::NAME);
            ::libc::abort()
        }
    }
}
