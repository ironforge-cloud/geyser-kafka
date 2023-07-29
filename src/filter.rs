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

use crate::allowlist::Allowlist;
use crate::EnvConfig;

pub struct Filter {
    program_allowlist: Allowlist,
}

impl Filter {
    pub fn new(config: &EnvConfig) -> Self {
        Self {
            program_allowlist: Allowlist::new_from_config(config).unwrap(),
        }
    }

    pub fn get_allowlist(&self) -> Allowlist {
        self.program_allowlist.clone()
    }

    pub fn wants_account_key(&self, account_key: &[u8]) -> bool {
        if self.program_allowlist.len() > 0 {
            self.program_allowlist.wants_program(account_key)
        } else {
            false
        }
    }
}
