#![allow(clippy::not_unsafe_ptr_arg_deref)]
use swc_core::{
    ecma::ast::Program,
    common::plugin::metadata::TransformPluginMetadataContextKind,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

#[plugin_transform]
fn swc_plugin(program: Program, data: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<Option<wrap_components_with_observer::Config>>(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config for observing-components"),
    )
    .expect("invalid plugin config")
    .unwrap();

    // Get the filename from metadata and check if we should process it
    let should_process = data
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .map(|filename| !filename.contains("node_modules"))
        .unwrap_or(true);

    if !should_process {
        return program;
    }

    program.apply(wrap_components_with_observer::observer_transform(config))
}