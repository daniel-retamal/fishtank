use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Write;

use super::Part;
use crate::tank::{ChannelRegistry, Wires};

pub const PIN_WIDTH_MIN: u8 = 1;
pub const PIN_WIDTH_MAX: u8 = 16;
const BIT_SUFFIX_MAX_LEN: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PinDirection {
    In,
    Out,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sensitivity {
    Level,
    Edge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PinSpec {
    pub name: &'static str,
    pub direction: PinDirection,
    pub width: u8,
    pub sensitivity: Sensitivity,
}

impl PinSpec {
    pub const fn out(name: &'static str, width: u8) -> Self {
        Self {
            name,
            direction: PinDirection::Out,
            width,
            sensitivity: Sensitivity::Level,
        }
    }

    pub const fn flag(name: &'static str) -> Self {
        Self::out(name, PIN_WIDTH_MIN)
    }

    pub const fn input(name: &'static str, width: u8) -> Self {
        Self {
            name,
            direction: PinDirection::In,
            width,
            sensitivity: Sensitivity::Level,
        }
    }

    pub const fn strobe(name: &'static str) -> Self {
        Self {
            sensitivity: Sensitivity::Edge,
            ..Self::input(name, PIN_WIDTH_MIN)
        }
    }

    pub fn is_strobe(&self) -> bool {
        self.sensitivity == Sensitivity::Edge
    }
}

pub const OUTPUT_PIN: PinSpec = PinSpec::flag("out");

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PinOwner {
    Body,
    Part(Part),
}

impl From<Part> for PinOwner {
    fn from(part: Part) -> Self {
        PinOwner::Part(part)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pin {
    pub owner: PinOwner,
    pub name: Cow<'static, str>,
    pub direction: PinDirection,
    pub width: u8,
}

impl Pin {
    pub fn of(owner: impl Into<PinOwner>, spec: PinSpec) -> Self {
        Self {
            owner: owner.into(),
            name: Cow::Borrowed(spec.name),
            direction: spec.direction,
            width: spec.width,
        }
    }

    pub fn output() -> Self {
        Self::of(PinOwner::Body, OUTPUT_PIN)
    }

    pub fn line(name: String, direction: PinDirection) -> Self {
        Self {
            owner: PinOwner::Body,
            name: Cow::Owned(name),
            direction,
            width: PIN_WIDTH_MIN,
        }
    }

    pub fn is_output(&self) -> bool {
        self.same_wire(&Self::output())
    }

    pub fn same_wire(&self, other: &Pin) -> bool {
        self.owner == other.owner && self.name == other.name
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bus {
    base: String,
    width: u8,
}

impl Bus {
    pub fn new(channel: &str, width: u8) -> Option<Self> {
        if !(PIN_WIDTH_MIN..=PIN_WIDTH_MAX).contains(&width) {
            return None;
        }
        Some(Self {
            base: ChannelRegistry::normalize(channel)?.into_owned(),
            width,
        })
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    fn line_into(base: &str, width: u8, bit: u8, out: &mut String) {
        out.clear();
        out.push_str(base);
        if width == PIN_WIDTH_MIN {
            return;
        }
        let _ = write!(out, "{bit}");
    }

    fn line_buffer(base: &str) -> String {
        String::with_capacity(base.len() + BIT_SUFFIX_MAX_LEN)
    }

    pub fn lines(&self) -> Vec<String> {
        let mut line = String::new();
        let mut named = Vec::with_capacity(self.width as usize);
        for bit in 0..self.width {
            Self::line_into(&self.base, self.width, bit, &mut line);
            named.push(line.clone());
        }
        named
    }

    pub fn ceiling(&self) -> u32 {
        (1u32 << self.width) - 1
    }

    pub fn drive_into<W: Wires>(&self, value: u32, wires: &mut W) {
        let held = self.saturate(value);
        let mut line = Self::line_buffer(&self.base);
        for bit in 0..self.width {
            Self::line_into(&self.base, self.width, bit, &mut line);
            wires.drive(&line, held >> bit & 1 == 1);
        }
    }

    pub fn read_from<W: Wires>(&self, wires: &W) -> u32 {
        Self::read_lines(&self.base, self.width, wires)
    }

    fn read_lines<W: Wires>(base: &str, width: u8, wires: &W) -> u32 {
        if width == PIN_WIDTH_MIN {
            return u32::from(wires.level(base));
        }
        let mut line = Self::line_buffer(base);
        (0..width).fold(0, |value, bit| {
            Self::line_into(base, width, bit, &mut line);
            value | u32::from(wires.level(&line)) << bit
        })
    }

    fn saturate(&self, value: u32) -> u32 {
        value.min(self.ceiling())
    }

    pub fn encode(&self, value: u32) -> Vec<bool> {
        let held = self.saturate(value);
        (0..self.width).map(|bit| held >> bit & 1 == 1).collect()
    }

    pub fn decode(&self, levels: &[bool]) -> u32 {
        levels
            .iter()
            .take(self.width as usize)
            .enumerate()
            .filter(|(_, high)| **high)
            .map(|(bit, _)| 1u32 << bit)
            .sum()
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct PinReadings {
    values: BTreeMap<&'static str, u32>,
}

impl PinReadings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, spec: &PinSpec, value: u32) {
        self.values.insert(spec.name, value);
    }

    pub fn flag(&mut self, spec: &PinSpec, high: bool) {
        self.set(spec, u32::from(high));
    }

    pub fn get(&self, pin: &str) -> Option<u32> {
        self.values.get(pin).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct PinMap {
    wires: BTreeMap<PinOwner, BTreeMap<String, String>>,
}

impl PinMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wire(&mut self, owner: impl Into<PinOwner>, pin: &str, channel: &str) -> bool {
        let Some(key) = ChannelRegistry::normalize(pin) else {
            return false;
        };
        let Some(target) = ChannelRegistry::normalize(channel) else {
            return false;
        };
        self.wires
            .entry(owner.into())
            .or_default()
            .insert(key.into_owned(), target.into_owned());
        true
    }

    pub fn unwire(&mut self, owner: impl Into<PinOwner>, pin: &str) -> bool {
        let owner = owner.into();
        let Some(key) = ChannelRegistry::normalize(pin) else {
            return false;
        };
        let Some(owned) = self.wires.get_mut(&owner) else {
            return false;
        };
        let removed = owned.remove(key.as_ref()).is_some();
        if owned.is_empty() {
            self.wires.remove(&owner);
        }
        removed
    }

    pub fn channel(&self, owner: impl Into<PinOwner>, pin: &str) -> Option<&str> {
        let key = ChannelRegistry::normalize(pin)?;
        self.wires
            .get(&owner.into())?
            .get(key.as_ref())
            .map(|channel| channel.as_str())
    }

    pub fn channel_of(&self, pin: &Pin) -> Option<&str> {
        self.channel(pin.owner, &pin.name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (PinOwner, &str, &str)> {
        self.wires.iter().flat_map(|(&owner, owned)| {
            owned
                .iter()
                .map(move |(pin, channel)| (owner, pin.as_str(), channel.as_str()))
        })
    }

    pub fn len(&self) -> usize {
        self.wires.values().map(BTreeMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.wires.is_empty()
    }

    pub fn bus(&self, pin: &Pin) -> Option<Bus> {
        Bus::new(self.channel_of(pin)?, pin.width)
    }

    pub fn value<W: Wires>(&self, pin: &Pin, wires: &W) -> u32 {
        let Some(channel) = self.channel_of(pin) else {
            return 0;
        };
        Bus::read_lines(channel, pin.width, wires)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: PinOwner = PinOwner::Body;
    const MODULE: Part = Part::CommandModule;

    #[test]
    fn a_fresh_map_wires_nothing() {
        let map = PinMap::new();
        assert!(map.is_empty());
        assert_eq!(map.channel(BODY, OUTPUT_PIN.name), None);
    }

    #[test]
    fn a_pin_carries_the_channel_it_was_wired_to() {
        let mut map = PinMap::new();
        assert!(map.wire(BODY, OUTPUT_PIN.name, "carry"));
        assert_eq!(map.channel(BODY, OUTPUT_PIN.name), Some("carry"));
        assert_eq!(map.channel_of(&Pin::output()), Some("carry"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn a_channel_is_normalised_the_same_way_the_registry_normalises_it() {
        let mut map = PinMap::new();
        map.wire(BODY, "OUT", "  Carry  In ");
        assert_eq!(map.channel(BODY, "out"), Some("carry in"));
        assert_eq!(map.channel(BODY, "Out"), Some("carry in"));
    }

    #[test]
    fn a_pin_holds_exactly_one_channel() {
        let mut map = PinMap::new();
        map.wire(MODULE, "fire", "harvest");
        map.wire(MODULE, "fire", "restock");
        assert_eq!(
            map.channel(MODULE, "fire"),
            Some("restock"),
            "rewiring replaces"
        );
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn a_pin_belongs_to_its_owner_so_one_name_on_two_owners_is_two_wires() {
        let mut map = PinMap::new();
        map.wire(Part::Cochlea, "char", "kbd");
        map.wire(Part::GlyphPanel, "char", "lcd");
        assert_eq!(map.channel(Part::Cochlea, "char"), Some("kbd"));
        assert_eq!(map.channel(Part::GlyphPanel, "char"), Some("lcd"));
        assert_eq!(map.channel(BODY, "char"), None, "the fish owns no char");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn a_blank_name_wires_nothing() {
        let mut map = PinMap::new();
        assert!(!map.wire(MODULE, "fire", "   "));
        assert!(!map.wire(MODULE, "  ", "harvest"));
        assert!(map.is_empty());
    }

    #[test]
    fn unwiring_reports_whether_anything_was_connected() {
        let mut map = PinMap::new();
        map.wire(MODULE, "fire", "harvest");
        assert!(!map.unwire(BODY, "fire"), "the fish never owned that pin");
        assert!(map.unwire(MODULE, "FIRE"));
        assert!(!map.unwire(MODULE, "fire"), "already disconnected");
        assert!(map.is_empty());
    }

    #[test]
    fn pins_come_back_sorted_by_owner_then_name_for_a_stable_readout() {
        let mut map = PinMap::new();
        map.wire(Part::Cochlea, "strobe", "s");
        map.wire(Part::Cochlea, "char", "c");
        map.wire(BODY, "out", "o");
        let pins: Vec<(PinOwner, &str)> = map.iter().map(|(owner, pin, _)| (owner, pin)).collect();
        assert_eq!(
            pins,
            vec![
                (BODY, "out"),
                (PinOwner::Part(Part::Cochlea), "char"),
                (PinOwner::Part(Part::Cochlea), "strobe"),
            ]
        );
    }

    #[test]
    fn only_a_pin_declared_a_strobe_is_clocked() {
        assert!(PinSpec::strobe("write").is_strobe());
        assert!(
            !PinSpec::input("char", COUNT_WIDTH).is_strobe(),
            "a bus carries a value, not an edge"
        );
        assert!(
            !PinSpec::input("bit", PIN_WIDTH_MIN).is_strobe(),
            "a one-bit data line is a level, not an edge"
        );
        assert!(
            !PinSpec::flag("ready").is_strobe(),
            "an output is never clocked"
        );
        let strobe = PinSpec::strobe("write");
        assert_eq!(
            (strobe.direction, strobe.width),
            (PinDirection::In, PIN_WIDTH_MIN),
            "a strobe is one wire arriving at the part"
        );
    }

    #[test]
    fn an_input_bus_reads_back_the_number_its_lines_carry() {
        const CHAR: PinSpec = PinSpec::input("char", COUNT_WIDTH);
        let mut map = PinMap::new();
        let pin = Pin::of(Part::GlyphPanel, CHAR);
        let mut channels = ChannelRegistry::new();
        bus_of(COUNT_WIDTH).drive_into(u32::from(b'A'), &mut channels);
        channels.commit();

        assert_eq!(map.value(&pin, &channels), 0, "an unwired input reads zero");
        map.wire(Part::GlyphPanel, "char", "shoal");
        assert_eq!(map.value(&pin, &channels), u32::from(b'A'));
    }

    #[test]
    fn a_one_bit_input_reads_its_bare_channel() {
        const WRITE: PinSpec = PinSpec::strobe("write");
        let mut map = PinMap::new();
        map.wire(Part::GlyphPanel, "write", "wr");
        let mut channels = ChannelRegistry::new();
        channels.set_level("wr", true);
        assert_eq!(map.value(&Pin::of(Part::GlyphPanel, WRITE), &channels), 1);
    }

    #[test]
    fn an_input_pin_is_a_wire_the_fish_reads_instead_of_drives() {
        const FIRE: PinSpec = PinSpec::input("fire", PIN_WIDTH_MIN);
        assert_eq!(FIRE.direction, PinDirection::In);
        assert_eq!(FIRE.width, PIN_WIDTH_MIN);
        assert_ne!(
            FIRE.direction,
            PinSpec::flag("fire").direction,
            "a flag leaves the part; an input arrives at it"
        );
    }

    #[test]
    fn the_fish_output_pin_is_a_single_bit_leaving_the_fish() {
        assert_eq!(OUTPUT_PIN.direction, PinDirection::Out);
        assert_eq!(OUTPUT_PIN.width, PIN_WIDTH_MIN);
    }

    const COUNT_WIDTH: u8 = 8;
    const CASH_WIDTH: u8 = 16;

    fn bus_of(width: u8) -> Bus {
        Bus::new("shoal", width).expect("a declared width is a legal bus")
    }

    #[test]
    fn a_bus_is_one_wire_per_bit_numbered_from_the_lowest() {
        let bus = bus_of(COUNT_WIDTH);
        assert_eq!(bus.lines().len(), COUNT_WIDTH as usize);
        assert_eq!(bus.lines()[0], "shoal0", "bit 0 is the lowest bit");
        assert_eq!(bus.lines()[7], "shoal7");
    }

    #[test]
    fn a_single_bit_pin_keeps_the_channel_it_was_given() {
        let flag = Bus::new("ripe", PIN_WIDTH_MIN).expect("one bit is a legal width");
        assert_eq!(
            flag.lines(),
            vec!["ripe"],
            "a flag is a wire, not a bus of one"
        );
    }

    #[test]
    fn a_value_rides_the_bus_lowest_bit_first() {
        let bus = bus_of(COUNT_WIDTH);
        assert_eq!(
            bus.encode(1),
            vec![true, false, false, false, false, false, false, false]
        );
        assert_eq!(
            bus.encode(6),
            vec![false, true, true, false, false, false, false, false]
        );
    }

    #[test]
    fn every_declared_width_reads_back_from_the_wires_what_it_drove() {
        for width in PIN_WIDTH_MIN..=PIN_WIDTH_MAX {
            let bus = bus_of(width);
            for value in [0, 1, bus.ceiling() / 2, bus.ceiling()] {
                let mut channels = ChannelRegistry::new();
                bus.drive_into(value, &mut channels);
                channels.commit();
                assert_eq!(bus.read_from(&channels), value, "{width} bits");
            }
        }
    }

    #[test]
    fn every_declared_width_round_trips_its_whole_range() {
        for width in PIN_WIDTH_MIN..=PIN_WIDTH_MAX {
            let bus = bus_of(width);
            for value in [0, 1, bus.ceiling() / 2, bus.ceiling()] {
                assert_eq!(
                    bus.decode(&bus.encode(value)),
                    value,
                    "{value} does not survive a {width}-bit bus"
                );
            }
        }
    }

    #[test]
    fn a_value_too_big_for_the_bus_saturates_instead_of_wrapping() {
        let bus = bus_of(COUNT_WIDTH);
        assert_eq!(bus.ceiling(), 255);
        assert_eq!(
            bus.decode(&bus.encode(4000)),
            255,
            "a full bus reads full, never zero"
        );
        assert_eq!(bus_of(CASH_WIDTH).ceiling(), 65535);
    }

    #[test]
    fn a_short_reading_only_fills_the_bits_it_covers() {
        let bus = bus_of(COUNT_WIDTH);
        assert_eq!(bus.decode(&[true, false, true]), 5);
        assert_eq!(bus.decode(&[]), 0, "an unread bus is zero");
    }

    #[test]
    fn a_bus_spells_its_channel_the_way_the_registry_does() {
        let bus = Bus::new("  Carry  In ", 2).expect("a name with spaces is one channel");
        assert_eq!(bus.lines(), vec!["carry in0", "carry in1"]);
        assert!(
            Bus::new("   ", COUNT_WIDTH).is_none(),
            "blank wires nothing"
        );
        assert!(
            Bus::new("shoal", PIN_WIDTH_MAX + 1).is_none(),
            "a width outside the declared range is not a bus"
        );
    }

    #[test]
    fn driving_a_bus_puts_the_value_on_exactly_the_wires_it_names() {
        for width in PIN_WIDTH_MIN..=PIN_WIDTH_MAX {
            let bus = bus_of(width);
            let value = bus.ceiling() / 3;
            let mut channels = ChannelRegistry::new();

            bus.drive_into(value, &mut channels);
            channels.commit();

            let levels: Vec<bool> = bus.lines().iter().map(|w| channels.level(w)).collect();
            assert_eq!(
                bus.decode(&levels),
                value,
                "a {width}-bit bus must read back what it drove"
            );
            assert_eq!(
                channels.len(),
                width as usize,
                "a bus drives one wire per bit and no others"
            );
        }
    }

    #[test]
    fn driving_a_bus_too_big_a_number_pegs_every_wire() {
        let bus = bus_of(COUNT_WIDTH);
        let mut channels = ChannelRegistry::new();

        bus.drive_into(u32::MAX, &mut channels);
        channels.commit();

        assert!(
            bus.lines().iter().all(|wire| channels.level(wire)),
            "a saturated bus reads full, never wraps to zero"
        );
    }

    #[test]
    fn driving_a_flag_never_invents_a_bit_suffix() {
        let flag = Bus::new("ripe", PIN_WIDTH_MIN).expect("one bit is a legal width");
        let mut channels = ChannelRegistry::new();

        flag.drive_into(1, &mut channels);
        channels.commit();

        assert!(channels.level("ripe"));
        assert!(
            !channels.contains("ripe0"),
            "a flag is a wire, not a bus of one"
        );
    }

    #[test]
    fn a_reading_is_a_number_a_flag_is_a_reading_of_one_or_zero() {
        const COUNT: PinSpec = PinSpec::out("count", COUNT_WIDTH);
        const FULL: PinSpec = PinSpec::flag("full");
        let mut readings = PinReadings::new();
        assert!(readings.is_empty());

        readings.set(&COUNT, 12);
        readings.flag(&FULL, true);

        assert_eq!(readings.get("count"), Some(12));
        assert_eq!(readings.get("full"), Some(1));
        assert_eq!(readings.get("empty"), None, "an unread pin is not driven");
    }

    #[test]
    fn a_low_flag_still_reads_as_a_driven_zero() {
        const EMPTY: PinSpec = PinSpec::flag("empty");
        let mut readings = PinReadings::new();
        readings.flag(&EMPTY, false);
        assert_eq!(
            readings.get("empty"),
            Some(0),
            "a sense that says no is still driving the wire"
        );
    }

    #[test]
    fn a_wired_pin_hands_back_the_bus_at_its_declared_width() {
        const COUNT_PIN: PinSpec = PinSpec::out("count", COUNT_WIDTH);
        let mut map = PinMap::new();
        let pin = Pin::of(Part::ShoalCounter, COUNT_PIN);
        assert_eq!(map.bus(&pin), None, "an unwired pin has no bus");
        map.wire(Part::ShoalCounter, COUNT_PIN.name, "shoal");
        let bus = map.bus(&pin).expect("the pin is wired");
        assert_eq!(bus.width(), COUNT_WIDTH);
        assert_eq!(bus.lines().len(), COUNT_WIDTH as usize);
    }
}
