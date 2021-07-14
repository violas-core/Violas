// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub const DEPENDENCY: &str = "dependency";
pub const DEPENDENCY_SHORT: &str = "d";

pub const SENDER: &str = "sender";
pub const SENDER_SHORT: &str = "s";

pub const OUT_DIR: &str = "out-dir";
pub const OUT_DIR_SHORT: &str = "o";
pub const DEFAULT_OUTPUT_DIR: &str = "build";

pub const NO_SHADOW: &str = "no-shadow";
pub const NO_SHADOW_SHORT: &str = "S";

pub const SOURCE_MAP: &str = "source-map";
pub const SOURCE_MAP_SHORT: &str = "m";

pub const TEST: &str = "test";
pub const TEST_SHORT: &str = "t";

pub const COLOR_MODE_ENV_VAR: &str = "COLOR_MODE";

pub fn read_env_var(v: &str) -> String {
    std::env::var(v)
        .unwrap_or_else(|_| "".into())
        .to_uppercase()
}

pub fn read_bool_env_var(v: &str) -> bool {
    let val = read_env_var(v);
    val == "1" || val == "TRUE"
}
