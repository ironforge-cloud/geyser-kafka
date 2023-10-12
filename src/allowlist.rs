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
    http_updater_one: Arc<Mutex<()>>,
    /// A slot in Solana is a fixed duration of time, currently set at 400 milliseconds, during
    /// which a validator has the opportunity to produce a block.
    /// Slots are sequential, meaning that they occur one after another in a linear fashion.
    /// We use slot updates as an indicator of how much time has passed.
    /// This slots interval determines how many slots to wait before updating the allowlist.
    slot_interval: u64,
}

impl AllowlistUpdater {
    fn is_updating(&self) -> bool {
        let v = self.http_updater_one.try_lock();
        v.is_err()
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
    pub fn update_from_http_non_blocking(&self, slot: u64) {
        let updater = match &self.updater {
            Some(updater) if !updater.is_updating() => updater,
            _ => return,
        };
        let _once = updater.http_updater_one.lock().unwrap();

        // While we were aquiring the lock another thread may have updated the list
        // and thus we don't need to do that again.
        if updater.needs_update(slot) {
            let list = self.list.clone();
            let url = updater.http_url.clone();
            let auth_header = updater.http_auth.clone();
            std::thread::spawn(move || {
                let thread_id = std::thread::current().id();
                debug!("Updating remote allowlist, thread {:?}", thread_id);
                let program_allowlist = Self::fetch_remote_allowlist(&url, &auth_header);
                if program_allowlist.is_err() {
                    return;
                }

                let mut list = list.lock().unwrap();
                *list = program_allowlist.unwrap();

                debug!("Updated remote allowlist, thread {:?}", thread_id);
            });
        }
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
            self.update_from_http_non_blocking(slot);
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
            http_updater_one: Arc::new(Mutex::new(())),
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
        list.is_empty() || list.contains(key)
    }
}

/*
#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::env_config::EnvConfigKafka;

    use super::*;
    #[test]
    fn test_allowlist_from_vec() {
        let config = EnvConfig::Kafka(EnvConfigKafka {
            program_allowlist: vec![
                "Sysvar1111111111111111111111111111111111111".to_owned(),
                "Vote111111111111111111111111111111111111111".to_owned(),
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
    fn test_allowlist_from_http() {
        // create fake http server
        let _m = mockito::mock("GET", "/allowlist.txt")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("{\"result\":[\"Sysvar1111111111111111111111111111111111111\",\"Vote111111111111111111111111111111111111111\"]}")
            .create();

        let config = EnvConfig::Kafka(EnvConfigKafka {
            program_allowlist_url: [mockito::server_url(), "/allowlist.txt".to_owned()].join(""),
            program_allowlist_expiry_sec: 3,
            program_allowlist: vec!["WormT3McKhFJ2RkiGpdw9GKvNCrB2aB54gb2uV9MfQC".to_owned()],
            ..EnvConfigKafka::default()
        });

        let mut allowlist = Allowlist::new_from_config(&config).unwrap();
        let now = std::time::Instant::now();
        assert_eq!(allowlist.len(), 3);
        assert!(!allowlist.is_remote_allowlist_expired(&now));

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
        // negative test
        assert!(!allowlist.wants_program(
            &Pubkey::from_str("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
                .unwrap()
                .to_bytes()
        ));

        {
            let _u = mockito::mock("GET", "/allowlist.txt")
                .with_status(200)
                .with_header("content-type", "text/plain")
                .with_body("{\"result\":[\"9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin\"]}")
                .create();
            allowlist.update_from_http().unwrap();
            assert_eq!(allowlist.len(), 1);

            assert!(allowlist.wants_program(
                &Pubkey::from_str("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
                    .unwrap()
                    .to_bytes()
            ));
        }
        {
            let _u = mockito::mock("GET", "/allowlist.txt")
                .with_status(200)
                .with_header("content-type", "text/plain")
                .with_body("{\"result\":[]}")
                .create();
            let last_updated = allowlist.get_last_updated();
            println!("last_updated: {last_updated:?}");
            allowlist.update_from_http().unwrap();
            assert_ne!(allowlist.get_last_updated(), last_updated);
            assert_eq!(allowlist.len(), 0);
            println!("last_updated: {:?}", allowlist.get_last_updated());

            assert!(allowlist.wants_program(
                &Pubkey::from_str("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
                    .unwrap()
                    .to_bytes()
            ));
        }
        {
            // async
            let _u = mockito::mock("GET", "/allowlist.txt")
                .with_status(200)
                .with_header("content-type", "text/plain")
                .with_body("{\"result\":[\"Sysvar1111111111111111111111111111111111111\",\"Vote111111111111111111111111111111111111111\"]}")
                .create();

            let last_updated = allowlist.get_last_updated();
            allowlist.update_from_http_non_blocking(&last_updated);
            // the values should be the same because it returns immediately
            // before the async task completes
            assert_eq!(allowlist.get_last_updated(), last_updated);
            assert_eq!(allowlist.len(), 0);

            // sleep for 100 milliseconds to allow the async task to complete
            thread::sleep(std::time::Duration::from_millis(100));
            let now = std::time::Instant::now();

            assert!(!allowlist.is_remote_allowlist_expired(&now));

            assert_eq!(allowlist.len(), 2);
            assert_ne!(allowlist.get_last_updated(), last_updated);

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

            // Claim we are 3 seconds in the future
            let now = std::time::Instant::now()
                .checked_add(Duration::from_secs(3))
                .unwrap();
            assert!(allowlist.is_remote_allowlist_expired(&now));
        }
    }
}
*/
