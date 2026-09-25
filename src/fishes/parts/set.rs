use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

use super::{
    Buffer, ConfigSpec, Display, KeyBinding, Part, PartConfig, PartEffect, PinDirection,
    PinReadings, PinSpec,
};
use crate::tank::{Link, Transmission, WorldView};

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct Installation {
    count: u32,
    memory: VecDeque<bool>,
    config: PartConfig,
    buffer: Buffer,
}

fn remembered(memory: &mut VecDeque<bool>, index: usize, level: bool) -> bool {
    while memory.len() <= index {
        memory.push_back(false);
    }
    std::mem::replace(&mut memory[index], level)
}

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PartSet {
    installed: BTreeMap<Part, Installation>,
}

impl PartSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(&mut self, part: Part) -> bool {
        let slot = self.installed.entry(part).or_default();
        if slot.count > 0 && !part.stacks() {
            return false;
        }
        slot.count += 1;
        true
    }

    pub fn remove(&mut self, part: Part) -> bool {
        let Some(slot) = self.installed.get_mut(&part) else {
            return false;
        };
        slot.count -= 1;
        if slot.count == 0 {
            self.installed.remove(&part);
        }
        true
    }

    pub fn absorb(&mut self, donor: &PartSet) {
        for (&part, theirs) in &donor.installed {
            match self.installed.get_mut(&part) {
                Some(ours) if part.stacks() => ours.count += theirs.count,
                Some(_) => {}
                None => {
                    self.installed.insert(part, theirs.clone());
                }
            }
        }
    }

    pub fn design(&self) -> Self {
        let installed = self
            .installed
            .iter()
            .map(|(&part, slot)| {
                let fresh = Installation {
                    count: slot.count,
                    memory: VecDeque::new(),
                    config: slot.config.clone(),
                    buffer: Buffer::new(),
                };
                (part, fresh)
            })
            .collect();
        Self { installed }
    }

    pub fn modify_output(&mut self, level: bool) -> bool {
        self.installed
            .iter_mut()
            .fold(level, |carried, (&part, slot)| {
                part.modify_output(carried, slot.count, &mut slot.memory)
            })
    }

    pub fn rising_edges(
        &mut self,
        value_of: impl Fn(Part, &PinSpec) -> u32,
    ) -> Vec<(Part, PartEffect)> {
        let mut fired = Vec::new();
        for (&part, slot) in &mut self.installed {
            let inputs = |spec: &PinSpec| value_of(part, spec);
            for (index, pin) in part.strobes().enumerate() {
                let level = inputs(pin) != 0;
                if remembered(&mut slot.memory, index, level) || !level {
                    continue;
                }
                let effect = part.on_rising_edge(pin.name, &slot.config, &inputs, &mut slot.buffer);
                fired.extend(effect.map(|effect| (part, effect)));
            }
        }
        fired
    }

    pub fn reads_world(&self) -> bool {
        self.installed.keys().any(|part| {
            part.pins()
                .iter()
                .any(|pin| pin.direction == PinDirection::Out)
        })
    }

    pub fn report(
        &self,
        value_of: impl Fn(Part, &PinSpec) -> u32,
        world: &WorldView,
        mut reported: impl FnMut(Part, &PinReadings),
    ) {
        for (&part, slot) in &self.installed {
            let inputs = |spec: &PinSpec| value_of(part, spec);
            let mut readings = PinReadings::new();
            part.report(&slot.config, &inputs, &slot.buffer, world, &mut readings);
            if !readings.is_empty() {
                reported(part, &readings);
            }
        }
    }

    pub fn links(&self) -> impl Iterator<Item = Link> + '_ {
        self.installed
            .iter()
            .filter_map(|(&part, slot)| part.link(&slot.config))
    }

    pub fn transmissions(
        &self,
        value_of: impl Fn(Part, &PinSpec) -> Option<u32>,
    ) -> Vec<Transmission> {
        let mut sent = Vec::new();
        for (&part, slot) in &self.installed {
            let Some(link) = part.link(&slot.config) else {
                continue;
            };
            let inputs = |spec: &PinSpec| value_of(part, spec);
            if let Some(level) = part.transmit(&inputs) {
                sent.push(Transmission { link, level });
            }
        }
        sent
    }

    pub fn displays(&self) -> impl Iterator<Item = Display> + '_ {
        self.installed
            .iter()
            .filter_map(|(&part, slot)| part.display(&slot.config, &slot.buffer))
    }

    pub fn pulse(&mut self, part: Part) {
        if let Some(slot) = self.installed.get_mut(&part) {
            slot.buffer.raise();
        }
    }

    pub fn settle(&mut self) {
        for slot in self.installed.values_mut() {
            slot.buffer.settle();
        }
    }

    pub fn hear(&mut self, speech: &str) {
        for (&part, slot) in &mut self.installed {
            part.hear(speech, &mut slot.buffer);
        }
    }

    pub fn key(&mut self, key: &str, down: bool) {
        for (&part, slot) in &mut self.installed {
            part.key(key, down, &slot.config, &mut slot.buffer);
        }
    }

    pub fn bindings(&self) -> impl Iterator<Item = KeyBinding> + '_ {
        self.installed
            .iter()
            .flat_map(|(&part, slot)| part.bindings(&slot.config))
    }

    pub fn configure(&mut self, part: Part, spec: &ConfigSpec, value: &str) -> bool {
        let Some(slot) = self.installed.get_mut(&part) else {
            return false;
        };
        slot.config.set(spec, value)
    }

    pub fn config(&self, part: Part) -> PartConfig {
        self.installed
            .get(&part)
            .map(|slot| slot.config.clone())
            .unwrap_or_default()
    }

    pub fn count(&self, part: Part) -> u32 {
        self.installed.get(&part).map_or(0, |slot| slot.count)
    }

    pub fn has(&self, part: Part) -> bool {
        self.count(part) > 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (Part, u32)> {
        self.installed
            .iter()
            .map(|(&part, slot)| (part, slot.count))
    }

    pub fn len(&self) -> usize {
        self.installed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.installed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_set_holds_nothing() {
        let set = PartSet::new();
        assert!(set.is_empty());
        assert_eq!(set.count(Part::InverterCoil), 0);
        assert!(!set.has(Part::DelaySpool));
    }

    #[test]
    fn installing_a_part_bolts_it_on() {
        let mut set = PartSet::new();
        assert!(set.install(Part::InverterCoil));
        assert!(set.has(Part::InverterCoil));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn spools_stack_because_a_count_of_them_means_something() {
        let mut set = PartSet::new();
        assert!(set.install(Part::DelaySpool));
        assert!(set.install(Part::DelaySpool));
        assert!(set.install(Part::DelaySpool));
        assert_eq!(set.count(Part::DelaySpool), 3);
        assert_eq!(set.len(), 1, "three spools are still one kind of part");
    }

    #[test]
    fn a_second_coil_does_nothing_and_says_so() {
        let mut set = PartSet::new();
        set.install(Part::InverterCoil);
        assert!(
            !set.install(Part::InverterCoil),
            "a coil never double-inverts, so the second one is refused"
        );
        assert_eq!(set.count(Part::InverterCoil), 1);
    }

    #[test]
    fn removing_takes_one_off_the_stack() {
        let mut set = PartSet::new();
        set.install(Part::DelaySpool);
        set.install(Part::DelaySpool);
        assert!(set.remove(Part::DelaySpool));
        assert_eq!(set.count(Part::DelaySpool), 1);
        assert!(set.remove(Part::DelaySpool));
        assert!(set.is_empty(), "the last one off clears the entry");
    }

    #[test]
    fn removing_what_was_never_installed_is_a_truthful_false() {
        let mut set = PartSet::new();
        assert!(!set.remove(Part::InverterCoil));
    }

    #[test]
    fn an_empty_set_hands_the_output_straight_back() {
        let mut set = PartSet::new();
        assert!(set.modify_output(true));
        assert!(!set.modify_output(false));
    }

    #[test]
    fn a_coil_inverts_the_output() {
        let mut set = PartSet::new();
        set.install(Part::InverterCoil);
        assert!(!set.modify_output(true));
        assert!(set.modify_output(false));
    }

    #[test]
    fn a_refused_second_coil_never_double_inverts() {
        let mut set = PartSet::new();
        set.install(Part::InverterCoil);
        set.install(Part::InverterCoil);
        assert!(
            !set.modify_output(true),
            "the second coil was refused, so the output is inverted once"
        );
    }

    #[test]
    fn a_spool_holds_the_output_back_by_a_stage() {
        let mut set = PartSet::new();
        set.install(Part::DelaySpool);
        assert!(!set.modify_output(true), "the line starts empty");
        assert!(set.modify_output(false), "here comes the stage before");
    }

    #[test]
    fn a_coil_and_a_spool_invert_first_then_delay() {
        let mut set = PartSet::new();
        set.install(Part::InverterCoil);
        set.install(Part::DelaySpool);
        assert!(!set.modify_output(false), "the line starts empty");
        assert!(
            set.modify_output(false),
            "the delayed value is the inverted one"
        );
    }

    const WIRE: [bool; 6] = [false, true, true, false, false, true];
    const FIRINGS: [usize; 6] = [0, 1, 0, 0, 0, 1];

    fn firings(set: &mut PartSet, wire: &[bool]) -> Vec<usize> {
        wire.iter()
            .map(|&high| set.rising_edges(|_, _| u32::from(high)).len())
            .collect()
    }

    #[test]
    fn a_command_module_fires_the_stage_its_wire_rises_and_never_while_it_holds() {
        let mut set = PartSet::new();
        set.install(Part::CommandModule);
        assert_eq!(
            firings(&mut set, &WIRE),
            FIRINGS.to_vec(),
            "one firing per rise, none on a hold and none on a fall"
        );
    }

    #[test]
    fn a_wire_that_is_already_high_is_still_a_rise_from_nothing() {
        let mut set = PartSet::new();
        set.install(Part::CommandModule);
        assert_eq!(firings(&mut set, &[true, true]), vec![1, 0]);
    }

    #[test]
    fn an_unwired_input_never_fires() {
        let mut set = PartSet::new();
        set.install(Part::CommandModule);
        assert!(
            set.rising_edges(|_, _| 0).is_empty(),
            "a pin connected to nothing reads low forever"
        );
    }

    #[test]
    fn a_part_with_no_input_pins_answers_no_edge() {
        let mut set = PartSet::new();
        set.install(Part::InverterCoil);
        set.install(Part::ShoalCounter);
        assert!(set.rising_edges(|_, _| 1).is_empty());
    }

    #[test]
    fn an_edge_detector_and_a_spool_do_not_share_a_memory() {
        let mut set = PartSet::new();
        set.install(Part::DelaySpool);
        set.install(Part::CommandModule);

        assert_eq!(set.rising_edges(|_, _| 1).len(), 1);
        assert!(!set.modify_output(true), "the spool line is still filling");
        assert!(
            set.rising_edges(|_, _| 1).is_empty(),
            "the spool must not have disturbed the module's remembered level"
        );
        assert!(
            set.modify_output(false),
            "and the spool still delays by one"
        );
    }

    #[test]
    fn absorbing_a_set_adds_spools_and_refuses_a_second_coil() {
        let mut receiver = PartSet::new();
        receiver.install(Part::InverterCoil);
        receiver.install(Part::DelaySpool);
        let mut donor = PartSet::new();
        donor.install(Part::InverterCoil);
        donor.install(Part::DelaySpool);
        donor.install(Part::DelaySpool);
        donor.install(Part::CommandModule);

        receiver.absorb(&donor);

        assert_eq!(
            receiver.iter().collect::<Vec<_>>(),
            vec![
                (Part::InverterCoil, 1),
                (Part::DelaySpool, 3),
                (Part::CommandModule, 1)
            ],
            "spools add up, a coil does not, and the declaration order holds"
        );
        assert!(
            !receiver.modify_output(true),
            "the absorbed coil never double-inverts"
        );
    }

    #[test]
    fn an_absorbed_part_keeps_the_config_it_came_with_unless_the_receiver_has_one() {
        let spec = Part::ShoalCounter
            .config()
            .iter()
            .find(|spec| spec.name == "threshold")
            .expect("the counter has a threshold");
        let mut receiver = PartSet::new();
        let mut donor = PartSet::new();
        donor.install(Part::ShoalCounter);
        donor.configure(Part::ShoalCounter, spec, "9");

        receiver.absorb(&donor);
        assert_eq!(receiver.config(Part::ShoalCounter).number(spec), 9);

        let mut other = PartSet::new();
        other.install(Part::ShoalCounter);
        other.configure(Part::ShoalCounter, spec, "2");
        receiver.absorb(&other);
        assert_eq!(
            receiver.config(Part::ShoalCounter).number(spec),
            9,
            "the part already bolted on keeps its settings"
        );
    }

    fn reading(set: &PartSet, pin: &str) -> Option<u32> {
        let mut found = None;
        set.report(
            |_, _| 0,
            &WorldView::default(),
            |_, readings| {
                found = found.or(readings.get(pin));
            },
        );
        found
    }

    fn character(set: &PartSet) -> Option<u32> {
        reading(set, "char")
    }

    fn ear_that_heard(line: &str) -> PartSet {
        let mut set = PartSet::new();
        set.install(Part::Cochlea);
        set.hear(line);
        set
    }

    #[test]
    fn a_cochlea_moves_one_character_per_rise_of_its_strobe_and_none_while_it_holds() {
        let mut set = ear_that_heard("abc");
        let seen: Vec<Option<u32>> = [true, true, false, true, false, false, true]
            .iter()
            .map(|&high| {
                set.rising_edges(|_, _| u32::from(high));
                character(&set)
            })
            .collect();
        let byte = |b: u8| Some(u32::from(b));
        assert_eq!(
            seen,
            vec![
                byte(b'a'),
                byte(b'a'),
                byte(b'a'),
                byte(b'b'),
                byte(b'b'),
                byte(b'b'),
                byte(b'c')
            ]
        );
    }

    #[test]
    fn a_design_forgets_the_line_a_cochlea_was_holding() {
        let mut set = ear_that_heard("abc");
        set.rising_edges(|_, _| 1);

        let design = set.design();

        assert_eq!(character(&design), Some(0));
        assert_eq!(
            reading(&design, "ready"),
            Some(0),
            "a blueprint has heard nothing"
        );
    }

    #[test]
    fn an_absorbed_cochlea_arrives_mid_line() {
        let mut donor = ear_that_heard("abc");
        donor.rising_edges(|_, _| 1);
        let mut receiver = PartSet::new();

        receiver.absorb(&donor);

        assert_eq!(character(&receiver), Some(u32::from(b'a')));
        receiver.rising_edges(|_, _| 0);
        receiver.rising_edges(|_, _| 1);
        assert_eq!(
            character(&receiver),
            Some(u32::from(b'b')),
            "the fused ear carries on where the donor stopped"
        );
    }

    const LETTER: u32 = b'Q' as u32;

    fn panel_bus(part: Part, spec: &PinSpec, write: bool) -> u32 {
        assert_eq!(part, Part::GlyphPanel, "only the panel has inputs here");
        match spec.name {
            "char" => LETTER,
            "write" => u32::from(write),
            _ => 0,
        }
    }

    fn panels(set: &PartSet) -> Vec<Vec<String>> {
        set.displays()
            .map(|display| display.rows().to_vec())
            .collect()
    }

    #[test]
    fn a_glyph_panel_writes_the_byte_on_its_bus_at_each_rise_of_write_and_holds_it() {
        let mut set = PartSet::new();
        set.install(Part::GlyphPanel);
        assert_eq!(set.displays().count(), 0, "a blank panel shows nothing");

        for write in [true, true, false, false, false] {
            set.rising_edges(|part, spec| panel_bus(part, spec, write));
        }

        let shown = panels(&set);
        assert_eq!(shown.len(), 1);
        assert_eq!(
            shown[0][0].trim_end(),
            "Q",
            "one rise, one character, held with no signal"
        );
    }

    #[test]
    fn two_parts_with_a_pin_of_one_name_each_see_their_own_value() {
        let mut set = ear_that_heard("abc");
        set.install(Part::GlyphPanel);
        let effects = set.rising_edges(|part, spec| match (part, spec.name) {
            (Part::GlyphPanel, "char") => LETTER,
            (Part::GlyphPanel, "write") => 1,
            _ => 0,
        });
        assert!(effects.is_empty());
        assert_eq!(character(&set), Some(0), "the ear's strobe never rose");
        let shown = panels(&set);
        assert_eq!(shown[0][0].trim_end(), "Q", "the panel read its own char");
    }

    #[test]
    fn a_design_blanks_a_glyph_panel_and_an_absorbed_one_arrives_lit() {
        let mut lit = PartSet::new();
        lit.install(Part::GlyphPanel);
        lit.rising_edges(|part, spec| panel_bus(part, spec, true));

        assert_eq!(
            lit.design().displays().count(),
            0,
            "a blueprint shows nothing"
        );

        let mut receiver = PartSet::new();
        receiver.absorb(&lit);
        assert_eq!(
            receiver.displays().collect::<Vec<_>>(),
            lit.displays().collect::<Vec<_>>(),
            "fusion carries the panel's content"
        );
    }

    #[test]
    fn parts_come_back_with_their_counts() {
        let mut set = PartSet::new();
        set.install(Part::InverterCoil);
        set.install(Part::DelaySpool);
        set.install(Part::DelaySpool);
        let held: Vec<(Part, u32)> = set.iter().collect();
        assert_eq!(held, vec![(Part::InverterCoil, 1), (Part::DelaySpool, 2)]);
    }

    struct Memory {
        addr: u32,
        data_in: u32,
        write: bool,
    }

    impl Memory {
        fn value(&self, part: Part, spec: &PinSpec) -> u32 {
            assert_eq!(part, Part::CoreStack, "only the stack has inputs here");
            match spec.name {
                "addr" => self.addr,
                "data_in" => self.data_in,
                "write" => u32::from(self.write),
                _ => 0,
            }
        }
    }

    fn stack() -> PartSet {
        let mut set = PartSet::new();
        set.install(Part::CoreStack);
        set
    }

    fn clock(set: &mut PartSet, memory: &Memory) {
        assert!(
            set.rising_edges(|part, spec| memory.value(part, spec))
                .is_empty()
        );
    }

    fn word(set: &PartSet, addr: u32) -> Option<u32> {
        let at = Memory {
            addr,
            data_in: 0,
            write: false,
        };
        let mut found = None;
        set.report(
            |part, spec| at.value(part, spec),
            &WorldView::default(),
            |_, readings| found = readings.get("data_out"),
        );
        found
    }

    #[test]
    fn a_core_stack_stores_the_word_on_its_bus_at_the_rise_of_write_and_never_while_it_holds() {
        let mut set = stack();
        for (data_in, write) in [(5, false), (6, true), (7, true), (8, false)] {
            clock(
                &mut set,
                &Memory {
                    addr: 3,
                    data_in,
                    write,
                },
            );
        }
        assert_eq!(
            word(&set, 3),
            Some(6),
            "one rise, one word: the one on the bus as it rose"
        );
    }

    #[test]
    fn a_core_stack_whose_write_never_rises_holds_nothing() {
        let mut set = stack();
        for addr in 0..4 {
            clock(
                &mut set,
                &Memory {
                    addr,
                    data_in: 99,
                    write: false,
                },
            );
        }
        assert!((0..4).all(|addr| word(&set, addr) == Some(0)));
    }

    #[test]
    fn a_design_forgets_every_word_and_fusion_carries_them() {
        let mut set = stack();
        clock(
            &mut set,
            &Memory {
                addr: 200,
                data_in: 42,
                write: true,
            },
        );
        assert_eq!(
            word(&set.design(), 200),
            Some(0),
            "a blueprint remembers nothing"
        );

        let mut receiver = PartSet::new();
        receiver.absorb(&set);
        assert_eq!(
            word(&receiver, 200),
            Some(42),
            "the fused stack keeps what it held"
        );
    }

    struct Screen {
        addr: u32,
        bit: bool,
        write: bool,
        byte: u32,
        write_byte: bool,
    }

    impl Screen {
        const DARK: Screen = Screen {
            addr: 0,
            bit: false,
            write: false,
            byte: 0,
            write_byte: false,
        };

        fn value(&self, part: Part, spec: &PinSpec) -> u32 {
            assert_eq!(part, Part::CathodeArray, "only the cathode has inputs here");
            match spec.name {
                "addr" => self.addr,
                "bit" => u32::from(self.bit),
                "write" => u32::from(self.write),
                "byte" => self.byte,
                "write_byte" => u32::from(self.write_byte),
                _ => 0,
            }
        }
    }

    fn cathode() -> PartSet {
        let mut set = PartSet::new();
        set.install(Part::CathodeArray);
        let width = Part::CathodeArray
            .config()
            .iter()
            .find(|spec| spec.name == "width")
            .expect("the cathode has a width");
        set.configure(Part::CathodeArray, width, "8");
        set
    }

    fn drive(set: &mut PartSet, screen: &Screen) {
        assert!(
            set.rising_edges(|part, spec| screen.value(part, spec))
                .is_empty()
        );
    }

    fn surface(set: &PartSet) -> Option<Vec<String>> {
        set.displays().next().map(|display| match display {
            Display::Body(rows) => rows,
            Display::Bubble(_) => panic!("a surface is drawn on the body"),
        })
    }

    #[test]
    fn a_cathode_writes_one_dot_per_rise_of_write_and_reads_bit_as_a_level() {
        let mut set = cathode();
        for (bit, write) in [(true, false), (false, false), (true, false)] {
            drive(
                &mut set,
                &Screen {
                    addr: 0,
                    bit,
                    write,
                    ..Screen::DARK
                },
            );
        }
        assert_eq!(surface(&set), None, "a bit that rises writes nothing");

        for (addr, write) in [(0, true), (1, true), (2, false), (3, true)] {
            drive(
                &mut set,
                &Screen {
                    addr,
                    bit: true,
                    write,
                    ..Screen::DARK
                },
            );
        }
        assert_eq!(
            surface(&set),
            Some(vec!["⠁⠈⠀⠀".to_string()]),
            "dots 0 and 3 were written on rises; 1 was a hold, 2 a fall"
        );
    }

    #[test]
    fn a_dot_and_a_byte_written_on_one_stage_land_in_declaration_order() {
        let mut set = cathode();
        drive(
            &mut set,
            &Screen {
                addr: 0,
                bit: true,
                write: true,
                byte: 0b0000_0010,
                write_byte: true,
            },
        );
        assert_eq!(
            surface(&set),
            Some(vec!["⠈⠀⠀⠀".to_string()]),
            "the dot lands first, then byte 0 replaces all eight of its dots"
        );
    }

    #[test]
    fn a_design_blanks_a_cathode_and_an_absorbed_one_arrives_lit() {
        let mut lit = cathode();
        drive(
            &mut lit,
            &Screen {
                byte: 0xFF,
                write_byte: true,
                ..Screen::DARK
            },
        );
        assert!(surface(&lit).is_some());
        assert_eq!(surface(&lit.design()), None, "a blueprint shows nothing");

        let mut receiver = PartSet::new();
        receiver.absorb(&lit);
        assert_eq!(
            surface(&receiver),
            surface(&lit),
            "fusion carries the picture"
        );
    }

    #[test]
    fn a_pulse_reaches_only_the_part_it_was_raised_on_and_the_settle_ends_it() {
        let mut set = PartSet::new();
        set.install(Part::AnglerRig);
        set.install(Part::CommandModule);
        let caught = |set: &PartSet| {
            let mut high = None;
            set.report(
                |_, _| 0,
                &WorldView::default().with_bait(1),
                |part, readings| {
                    if part == Part::AnglerRig {
                        high = readings.get("caught");
                    }
                },
            );
            high
        };

        set.pulse(Part::CommandModule);
        assert_eq!(caught(&set), Some(0), "the module's pulse is not the rig's");
        set.pulse(Part::AnglerRig);
        assert_eq!(caught(&set), Some(1));
        set.settle();
        assert_eq!(caught(&set), Some(0));
    }

    #[test]
    fn an_effect_names_the_part_that_raised_it() {
        let mut set = PartSet::new();
        set.install(Part::AnglerRig);
        set.install(Part::CommandModule);
        let fired: Vec<Part> = set
            .rising_edges(|_, _| 1)
            .into_iter()
            .map(|(part, _)| part)
            .collect();
        assert_eq!(fired, vec![Part::CommandModule, Part::AnglerRig]);
    }
}
