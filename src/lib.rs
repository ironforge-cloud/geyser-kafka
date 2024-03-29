// Copyright 2022 Blockdaemon Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// The generated code (via prost) does not add `Eq` derive which causes clippy warnings since Rust
// v1.63. This is the only way to suppress those since we cannot edit the generated file.
#![allow(clippy::derive_partial_eq_without_eq)]

use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin;

mod allowlist;
mod cluster;
mod config;
mod env_config;
mod errors;
mod event;
pub mod events;
mod filter;
mod plugin;
mod prom;
mod publisher;
mod system_programs;
mod version;

pub use {
    cluster::Cluster,
    config::{Config, Producer},
    env_config::EnvConfig,
    errors::*,
    event::*,
    filter::Filter,
    plugin::KafkaPlugin,
    prom::PrometheusService,
    publisher::{serializable_events, FilteringPublisher},
    system_programs::*,
};

#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// This function returns a pointer to the Kafka Plugin box implementing trait GeyserPlugin.
///
/// The Solana validator and this plugin must be compiled with the same Rust compiler version and Solana core version.
/// Loading this plugin with mismatching versions is undefined behavior and will likely cause memory corruption.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = KafkaPlugin::new();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}
