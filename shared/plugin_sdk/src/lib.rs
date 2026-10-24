mod error;
pub mod params;
pub mod router;
mod wire;

// forwarding PluginRouter
pub use router::PluginRouter;

pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use abi_stable;
    pub use plugin_interface;
}

// pub struct PluginMeta {
//     pub name: String,
//     pub uuid: String,
//     pub author: String,
//     pub description: String,
// }

#[macro_export]
macro_rules! export_plugin {
    (
        meta: PluginMeta {
            name: $name:expr,
            uuid: $uuid:expr,
            author: $author:expr,
            description: $desc:expr $(,)?,
        },
        router: $build_router:expr $(,)?,
        $( on_init: $on_init:expr $(,)? )?
    ) => {
        use $crate::__private::abi_stable::prefix_type::PrefixTypeTrait;
        use $crate::__private::abi_stable::std_types::{RResult, RString, RVec, Tuple2};
        use $crate::__private::plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};
        static ROUTER: std::sync::OnceLock<$crate::router::PluginRouter> =
            std::sync::OnceLock::new();

        #[::abi_stable::sabi_extern_fn]
        pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
            let router = ROUTER.get_or_init($build_router);
            let mut info = RVec::new();
            $( $on_init(info); )?

            info.push(Tuple2(RString::from("name"), RString::from($name)));
            info.push(Tuple2(RString::from("UUID"), RString::from($uuid)));
            info.push(Tuple2(RString::from("description"), RString::from($desc)));
            info.push(Tuple2(RString::from("author"), RString::from($author)));
            info.push(Tuple2(
                RString::from("function"),
                RString::from(router.method_names().join("/")),
            ));
            info.push(Tuple2(
                RString::from("sdk_version"),
                RString::from($crate::SDK_VERSION),
            ));

            RResult::ROk(info)
        }

        #[::abi_stable::sabi_extern_fn]
        extern "C" fn handle_message(msg: RString) -> RString {
            let router = ROUTER.get_or_init($build_router);
            RString::from(router.handle_raw(msg.as_str()))
        }

        #[::abi_stable::export_root_module]
        pub fn get_library() -> PluginRoot_Ref {
            PluginRoot {
                plugin: PluginI {
                    init,
                    handle_message,
                }
                .leak_into_prefix(),
            }
            .leak_into_prefix()
        }
    };
}
