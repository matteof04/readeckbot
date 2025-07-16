/*
 * Copyright (c) 2025 Matteo Franceschini
 * All rights reserved.
 *
 * Use of this source code is governed by BSD-3-Clause-Clear
 * license that can be found in the LICENSE file
 */

use std::{collections::HashMap, path::Path};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserData {
    pub api_token: String,
    pub bot_marked: bool,
}

#[derive(Deserialize)]
pub struct Users {
    users: HashMap<u64, UserData>,
}

impl Users {
    pub fn load<P: AsRef<Path>>(path: P) -> Users {
        let data = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&data).unwrap()
    }

    pub fn find(&self, user_id: u64) -> Option<&UserData> {
        self.users.get(&user_id)
    }
}
