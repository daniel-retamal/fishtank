use std::borrow::Cow;
use std::collections::BTreeMap;

const SPACE: char = ' ';

pub trait Wires {
    fn level(&self, channel: &str) -> bool;
    fn drive(&mut self, channel: &str, level: bool);
}

#[derive(Clone, Default)]
pub struct ChannelRegistry {
    levels: BTreeMap<String, bool>,
    pending: BTreeMap<String, Option<bool>>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normalize(name: &str) -> Option<Cow<'_, str>> {
        if Self::is_normal(name) {
            return Some(Cow::Borrowed(name));
        }
        let trimmed: String = name.split_whitespace().collect::<Vec<_>>().join(" ");
        if trimmed.is_empty() {
            return None;
        }
        Some(Cow::Owned(trimmed.to_ascii_lowercase()))
    }

    fn is_normal(name: &str) -> bool {
        let mut after_space = true;
        for character in name.chars() {
            let space = character == SPACE;
            let odd_space = character.is_whitespace() && !space;
            if character.is_ascii_uppercase() || odd_space || (space && after_space) {
                return false;
            }
            after_space = space;
        }
        !after_space
    }

    pub fn register(&mut self, name: &str) -> Option<String> {
        let key = Self::normalize(name)?.into_owned();
        self.levels.entry(key.clone()).or_insert(false);
        Some(key)
    }

    pub fn contains(&self, name: &str) -> bool {
        Self::normalize(name).is_some_and(|key| self.levels.contains_key(key.as_ref()))
    }

    pub fn level(&self, name: &str) -> bool {
        Self::normalize(name)
            .and_then(|key| self.levels.get(key.as_ref()).copied())
            .unwrap_or(false)
    }

    pub fn set_level(&mut self, name: &str, level: bool) -> bool {
        let Some(key) = Self::normalize(name) else {
            return false;
        };
        Self::store(&mut self.levels, key, level);
        true
    }

    pub fn drive(&mut self, name: &str, level: bool) -> bool {
        let Some(key) = Self::normalize(name) else {
            return false;
        };
        if let Some(merged) = self.pending.get_mut(key.as_ref()) {
            *merged = Some(merged.unwrap_or(false) | level);
            return true;
        }
        self.pending.insert(key.into_owned(), Some(level));
        true
    }

    pub fn commit(&mut self) {
        for (name, driven) in &mut self.pending {
            let Some(level) = driven.take() else {
                continue;
            };
            Self::store(&mut self.levels, Cow::Borrowed(name.as_str()), level);
        }
    }

    fn store(levels: &mut BTreeMap<String, bool>, key: Cow<'_, str>, level: bool) {
        if let Some(held) = levels.get_mut(key.as_ref()) {
            *held = level;
            return;
        }
        levels.insert(key.into_owned(), level);
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.levels.keys().map(|k| k.as_str())
    }

    pub fn len(&self) -> usize {
        self.levels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }
}

impl Wires for ChannelRegistry {
    fn level(&self, channel: &str) -> bool {
        ChannelRegistry::level(self, channel)
    }

    fn drive(&mut self, channel: &str, level: bool) {
        ChannelRegistry::drive(self, channel, level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_trimmed_lowercased_and_inner_space_collapsed() {
        assert_eq!(
            ChannelRegistry::normalize("  CLK  ").as_deref(),
            Some("clk")
        );
        assert_eq!(
            ChannelRegistry::normalize("Carry\tIn").as_deref(),
            Some("carry in")
        );
        assert_eq!(
            ChannelRegistry::normalize("bit  0").as_deref(),
            Some("bit 0")
        );
    }

    #[test]
    fn a_name_already_in_shape_is_borrowed_and_never_copied() {
        for name in ["clk", "carry in", "ring-clock1.q", "bit0"] {
            assert!(
                matches!(ChannelRegistry::normalize(name), Some(Cow::Borrowed(_))),
                "{name} is already normal"
            );
        }
        for name in ["Clk", " clk", "clk ", "carry  in", "carry\tin"] {
            assert!(
                matches!(ChannelRegistry::normalize(name), Some(Cow::Owned(_))),
                "{name:?} needs reshaping"
            );
        }
    }

    #[test]
    fn a_wire_driven_one_stage_and_left_alone_the_next_keeps_its_level() {
        let mut reg = ChannelRegistry::new();
        reg.drive("q", true);
        reg.commit();
        reg.commit();
        assert!(reg.level("q"), "a stage with no driver changes nothing");
        reg.drive("q", false);
        reg.commit();
        assert!(!reg.level("q"));
    }

    #[test]
    fn a_blank_name_is_not_a_channel() {
        assert!(ChannelRegistry::normalize("").is_none());
        assert!(ChannelRegistry::normalize("   ").is_none());
        let mut reg = ChannelRegistry::new();
        assert!(reg.register(" ").is_none());
        assert!(reg.is_empty());
    }

    #[test]
    fn the_same_channel_under_any_spelling_is_one_wire() {
        let mut reg = ChannelRegistry::new();
        reg.register("clk");
        reg.register("CLK");
        reg.register("  Clk ");
        assert_eq!(reg.len(), 1, "one wire, however it was typed");
        reg.set_level("CLK", true);
        assert!(reg.level("clk"), "driving any spelling drives the wire");
    }

    #[test]
    fn a_channel_starts_low_and_holds_its_level() {
        let mut reg = ChannelRegistry::new();
        reg.register("carry");
        assert!(!reg.level("carry"), "a fresh channel is low");
        reg.set_level("carry", true);
        assert!(reg.level("carry"));
        assert!(reg.level("carry"), "a level stays until driven otherwise");
        reg.set_level("carry", false);
        assert!(!reg.level("carry"));
    }

    #[test]
    fn an_unknown_channel_reads_low() {
        let reg = ChannelRegistry::new();
        assert!(!reg.level("nothing is wired here"));
        assert!(!reg.contains("nothing is wired here"));
    }

    #[test]
    fn driving_a_channel_registers_it() {
        let mut reg = ChannelRegistry::new();
        assert!(reg.set_level("bit0", true));
        assert!(reg.contains("BIT0"));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn a_driven_level_is_invisible_until_the_stage_ends() {
        let mut reg = ChannelRegistry::new();
        assert!(reg.drive("q", true));
        assert!(
            !reg.level("q"),
            "everyone reads the previous stage, never a neighbour's fresh output"
        );
        reg.commit();
        assert!(reg.level("q"));
    }

    #[test]
    fn many_drivers_on_one_wire_merge_as_or() {
        let mut reg = ChannelRegistry::new();
        reg.drive("bus", false);
        reg.drive("bus", false);
        reg.commit();
        assert!(!reg.level("bus"), "no driver is high, so the wire is low");

        reg.drive("bus", false);
        reg.drive("bus", true);
        reg.drive("bus", false);
        reg.commit();
        assert!(
            reg.level("bus"),
            "one high driver pulls the whole wire high"
        );
    }

    #[test]
    fn an_undriven_channel_keeps_the_level_it_was_given() {
        let mut reg = ChannelRegistry::new();
        reg.set_level("x", true);
        for _ in 0..STAGES_WATCHED {
            reg.commit();
        }
        assert!(reg.level("x"), "an input nobody drives holds its level");
    }

    #[test]
    fn a_committed_wire_falls_when_its_drivers_go_low() {
        let mut reg = ChannelRegistry::new();
        reg.drive("q", true);
        reg.commit();
        reg.drive("q", false);
        reg.commit();
        assert!(!reg.level("q"), "the wire follows its drivers down again");
    }

    #[test]
    fn driving_a_channel_registers_it_once_committed() {
        let mut reg = ChannelRegistry::new();
        reg.drive("carry", false);
        reg.commit();
        assert!(reg.contains("CARRY"));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn a_blank_channel_cannot_be_driven() {
        let mut reg = ChannelRegistry::new();
        assert!(!reg.drive("  ", true));
        reg.commit();
        assert!(reg.is_empty());
    }

    const STAGES_WATCHED: usize = 8;
    const MANY_CHANNELS: usize = 5_000;

    #[test]
    fn the_registry_is_unbounded() {
        let mut reg = ChannelRegistry::new();
        for i in 0..MANY_CHANNELS {
            reg.register(&format!("bit{i}"));
        }
        assert_eq!(reg.len(), MANY_CHANNELS, "channels are never capped");
    }

    #[test]
    fn names_come_back_sorted_for_a_stable_readout() {
        let mut reg = ChannelRegistry::new();
        reg.register("sum");
        reg.register("carry");
        reg.register("clk");
        let names: Vec<&str> = reg.names().collect();
        assert_eq!(names, vec!["carry", "clk", "sum"]);
    }
}
