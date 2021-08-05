macro_rules! protocol {
    ($name:ident) => ({
        #[allow(deprecated)]
        #[inline(always)]
        fn get_protocol(name: &str) -> Option<&'static objc::runtime::Protocol> {
            unsafe {
                #[cfg_attr(feature = "cargo-clippy", allow(replace_consts))]
                static PROTOCOL: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::ATOMIC_USIZE_INIT;
                // `Relaxed` should be fine since `objc_getClass` is thread-safe.
                let ptr = PROTOCOL.load(::std::sync::atomic::Ordering::Relaxed) as *const objc::runtime::Protocol;
                if ptr.is_null() {
                    let cls = objc::runtime::objc_getProtocol(name.as_ptr() as *const _);
                    PROTOCOL.store(cls as usize, ::std::sync::atomic::Ordering::Relaxed);
                    if cls.is_null() { None } else { Some(&*cls) }
                } else {
                    Some(&*ptr)
                }
            }
        }

        match get_protocol(concat!(stringify!($name), '\0')) {
            Some(cls) => cls,
            None => panic!("Protocol with name {} could not be found", stringify!($name)),
        }
    })
}

macro_rules! cascade_binding {
    ($arg_name:ident) => {
        $arg_name
    };
    ($arg_name:ident, $binding:ident) => {
        $binding
    };
}

// Get the selector as a Sel
macro_rules! method_sel {
    ($fn_name:ident) => {
        sel!($fn_name)
    };
    ($fn_name:ident, $arg0_name:ident) => {
        sel!($fn_name:)
    };
    ($fn_name:ident, $arg0_name:ident, $($arg_name:ident),+) => {
        sel!($fn_name: $($arg_name:)+)
    };
}

/// Define one or more Objective-C classes
macro_rules! def_class {
    ($(
        $(#[$outer:meta])*
        $vis:vis class $name:ident$(<$($generic:ident$(: $($generic_lt:lifetime +)? $generic_constraint:path)?),+>)?: $superclass:ident$(, $protocol:ident)* {
            $(
                ivar $ivar_name:ident: $ivar_type:ty;
            )*
            $(
                fn $fn_name:ident($this:ident$(, $arg_name:ident$( $binding:ident)?: $arg_type:ty)*) $(-> $ret_type:ty)? $body:block
            )*
        }
    )+) => {
        paste! {$(
            $(#[$outer])*
            $vis struct $name$(<$($generic$(: $($generic_lt +)? $generic_constraint)?),+>)*($($(::std::marker::PhantomData<$generic>),+)*);

            impl$(<$($generic$(: $($generic_lt +)? $generic_constraint)?),+>)* $name$(<$($generic),+>)* {
                $(
                    #[doc(hidden)]
                    #[allow(dead_code, non_snake_case)]
                    extern fn [<$fn_name $($arg_name)*>](
                        $this: &mut ::objc::runtime::Object,
                        _: ::objc::runtime::Sel
                        $(, cascade_binding!($arg_name$(, $binding)*): $arg_type)*
                    ) $(-> $ret_type)* $body
                )*

                pub fn class() -> &'static ::objc::runtime::Class {
                    static mut CLASS: *const ::objc::runtime::Class = ::std::ptr::null();
                    static REGISTER_ONCE: ::std::sync::Once = ::std::sync::Once::new();
                    REGISTER_ONCE.call_once(|| unsafe {
                        let mut decl = ::objc::declare::ClassDecl::new(::std::any::type_name::<Self>(), class!($superclass)).unwrap();
                        $(
                            decl.add_method(method_sel!($fn_name$(, $arg_name)*), Self::[<$fn_name $($arg_name)*>] as extern fn(&mut ::objc::runtime::Object, ::objc::runtime::Sel$(, $arg_type)*) $(-> $ret_type)*);
                        )*

                        $({
                            let proto = protocol!($protocol);
                            decl.add_protocol(proto as _);
                        })*

                        $(
                            decl.add_ivar::<$ivar_type>(stringify!($ivar_name));
                        )*

                        CLASS = decl.register() as _;
                    });

                    unsafe { &*CLASS }
                }
            }
        )+}
    };
}
