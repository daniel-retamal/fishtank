use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConfigSpec {
    pub name: &'static str,
    pub default: &'static str,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct PartConfig {
    values: BTreeMap<String, String>,
}

impl PartConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, spec: &ConfigSpec, value: &str) -> bool {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return self.values.remove(spec.name).is_some();
        }
        self.values
            .insert(spec.name.to_string(), trimmed.to_string())
            .as_deref()
            != Some(trimmed)
    }

    pub fn text(&self, spec: &ConfigSpec) -> &str {
        self.values
            .get(spec.name)
            .map_or(spec.default, |value| value.as_str())
    }

    pub fn number(&self, spec: &ConfigSpec) -> u32 {
        self.text(spec)
            .parse()
            .or_else(|_| spec.default.parse())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET: ConfigSpec = ConfigSpec {
        name: "target",
        default: "heaviest",
    };
    const THRESHOLD: ConfigSpec = ConfigSpec {
        name: "threshold",
        default: "1000",
    };

    #[test]
    fn an_unset_field_reads_the_value_the_part_shipped_with() {
        let config = PartConfig::new();
        assert_eq!(config.text(&TARGET), "heaviest");
        assert_eq!(config.number(&THRESHOLD), 1000);
    }

    #[test]
    fn a_set_field_replaces_the_default() {
        let mut config = PartConfig::new();
        assert!(config.set(&TARGET, "richest mutantfish"));
        assert_eq!(config.text(&TARGET), "richest mutantfish");
    }

    #[test]
    fn setting_the_same_value_twice_changes_nothing() {
        let mut config = PartConfig::new();
        config.set(&THRESHOLD, "1500");
        assert!(!config.set(&THRESHOLD, "  1500  "), "already configured");
        assert_eq!(config.number(&THRESHOLD), 1500);
    }

    #[test]
    fn clearing_a_field_hands_it_back_to_the_default() {
        let mut config = PartConfig::new();
        config.set(&TARGET, "lightest");
        assert!(config.set(&TARGET, "   "));
        assert_eq!(config.text(&TARGET), "heaviest");
        assert!(!config.set(&TARGET, ""), "nothing left to clear");
    }

    #[test]
    fn a_number_field_holding_words_falls_back_to_the_default() {
        let mut config = PartConfig::new();
        config.set(&THRESHOLD, "plenty");
        assert_eq!(config.number(&THRESHOLD), 1000);
    }

    #[test]
    fn zero_is_a_number_a_player_may_mean() {
        let mut config = PartConfig::new();
        config.set(&THRESHOLD, "0");
        assert_eq!(config.number(&THRESHOLD), 0);
    }
}
