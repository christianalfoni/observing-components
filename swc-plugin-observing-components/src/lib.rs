#![allow(clippy::not_unsafe_ptr_arg_deref)]
use swc_core::{
    ecma::ast::Program,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};


#[plugin_transform]
fn swc_plugin(program: Program, data: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<Option<wrap_components_with_observer::Config>>(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config for react-remove-properties"),
    )
    .expect("invalid packages")
    .unwrap();

    program.apply(wrap_components_with_observer::observer_transform(config))
}