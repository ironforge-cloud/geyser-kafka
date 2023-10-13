use log::debug;
use serde::Deserialize;
use simple_error::SimpleError;
use solana_program::pubkey::Pubkey;
use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::EnvConfig;

use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPluginError as PluginError, Result as PluginResult,
};

#[derive(Clone)]
pub struct Allowlist {
    /// List of programs to allow.
    list: Arc<Mutex<HashSet<[u8; 32]>>>,
    updater: Option<AllowlistUpdater>,
}

#[derive(Clone)]
pub struct AllowlistUpdater {
    /// Url to fetch allowlist from.
    http_url: String,
    /// Optional auth header to fetch allowlist with.
    http_auth: String,
    // http_updater_one is used to ensure that only one thread is fetching the allowlist from the
    // remote server at a time.
    http_is_updating: Arc<Mutex<bool>>,
    /// A slot in Solana is a fixed duration of time, currently set at 400 milliseconds, during
    /// which a validator has the opportunity to produce a block.
    /// Slots are sequential, meaning that they occur one after another in a linear fashion.
    /// We use slot updates as an indicator of how much time has passed.
    /// This slots interval determines how many slots to wait before updating the allowlist.
    slot_interval: u64,
}

impl AllowlistUpdater {
    fn is_updating(&self) -> bool {
        *self.http_is_updating.lock().unwrap()
    }

    fn needs_update(&self, slot: u64) -> bool {
        slot % self.slot_interval == 0
    }
}

#[derive(Deserialize, Debug)]
struct RemoteAllowlist {
    #[serde(rename = "result")]
    program_allowlist: Vec<String>,
}

impl Allowlist {
    pub fn len(&self) -> usize {
        let list = self.list.lock().unwrap();
        list.len()
    }
    pub fn new_from_config(config: &EnvConfig) -> PluginResult<Self> {
        match config {
            EnvConfig::Kafka(config) => {
                // Users can provide a URL to fetch the allow list from
                if !config.program_allowlist_url.is_empty() {
                    assert!(
                        config.program_allowlist_slot_interval > 0,
                        "program_allowlist_slot_interval must be greater than 0"
                    );
                    let mut this = Self::new_from_http(
                        &config.program_allowlist_url.clone(),
                        &config.program_allowlist_auth.clone(),
                        config.program_allowlist_slot_interval,
                    )
                    .unwrap();

                    if !config.program_allowlist.is_empty() {
                        // The allowlist to start with can be defined in the config
                        this.push_vec(config.program_allowlist.clone());
                    } else {
                        // Otherwise, fetch it from the provided url
                        this.init_list_from_http_blocking(
                            &config.program_allowlist_url,
                            &config.program_allowlist_auth,
                        )?;
                    }

                    return Ok(this);
                }

                // If no url is provided, then the allowlist needs to be defined in the config
                if config.program_allowlist.is_empty() {
                    return Err(PluginError::Custom(Box::new(SimpleError::new(
                        "Need to provide a program allowlist provided or a URL to fetch it from"
                            .to_string(),
                    ))));
                }

                Self::new_from_vec(config.program_allowlist.clone())
            }
            EnvConfig::Local(config) => Self::new_from_vec(config.program_allowlist.clone()),
        }
    }

    /// new_from_vec creates a new Allowlist from a vector of program ids.
    pub fn new_from_vec(program_allowlist: Vec<String>) -> PluginResult<Self> {
        let program_allowlist = program_allowlist
            .iter()
            .flat_map(|p| Pubkey::from_str(p).ok().map(|p| p.to_bytes()))
            .collect();
        Ok(Self {
            list: Arc::new(Mutex::new(program_allowlist)),
            updater: None,
        })
    }

    fn push_vec(&mut self, program_allowlist: Vec<String>) {
        let mut list = self.list.lock().unwrap();
        for pubkey in program_allowlist {
            let pubkey = Pubkey::from_str(&pubkey);
            if pubkey.is_err() {
                continue;
            }
            list.insert(pubkey.unwrap().to_bytes());
        }
    }

    // fetch_remote_allowlist fetches the allowlist from the remote server,
    // and returns a HashSet of program ids.
    fn fetch_remote_allowlist(url: &str, auth: &str) -> PluginResult<HashSet<[u8; 32]>> {
        let mut program_allowlist = HashSet::new();

        let mut req = ureq::get(url);
        if !auth.is_empty() {
            req = req.set("Authorization", auth);
        }

        match req.call() {
            Ok(response) => {
                if response.status() != 200 {
                    return Err(PluginError::Custom(Box::new(
                        simple_error::SimpleError::new(format!(
                            "Failed to fetch allowlist from remote server: status {}",
                            response.status()
                        )),
                    )));
                }
                /* the server returned a 200 OK response */
                let body = response.into_string();
                if body.is_err() {
                    return Err(PluginError::Custom(Box::new(
                        simple_error::SimpleError::new(format!(
                            "Failed to fetch allowlist from remote server: {}",
                            body.err().unwrap()
                        )),
                    )));
                }
                // parse the response body as json:
                let raw = serde_json::from_str(&body.unwrap());
                if raw.is_err() {
                    return Err(PluginError::Custom(Box::new(
                        simple_error::SimpleError::new(format!(
                            "Failed to fetch allowlist from remote server: {}",
                            raw.err().unwrap()
                        )),
                    )));
                }
                let list: RemoteAllowlist = raw.unwrap();
                for pubkey in list.program_allowlist {
                    let pubkey = Pubkey::from_str(&pubkey);
                    if pubkey.is_err() {
                        continue;
                    }
                    program_allowlist.insert(pubkey.unwrap().to_bytes());
                }
            }
            Err(ureq::Error::Status(code, _response)) => {
                return Err(PluginError::Custom(Box::new(
                    simple_error::SimpleError::new(format!(
                        "Failed to fetch allowlist from remote server: status {code}"
                    )),
                )));
            }
            Err(e) => {
                return Err(PluginError::Custom(Box::new(
                    simple_error::SimpleError::new(format!(
                        "Failed to fetch allowlist from remote server: status {e}"
                    )),
                )));
            }
        }

        Ok(program_allowlist)
    }

    // Updates the allowlist from a remote URL without blocking the main thread.
    pub fn update_from_http_non_blocking(&self) {
        let updater = match &self.updater {
            Some(updater) if !updater.is_updating() => updater,
            _ => return,
        };

        let list = self.list.clone();
        let url = updater.http_url.clone();
        let auth_header = updater.http_auth.clone();

        let is_updating = updater.http_is_updating.clone();
        *is_updating.lock().unwrap() = true;

        std::thread::spawn(move || {
            let thread_id = std::thread::current().id();
            debug!("Updating remote allowlist, thread {:?}", thread_id);
            let program_allowlist = Self::fetch_remote_allowlist(&url, &auth_header);
            if program_allowlist.is_err() {
                *is_updating.lock().unwrap() = false;
                return;
            }

            let mut list = list.lock().unwrap();
            *list = program_allowlist.unwrap();
            *is_updating.lock().unwrap() = false;

            debug!("Updated remote allowlist, thread {:?}", thread_id);
        });
    }

    /// Initializes this allow list with data obtained from the given URL synchronously.
    pub fn init_list_from_http_blocking(&self, url: &str, auth: &str) -> PluginResult<()> {
        let program_allowlist = Self::fetch_remote_allowlist(url, auth)?;

        let mut list = self.list.lock().unwrap();
        *list = program_allowlist;

        Ok(())
    }

    fn needs_remote_update(&self, slot: u64) -> bool {
        match &self.updater {
            Some(updater) => updater.needs_update(slot),
            None => false,
        }
    }

    pub fn update_from_http_if_needed_async(&mut self, slot: u64) {
        if self.needs_remote_update(slot) {
            self.update_from_http_non_blocking();
        }
    }

    pub fn new_from_http(url: &str, auth_header: &str, slot_interval: u64) -> PluginResult<Self> {
        let program_allowlist = Self::fetch_remote_allowlist(url, auth_header);
        if program_allowlist.is_err() {
            return Err(program_allowlist.err().unwrap());
        }
        let updater = AllowlistUpdater {
            http_url: url.to_string(),
            http_auth: auth_header.to_string(),
            http_is_updating: Arc::new(Mutex::new(false)),
            slot_interval,
        };
        Ok(Self {
            list: Arc::new(Mutex::new(program_allowlist.unwrap())),
            updater: Some(updater),
        })
    }

    pub fn wants_program(&self, program: &[u8]) -> bool {
        let key = match <&[u8; 32]>::try_from(program) {
            Ok(key) => key,
            _ => return true,
        };
        let list = self.list.lock().unwrap();
        // If we were given an empty list and we're not ever updating it then we assume
        // that we want all programs.
        // However if updating the list failed and it is empty for that reason we prefer
        // to not include any programs instead of flooding kafka.
        (self.updater.is_none() && list.is_empty()) || list.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::env_config::EnvConfigKafka;

    use super::*;
    #[test]
    fn test_allowlist_from_vec() {
        let config = EnvConfig::Kafka(EnvConfigKafka {
            program_allowlist: vec![
                "Sysvar1111111111111111111111111111111111111".to_string(),
                "Vote111111111111111111111111111111111111111".to_string(),
            ],
            ..EnvConfigKafka::default()
        });

        let allowlist = Allowlist::new_from_vec(config.program_allowlist().to_vec()).unwrap();
        assert_eq!(allowlist.len(), 2);

        assert!(allowlist.wants_program(
            &Pubkey::from_str("Sysvar1111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));
        assert!(allowlist.wants_program(
            &Pubkey::from_str("Vote111111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));
        // negative test
        assert!(!allowlist.wants_program(
            &Pubkey::from_str("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
                .unwrap()
                .to_bytes()
        ));
    }

    #[test]
    fn test_allowlist_create_from_http() {
        let _m = mockito::mock("GET", "/allowlist.txt")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("{\"result\":[\"Sysvar1111111111111111111111111111111111111\",\"Vote111111111111111111111111111111111111111\"]}")
            .create();

        let config = EnvConfig::Kafka(EnvConfigKafka {
            program_allowlist_url: [mockito::server_url(), "/allowlist.txt".to_string()].join(""),
            program_allowlist_slot_interval: 5,
            program_allowlist: vec!["WormT3McKhFJ2RkiGpdw9GKvNCrB2aB54gb2uV9MfQC".to_string()],
            ..EnvConfigKafka::default()
        });


        let allowlist = Allowlist::new_from_config(&config).unwrap();
        assert_eq!(allowlist.len(), 3);

        assert!(allowlist.wants_program(
            &Pubkey::from_str("WormT3McKhFJ2RkiGpdw9GKvNCrB2aB54gb2uV9MfQC")
                .unwrap()
                .to_bytes()
        ));
        assert!(allowlist.wants_program(
            &Pubkey::from_str("Sysvar1111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));
        assert!(allowlist.wants_program(
            &Pubkey::from_str("Vote111111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));
        assert!(!allowlist.wants_program(
            &Pubkey::from_str("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
                .unwrap()
                .to_bytes()
        ));
    }

    fn wait_for_update_completion(allowlist: &Allowlist) {
        assert!(allowlist.updater.as_ref().unwrap().is_updating());
        while allowlist.updater.as_ref().unwrap().is_updating() {
            sleep(Duration::from_millis(100));
        }
    }

    #[test]
    fn test_allowlist_needs_remote_upate() {
        let _m = mockito::mock("GET", "/allowlist.txt")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("{\"result\":[]}")
            .create();
        
        let config = EnvConfig::Kafka(EnvConfigKafka {
            program_allowlist_url: [mockito::server_url(), "/allowlist.txt".to_string()].join(""),
            program_allowlist_slot_interval: 5,
            ..EnvConfigKafka::default()
        });

        let allowlist = Allowlist::new_from_config(&config).unwrap();
        assert!(!allowlist.needs_remote_update(1));
        assert!(allowlist.needs_remote_update(5));
        assert!(!allowlist.needs_remote_update(9));
        assert!(allowlist.needs_remote_update(10));
    }

    #[test]
    fn test_allowlist_remote_upate_if_needed() {
        let _m = mockito::mock("GET", "/allowlist.txt")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("{\"result\":[]}")
            .create();

        let config = EnvConfig::Kafka(EnvConfigKafka {
            program_allowlist_url: [mockito::server_url(), "/allowlist.txt".to_string()].join(""),
            program_allowlist_slot_interval: 5,
            ..EnvConfigKafka::default()
        });

        let mut allowlist = Allowlist::new_from_config(&config).unwrap();
        // 1. Initial allowlist is empty
        assert_eq!(allowlist.len(), 0);

        // 2. Allowlist is updated remotely
        let _m = mockito::mock("GET", "/allowlist.txt")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("{\"result\":[\"Sysvar1111111111111111111111111111111111111\",\"Vote111111111111111111111111111111111111111\"]}")
            .create();

        // 3. Update if needed with slot not causing update
        allowlist.update_from_http_if_needed_async(7);
        assert!(!allowlist.updater.as_ref().unwrap().is_updating());
        assert_eq!(allowlist.len(), 0);
        assert!(!allowlist.wants_program(
            &Pubkey::from_str("Sysvar1111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));

        
        // 4. Update if needed with slot causing update
        allowlist.update_from_http_if_needed_async(10);
        wait_for_update_completion(&allowlist);
        assert_eq!(allowlist.len(), 2);
        assert!(allowlist.wants_program(
            &Pubkey::from_str("Sysvar1111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));

        // 5. Allowlist is updated remotely again
        let _m = mockito::mock("GET", "/allowlist.txt")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("{\"result\":[\"Sysvar1111111111111111111111111111111111111\",\"Vote111111111111111111111111111111111111111\", \"WormT3McKhFJ2RkiGpdw9GKvNCrB2aB54gb2uV9MfQC\"]}")
            .create();

        // 6. Update if needed with another slot not causing update
        allowlist.update_from_http_if_needed_async(13);
        assert!(!allowlist.updater.as_ref().unwrap().is_updating());
        assert_eq!(allowlist.len(), 2);
        assert!(allowlist.wants_program(
            &Pubkey::from_str("Sysvar1111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));
        assert!(!allowlist.wants_program(
            &Pubkey::from_str("WormT3McKhFJ2RkiGpdw9GKvNCrB2aB54gb2uV9MfQC")
                .unwrap()
                .to_bytes()
        ));

        // 7. Update if needed with another slot not causing update
        allowlist.update_from_http_if_needed_async(15);
        wait_for_update_completion(&allowlist);
        assert!(!allowlist.updater.as_ref().unwrap().is_updating());
        assert_eq!(allowlist.len(), 3);
        assert!(allowlist.wants_program(
            &Pubkey::from_str("Sysvar1111111111111111111111111111111111111")
                .unwrap()
                .to_bytes()
        ));
        assert!(allowlist.wants_program(
            &Pubkey::from_str("WormT3McKhFJ2RkiGpdw9GKvNCrB2aB54gb2uV9MfQC")
                .unwrap()
                .to_bytes()
        ));
    }

}
