use crate::entities::hardware_logs::{HardwareLog, MatchingHardwareLog as MatchingHardwareEntity};
use crate::models::privileges::Privileges;
use hashbrown::{HashMap, HashSet};
use rust_decimal::prelude::ToPrimitive;
use tracing::error;

#[derive(Debug, Default)]
pub struct UserAggregateHardware {
    pub info: AggregateHardwareInfo,
    pub total_occurrences: i64,
    pub has_activated_hardware: bool,
}

#[derive(Debug)]
pub struct AggregateHardwareMatch {
    pub user_id: i64,
    pub username: String,
    pub user_privileges: Privileges,
    pub info: AggregateHardwareInfo,
    pub total_occurrences: i64,
    pub has_activated_hardware: bool,
}

#[derive(Debug, Default)]
pub struct AggregateHardwareInfo {
    pub mac_hashes: HashSet<String>,
    pub unique_ids: HashSet<String>,
    pub disk_ids: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct AggregateMatchingHardwareResult {
    pub total_hardware_matches: i64,
    pub user_matches: HashMap<i64, AggregateHardwareMatch>,
}

impl UserAggregateHardware {
    /// Only use when entries is not empty
    pub fn from(entries: Vec<HardwareLog>) -> Self {
        let mut aggregated = Self::default();
        if entries.is_empty() {
            error!("Unexpected empty hardware logs");
            return aggregated;
        }

        for entry in entries {
            let entry_occurrences = match entry.occurencies.to_i64() {
                Some(occurrences) => occurrences,
                None => {
                    error!("Failed casting hardware log occurencies to i64");
                    continue;
                }
            };
            aggregated.info.add_entry([
                entry.adapters_md5,
                entry.uninstall_md5,
                entry.disk_signature_md5,
            ]);
            aggregated.total_occurrences += entry_occurrences;
            aggregated.has_activated_hardware |= entry.activated;
        }
        aggregated
    }
}

impl AggregateHardwareMatch {
    pub fn aggregate_by_user(
        hw_matches: Vec<MatchingHardwareEntity>,
    ) -> AggregateMatchingHardwareResult {
        let mut aggregated = AggregateMatchingHardwareResult::default();
        for hw_log_entry in hw_matches {
            let entry_occurrences = match hw_log_entry.occurencies.to_i64() {
                Some(occurrences) => occurrences,
                None => {
                    error!("Failed casting hardware log occurencies to i64");
                    continue;
                }
            };
            aggregated.total_hardware_matches += entry_occurrences;
            let identifiers = [
                hw_log_entry.adapters_md5,
                hw_log_entry.uninstall_md5,
                hw_log_entry.disk_signature_md5,
            ];

            match aggregated.user_matches.get_mut(&hw_log_entry.user_id) {
                Some(aggregated) => {
                    aggregated.info.add_entry(identifiers);
                    aggregated.total_occurrences += entry_occurrences;
                    aggregated.has_activated_hardware |= hw_log_entry.activated;
                }
                None => {
                    aggregated.user_matches.insert(
                        hw_log_entry.user_id,
                        Self {
                            user_id: hw_log_entry.user_id,
                            username: hw_log_entry.username,
                            user_privileges: Privileges::from_bits_retain(
                                hw_log_entry.user_privileges,
                            ),
                            info: AggregateHardwareInfo::new(identifiers),
                            total_occurrences: entry_occurrences,
                            has_activated_hardware: hw_log_entry.activated,
                        },
                    );
                }
            }
        }
        aggregated
    }
}

impl AggregateHardwareInfo {
    pub fn new([mac, unique_id, disk_id]: [String; 3]) -> Self {
        Self {
            mac_hashes: HashSet::from([mac]),
            unique_ids: HashSet::from([unique_id]),
            disk_ids: HashSet::from([disk_id]),
        }
    }

    pub fn add_entry(&mut self, [mac, unique_id, disk_id]: [String; 3]) {
        self.mac_hashes.insert(mac);
        self.unique_ids.insert(unique_id);
        self.disk_ids.insert(disk_id);
    }
}
