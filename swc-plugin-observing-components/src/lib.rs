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

    // Get the filename from metadata
    let filename = data
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap_or_default();
    
    // Check exclusion patterns
    let is_excluded = wrap_components_with_observer::should_exclude(&filename, &config.exclude);
    
    // Add debug output for path matching
    #[cfg(debug_assertions)]
    {
        eprintln!("File: {}", filename);
        eprintln!("Exclude patterns: {:?}", config.exclude);
        eprintln!("Is excluded: {}", is_excluded);
    }
    
    // Check if we should process this file:
    // 1. Skip node_modules
    // 2. Check against exclude patterns
    let should_process = !filename.contains("node_modules") && !is_excluded;

    if !should_process {
        return program;
    }

    program.apply(wrap_components_with_observer::observer_transform(config))
}