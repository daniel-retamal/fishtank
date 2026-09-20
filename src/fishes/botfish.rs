use std::collections::BTreeSet;
use std::f32::consts::TAU;

use rand::RngExt;
use ratatui::style::Color;

use super::chip::Chip;
use super::parts::{
    Bus, ConfigSpec, Display, KeyBinding, OUTPUT_PIN, Part, PartConfig, PartEffect, PartSet, Pin,
    PinDirection, PinMap, PinOwner, PinSpec,
};
use crate::colors::{BLUE, DARK_GRAY, LIGHT_BLUE, LIGHT_RED};
use crate::tank::{ChannelRegistry, Link, Transmission, Wires, WorldView};

pub const BODY_COLOR: Color = DARK_GRAY;
pub const ANTENNA_STALK: char = '‖';
pub const ANTENNA_TIP: char = 'o';
pub const ANTENNA_LENGTH: usize = 2;

const HIGH_COLOR: Color = LIGHT_RED;
const LOW_COLOR: Color = BODY_COLOR;
const BLINK_SPEED: f32 = 6.0;
const PROCESS_TICKS: f32 = 30.0;
const NET_JOIN: &str = ", ";
const RUN_TRIGGER_PREFIX: &str = "/run ";
const QUOTE: &str = "\"";

pub fn level_color(level: bool) -> Color {
    if level { HIGH_COLOR } else { LOW_COLOR }
}

#[derive(Clone)]
struct ScriptRun {
    next_line: usize,
    ticks_left: f32,
}

pub struct DueLine {
    pub path: Vec<usize>,
    pub command: String,
}

#[derive(Clone)]
struct Cast {
    part: Part,
    secs_left: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tackle {
    Bait,
    Landed,
}

enum Target {
    Listen(String),
    Wire(Pin),
    Part(Part),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strike {
    Retuned,
    KnockedLoose(Part),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DueCast {
    pub path: Vec<usize>,
    pub tackle: Tackle,
}

#[derive(Clone)]
pub struct BotfishState {
    pub trigger: String,
    pub script: Vec<String>,
    listens: BTreeSet<String>,
    pins: PinMap,
    parts: PartSet,
    chip: Option<Box<Chip>>,
    run: Option<ScriptRun>,
    cast: Option<Cast>,
    blink_phase: f32,
    sensed: bool,
    emitted: bool,
    printed: bool,
}

impl Default for BotfishState {
    fn default() -> Self {
        Self::new()
    }
}

impl BotfishState {
    pub fn new() -> Self {
        Self {
            trigger: String::new(),
            script: Vec::new(),
            listens: BTreeSet::new(),
            pins: PinMap::new(),
            parts: PartSet::new(),
            chip: None,
            run: None,
            cast: None,
            blink_phase: 0.0,
            sensed: false,
            emitted: false,
            printed: false,
        }
    }

    pub fn run_trigger(fish_name: &str) -> String {
        format!("{RUN_TRIGGER_PREFIX}{fish_name}")
    }

    pub fn run_target(trigger: &str) -> Option<&str> {
        trigger.strip_prefix(RUN_TRIGGER_PREFIX)
    }

    pub fn mark_printed(&mut self) {
        self.printed = true;
    }

    pub fn is_printed(&self) -> bool {
        self.printed
    }

    pub fn rename_references(&mut self, renames: &[(&str, &str)]) {
        self.trigger = renamed_trigger(&self.trigger, renames);
        self.script = self
            .script
            .iter()
            .map(|line| renamed_quotes(line, renames))
            .collect();
        if let Some(chip) = &mut self.chip {
            chip.rename_references(renames);
        }
    }

    pub fn etch(&mut self, chip: Chip) -> Vec<(Part, u32)> {
        let salvaged = self.hardware();
        let wiring: Vec<Pin> = chip.pins().collect();
        *self = Self {
            chip: Some(Box::new(chip)),
            printed: self.printed,
            ..Self::new()
        };
        for pin in wiring {
            self.pins.wire(pin.owner, &pin.name, &pin.name);
        }
        salvaged
    }

    pub fn chip(&self) -> Option<&Chip> {
        self.chip.as_deref()
    }

    pub fn hardware(&self) -> Vec<(Part, u32)> {
        self.parts
            .iter()
            .chain(self.chip.iter().flat_map(|chip| chip.hardware()))
            .collect()
    }

    pub fn namespace(&mut self, internals: &BTreeSet<String>, rename: impl Fn(&str) -> String) {
        self.listens = self
            .listens
            .iter()
            .map(|channel| {
                if internals.contains(channel) {
                    rename(channel)
                } else {
                    channel.clone()
                }
            })
            .collect();
        let rewired: Vec<(Pin, String)> = self
            .declared_pins()
            .into_iter()
            .filter_map(|pin| {
                let bus = self.pins.bus(&pin)?;
                let channel = rename(self.pins.channel_of(&pin)?);
                bus.lines()
                    .iter()
                    .any(|line| internals.contains(line))
                    .then_some((pin, channel))
            })
            .collect();
        for (pin, channel) in rewired {
            self.pins.wire(pin.owner, &pin.name, &channel);
        }
    }

    pub fn design(&self) -> Self {
        Self {
            trigger: self.trigger.clone(),
            script: self.script.clone(),
            listens: self.listens.clone(),
            pins: self.pins.clone(),
            parts: self.parts.design(),
            chip: self.chip.as_ref().map(|chip| Box::new(chip.design())),
            ..Self::new()
        }
    }

    pub fn fuse(&self, donor: &BotfishState) -> Self {
        let own_pins = self.declared_pins();
        let mut fused = self.clone();
        fused.parts.absorb(&donor.parts);
        if fused.chip.is_none() {
            fused.chip = donor.chip.clone();
        }
        fused.printed |= donor.printed;
        let arrived: Vec<Pin> = fused
            .declared_pins()
            .into_iter()
            .filter(|pin| !own_pins.iter().any(|own| own.same_wire(pin)))
            .collect();
        for pin in arrived {
            if let Some(channel) = donor.pins.channel_of(&pin) {
                fused.pins.wire(pin.owner, &pin.name, channel);
            }
        }
        fused
    }

    pub fn listen(&mut self, channel: &str) -> bool {
        let Some(key) = ChannelRegistry::normalize(channel) else {
            return false;
        };
        self.listens.insert(key.into_owned())
    }

    pub fn unlisten(&mut self, channel: &str) -> bool {
        ChannelRegistry::normalize(channel).is_some_and(|key| self.listens.remove(key.as_ref()))
    }

    pub fn listens(&self) -> impl Iterator<Item = &str> {
        self.listens.iter().map(|channel| channel.as_str())
    }

    pub fn hears(&self, channel: &str) -> bool {
        ChannelRegistry::normalize(channel).is_some_and(|key| self.listens.contains(key.as_ref()))
    }

    pub fn wire(&mut self, owner: impl Into<PinOwner>, pin: &str, channel: &str) -> bool {
        self.pins.wire(owner, pin, channel)
    }

    pub fn unwire(&mut self, owner: impl Into<PinOwner>, pin: &str) -> bool {
        self.pins.unwire(owner, pin)
    }

    pub fn pins(&self) -> &PinMap {
        &self.pins
    }

    pub fn declared_pins(&self) -> Vec<Pin> {
        let part_pins = self
            .parts
            .iter()
            .flat_map(|(part, _)| part.pins().iter().map(move |&spec| Pin::of(part, spec)));
        let chip_pins = self.chip.iter().flat_map(|chip| chip.pins());
        let mut declared: Vec<Pin> = Vec::new();
        for pin in std::iter::once(Pin::output())
            .chain(part_pins)
            .chain(chip_pins)
        {
            if !declared.iter().any(|seen| seen.same_wire(&pin)) {
                declared.push(pin);
            }
        }
        declared
    }

    pub fn drive(&mut self, channel: &str) -> bool {
        self.pins.wire(PinOwner::Body, OUTPUT_PIN.name, channel)
    }

    pub fn drives(&self) -> Option<&str> {
        self.pins.channel(PinOwner::Body, OUTPUT_PIN.name)
    }

    fn wired_pins(&self, direction: PinDirection) -> impl Iterator<Item = (Pin, &str)> + '_ {
        self.declared_pins()
            .into_iter()
            .filter(move |pin| pin.direction == direction)
            .filter_map(|pin| self.pins.channel_of(&pin).map(|channel| (pin, channel)))
    }

    fn pin_lines(&self, direction: PinDirection) -> Vec<String> {
        self.wired_pins(direction)
            .filter_map(|(pin, channel)| Bus::new(channel, pin.width))
            .flat_map(|bus| bus.lines())
            .collect()
    }

    pub fn driven_nets(&self) -> Vec<&str> {
        self.wired_pins(PinDirection::Out)
            .map(|(_, channel)| channel)
            .collect()
    }

    pub fn net_label(&self) -> Option<String> {
        let nets = self.driven_nets();
        if nets.is_empty() {
            return None;
        }
        Some(nets.join(NET_JOIN))
    }

    pub fn driven_lines(&self) -> Vec<String> {
        self.pin_lines(PinDirection::Out)
    }

    pub fn fed_lines(&self) -> Vec<String> {
        self.listens
            .iter()
            .cloned()
            .chain(self.pin_lines(PinDirection::In))
            .collect()
    }

    pub fn step<W: Wires>(&mut self, wires: &mut W, world: &WorldView) -> Vec<Transmission> {
        let sensed = self.sense(wires);
        self.react(wires);
        let level = self.emit(sensed);
        let readings = self.readings(wires, world);
        self.parts.settle();
        let mut sent = self.transmissions(wires);
        if let Some(channel) = self.drives() {
            wires.drive(channel, level);
        }
        for (bus, value) in readings {
            bus.drive_into(value, wires);
        }
        if let Some(chip) = &mut self.chip {
            sent.extend(chip.step(&self.pins, wires, world));
        }
        sent
    }

    pub fn transmissions<W: Wires>(&self, wires: &W) -> Vec<Transmission> {
        let pins = &self.pins;
        self.parts.transmissions(|part, spec| {
            let pin = Pin::of(part, *spec);
            pins.channel_of(&pin).map(|_| pins.value(&pin, wires))
        })
    }

    pub fn links(&self) -> Vec<Link> {
        self.parts
            .links()
            .chain(self.chip.iter().flat_map(|chip| chip.links()))
            .collect()
    }

    pub fn sense<W: Wires>(&mut self, wires: &W) -> bool {
        self.sensed = self.listens.iter().any(|channel| wires.level(channel));
        self.sensed
    }

    pub fn is_gate(&self) -> bool {
        self.chip.is_none() && self.declared_pins().len() == 1
    }

    pub fn respond(&mut self, sensed: bool) -> bool {
        self.sensed = sensed;
        self.emit(sensed)
    }

    pub fn emit(&mut self, sensed: bool) -> bool {
        self.emitted = self.parts.modify_output(sensed);
        self.emitted
    }

    pub fn react<W: Wires>(&mut self, wires: &W) {
        let pins = &self.pins;
        let fired = self
            .parts
            .rising_edges(|part, spec| pins.value(&Pin::of(part, *spec), wires));
        for (part, effect) in fired {
            match effect {
                PartEffect::RunScript => self.start(),
                PartEffect::Cast => self.cast(part),
            }
        }
    }

    pub fn reads_world(&self) -> bool {
        self.parts.reads_world() || self.chip.as_ref().is_some_and(|chip| chip.reads_world())
    }

    pub fn readings<W: Wires>(&self, wires: &W, world: &WorldView) -> Vec<(Bus, u32)> {
        let pins = &self.pins;
        let value_of = |part: Part, spec: &PinSpec| pins.value(&Pin::of(part, *spec), wires);
        let mut driven = Vec::new();
        self.parts.report(value_of, world, |part, readings| {
            for &spec in part.pins() {
                let Some(value) = readings.get(spec.name) else {
                    continue;
                };
                if let Some(bus) = self.pins.bus(&Pin::of(part, spec)) {
                    driven.push((bus, value));
                }
            }
        });
        driven
    }

    pub fn displays(&self) -> Vec<Display> {
        self.parts
            .displays()
            .chain(self.chip.iter().flat_map(|chip| chip.displays()))
            .collect()
    }

    pub fn configure(&mut self, part: Part, spec: &ConfigSpec, value: &str) -> bool {
        self.parts.configure(part, spec, value)
    }

    pub fn config(&self, part: Part) -> PartConfig {
        self.parts.config(part)
    }

    pub fn input_level(&self) -> bool {
        self.sensed || self.chip.as_ref().is_some_and(|chip| chip.hears_high())
    }

    pub fn output_level(&self) -> bool {
        self.emitted || self.chip.as_ref().is_some_and(|chip| chip.is_active())
    }

    pub fn irradiate(&mut self, channels: &[String], rng: &mut impl RngExt) -> Option<Strike> {
        let elsewhere = |channel: &str| channels.iter().any(|other| other != channel);
        let mut targets: Vec<Target> = self
            .listens
            .iter()
            .filter(|channel| elsewhere(channel))
            .map(|channel| Target::Listen(channel.clone()))
            .collect();
        for pin in self.declared_pins() {
            if self.pins.channel_of(&pin).is_some_and(elsewhere) {
                targets.push(Target::Wire(pin));
            }
        }
        for (part, count) in self.parts.iter() {
            targets.extend((0..count).map(|_| Target::Part(part)));
        }
        if targets.is_empty() {
            return None;
        }
        match targets.swap_remove(rng.random_range(0..targets.len())) {
            Target::Listen(old) => {
                let new = crosstalk(&old, channels, rng);
                self.unlisten(&old);
                self.listen(&new);
                Some(Strike::Retuned)
            }
            Target::Wire(pin) => {
                let old = self.pins.channel_of(&pin).unwrap_or_default().to_string();
                let new = crosstalk(&old, channels, rng);
                self.pins.wire(pin.owner, &pin.name, &new);
                Some(Strike::Retuned)
            }
            Target::Part(part) => {
                self.parts.remove(part);
                Some(Strike::KnockedLoose(part))
            }
        }
    }

    pub fn install(&mut self, part: Part) -> bool {
        self.parts.install(part)
    }

    pub fn uninstall(&mut self, part: Part) -> bool {
        self.parts.remove(part)
    }

    pub fn parts(&self) -> &PartSet {
        &self.parts
    }

    pub fn program(&mut self, trigger: String, script: Vec<String>) {
        self.trigger = trigger;
        self.script = script;
        self.run = None;
    }

    pub fn is_processing(&self) -> bool {
        self.run.is_some() || self.chip.as_ref().is_some_and(|chip| chip.is_processing())
    }

    pub fn is_wired(&self) -> bool {
        !self.trigger.is_empty()
            || !self.script.is_empty()
            || !self.listens.is_empty()
            || !self.pins.is_empty()
            || !self.parts.is_empty()
            || self.chip.is_some()
    }

    pub fn responds_to(&self, speech: &str) -> bool {
        !self.trigger.is_empty() && self.trigger == speech
    }

    pub fn hear(&mut self, speech: &str) {
        if self.responds_to(speech) {
            self.start();
        }
        self.parts.hear(speech);
        if let Some(chip) = &mut self.chip {
            chip.hear(speech);
        }
    }

    pub fn key(&mut self, key: &str, down: bool) {
        self.parts.key(key, down);
        if let Some(chip) = &mut self.chip {
            chip.key(key, down);
        }
    }

    pub fn bindings(&self) -> Vec<KeyBinding> {
        let mut bound: Vec<KeyBinding> = Vec::new();
        let chip_bindings = self.chip.iter().flat_map(|chip| chip.bindings());
        for binding in self.parts.bindings().chain(chip_bindings) {
            if !bound.contains(&binding) {
                bound.push(binding);
            }
        }
        bound
    }

    pub fn start(&mut self) {
        if self.run.is_some() || self.script.is_empty() {
            return;
        }
        self.run = Some(ScriptRun {
            next_line: 0,
            ticks_left: PROCESS_TICKS,
        });
    }

    fn cast(&mut self, part: Part) {
        if self.cast.is_some() {
            return;
        }
        self.cast = Some(Cast {
            part,
            secs_left: None,
        });
    }

    pub fn due_casts(&mut self, dt: f32) -> Vec<DueCast> {
        let mut due = Vec::new();
        if let Some(cast) = &mut self.cast {
            match &mut cast.secs_left {
                None => due.push(DueCast {
                    path: Vec::new(),
                    tackle: Tackle::Bait,
                }),
                Some(left) => {
                    *left -= dt;
                    if *left <= 0.0 {
                        due.push(DueCast {
                            path: Vec::new(),
                            tackle: Tackle::Landed,
                        });
                    }
                }
            }
        }
        if let Some(chip) = &mut self.chip {
            due.extend(chip.due_casts(dt));
        }
        due
    }

    pub fn lower_line(&mut self, path: &[usize], secs: f32) {
        let Some((&index, rest)) = path.split_first() else {
            if let Some(cast) = &mut self.cast {
                cast.secs_left = Some(secs);
            }
            return;
        };
        if let Some(chip) = &mut self.chip {
            chip.lower_line(index, rest, secs);
        }
    }

    pub fn reel_in(&mut self, path: &[usize], landed: bool) {
        let Some((&index, rest)) = path.split_first() else {
            let Some(cast) = self.cast.take() else {
                return;
            };
            if landed {
                self.parts.pulse(cast.part);
            }
            return;
        };
        if let Some(chip) = &mut self.chip {
            chip.reel_in(index, rest, landed);
        }
    }

    pub fn tick_blink(&mut self, dt: f32) {
        self.blink_phase = (self.blink_phase + dt * BLINK_SPEED).rem_euclid(TAU);
    }

    pub fn due_command(&mut self, coffee_stacks: u32) -> Option<String> {
        let run = self.run.as_mut()?;
        run.ticks_left -= 1.0 + coffee_stacks as f32;
        if run.ticks_left > 0.0 {
            return None;
        }
        if run.next_line >= self.script.len() {
            self.run = None;
            return None;
        }
        let command = self.script[run.next_line].clone();
        run.next_line += 1;
        run.ticks_left = PROCESS_TICKS;
        Some(command)
    }

    pub fn report(&mut self, success: bool) {
        if !success {
            self.run = None;
        }
    }

    pub fn due_lines(&mut self, coffee_stacks: u32) -> Vec<DueLine> {
        let mut due: Vec<DueLine> = self
            .due_command(coffee_stacks)
            .map(|command| DueLine {
                path: Vec::new(),
                command,
            })
            .into_iter()
            .collect();
        if let Some(chip) = &mut self.chip {
            due.extend(chip.due_lines(coffee_stacks));
        }
        due
    }

    pub fn report_line(&mut self, path: &[usize], success: bool) {
        let Some((&index, rest)) = path.split_first() else {
            self.report(success);
            return;
        };
        if let Some(chip) = &mut self.chip {
            chip.report_line(index, rest, success);
        }
    }

    pub fn eye_color(&self) -> Color {
        level_color(self.input_level())
    }

    pub fn tip_color(&self) -> Color {
        if !self.is_processing() {
            return level_color(self.output_level());
        }
        if self.blink_phase.sin() > 0.0 {
            return LIGHT_BLUE;
        }
        BLUE
    }
}

fn crosstalk(old: &str, channels: &[String], rng: &mut impl RngExt) -> String {
    let others: Vec<&String> = channels.iter().filter(|other| *other != old).collect();
    others[rng.random_range(0..others.len())].clone()
}

fn renamed<'a>(name: &str, renames: &[(&str, &'a str)]) -> Option<&'a str> {
    renames
        .iter()
        .find(|(old, _)| old.eq_ignore_ascii_case(name))
        .map(|(_, new)| *new)
}

fn renamed_quotes(line: &str, renames: &[(&str, &str)]) -> String {
    let pieces: Vec<&str> = line.split(QUOTE).collect();
    let last = pieces.len() - 1;
    pieces
        .iter()
        .enumerate()
        .map(|(index, piece)| {
            let quoted = index % 2 == 1 && index < last;
            match quoted.then(|| renamed(piece, renames)).flatten() {
                Some(new) => new,
                None => piece,
            }
        })
        .collect::<Vec<_>>()
        .join(QUOTE)
}

fn renamed_trigger(trigger: &str, renames: &[(&str, &str)]) -> String {
    let target = BotfishState::run_target(trigger).and_then(|name| renamed(name, renames));
    match target {
        Some(new) => BotfishState::run_trigger(new),
        None => renamed_quotes(trigger, renames),
    }
}

#[cfg(test)]
mod probe_tests {
    use super::*;

    #[test]
    fn a_dark_probe_is_the_colour_of_the_body_it_sits_on() {
        assert_eq!(
            level_color(false),
            BODY_COLOR,
            "an eye or antenna that is off must disappear into the fish, not glow dim red"
        );
        assert_eq!(level_color(true), LIGHT_RED, "and light up when it is high");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tank::WorldSignal;
    use rand::{SeedableRng, rngs::SmallRng};

    fn programmed(script: &[&str]) -> BotfishState {
        let mut bot = BotfishState::new();
        bot.program(
            "ping".to_string(),
            script.iter().map(|s| s.to_string()).collect(),
        );
        bot
    }

    fn drain_one(bot: &mut BotfishState) -> Option<String> {
        for _ in 0..PROCESS_TICKS as u32 {
            if let Some(cmd) = bot.due_command(0) {
                return Some(cmd);
            }
        }
        None
    }

    #[test]
    fn responds_only_to_exact_trigger() {
        let bot = programmed(&["/feed 5"]);
        assert!(bot.responds_to("ping"));
        assert!(!bot.responds_to("pin"));
        assert!(!bot.responds_to("ping "));
    }

    #[test]
    fn empty_trigger_never_responds() {
        let bot = BotfishState::new();
        assert!(!bot.responds_to(""));
    }

    #[test]
    fn a_fresh_bot_is_unwired() {
        assert!(!BotfishState::new().is_wired());
    }

    #[test]
    fn a_programmed_bot_is_wired() {
        assert!(programmed(&["/feed 5"]).is_wired());
    }

    #[test]
    fn a_trigger_alone_wires_the_bot() {
        let mut bot = BotfishState::new();
        bot.program("ping".to_string(), Vec::new());
        assert!(bot.is_wired());
    }

    #[test]
    fn listening_to_a_channel_wires_the_bot() {
        let mut bot = BotfishState::new();
        assert!(bot.listen("clk"));
        assert!(bot.is_wired(), "a fish on a wire is part of a circuit");
    }

    #[test]
    fn driving_a_channel_wires_the_bot() {
        let mut bot = BotfishState::new();
        assert!(bot.drive("carry"));
        assert_eq!(bot.drives(), Some("carry"));
        assert!(bot.is_wired());
    }

    #[test]
    fn installing_a_part_wires_the_bot() {
        let mut bot = BotfishState::new();
        assert!(bot.install(Part::InverterCoil));
        assert!(bot.is_wired(), "a fish with hardware in it is not idle");
    }

    #[test]
    fn a_channel_is_heard_however_it_was_typed() {
        let mut bot = BotfishState::new();
        bot.listen("  Carry In ");
        assert!(bot.hears("CARRY IN"));
        assert_eq!(bot.listens().collect::<Vec<_>>(), vec!["carry in"]);
    }

    #[test]
    fn listening_twice_to_one_channel_is_one_connection() {
        let mut bot = BotfishState::new();
        assert!(bot.listen("clk"));
        assert!(!bot.listen("CLK"), "already connected");
        assert_eq!(bot.listens().count(), 1);
    }

    #[test]
    fn a_blank_channel_connects_nothing() {
        let mut bot = BotfishState::new();
        assert!(!bot.listen("   "));
        assert!(!bot.drive(""));
        assert!(!bot.is_wired());
    }

    #[test]
    fn disconnecting_the_last_wire_leaves_the_bot_unwired() {
        let mut bot = BotfishState::new();
        bot.listen("clk");
        bot.drive("q");
        bot.install(Part::DelaySpool);

        assert!(bot.unlisten("clk"));
        assert!(bot.unwire(PinOwner::Body, OUTPUT_PIN.name));
        assert!(bot.uninstall(Part::DelaySpool));

        assert!(!bot.is_wired(), "a stripped bot goes back to being a fish");
    }

    #[test]
    fn listened_channels_come_back_sorted_for_a_stable_readout() {
        let mut bot = BotfishState::new();
        bot.listen("nripe");
        bot.listen("clkd");
        bot.listen("a");
        assert_eq!(
            bot.listens().collect::<Vec<_>>(),
            vec!["a", "clkd", "nripe"]
        );
    }

    #[test]
    fn parts_and_wiring_survive_being_captured_for_fusion() {
        let mut bot = BotfishState::new();
        bot.listen("x");
        bot.drive("nx");
        bot.install(Part::InverterCoil);
        bot.install(Part::DelaySpool);
        bot.install(Part::DelaySpool);

        let carried = bot.clone();

        assert!(carried.hears("x"));
        assert_eq!(carried.drives(), Some("nx"));
        assert_eq!(carried.parts().count(Part::DelaySpool), 2);
        assert!(carried.parts().has(Part::InverterCoil));
    }

    #[test]
    fn a_design_keeps_the_wiring_the_parts_and_the_program() {
        let mut bot = programmed(&["/feed 5"]);
        bot.listen("x");
        bot.drive("nx");
        bot.install(Part::InverterCoil);
        bot.install(Part::DelaySpool);
        bot.install(Part::DelaySpool);

        let design = bot.design();

        assert!(design.hears("x"));
        assert_eq!(design.drives(), Some("nx"));
        assert!(design.parts().has(Part::InverterCoil));
        assert_eq!(design.parts().count(Part::DelaySpool), 2);
        assert_eq!(design.trigger, "ping");
        assert_eq!(design.script, vec!["/feed 5".to_string()]);
    }

    #[test]
    fn a_design_forgets_everything_the_running_fish_was_doing() {
        let mut bot = programmed(&["/feed 5"]);
        bot.listen("x");
        bot.install(Part::DelaySpool);
        bot.sense(&board_with("x", true));
        bot.emit(true);
        bot.emit(true);
        bot.start();

        let mut design = bot.design();

        assert!(!design.is_processing(), "a blueprint is not mid-script");
        assert!(!design.input_level());
        assert!(!design.output_level());
        assert!(
            !design.emit(false),
            "a fresh spool holds no stage the original had already taken in"
        );
        assert!(bot.emit(false), "while the original still delivers it");
    }

    #[test]
    fn a_bare_fish_declares_only_its_own_output_pin() {
        assert_eq!(BotfishState::new().declared_pins(), vec![Pin::output()]);
    }

    #[test]
    fn declared_pins_are_the_output_plus_every_installed_parts_pins() {
        let mut bot = BotfishState::new();
        bot.install(Part::InverterCoil);
        bot.install(Part::DelaySpool);
        assert_eq!(
            bot.declared_pins(),
            vec![Pin::output()],
            "fabric parts bolt onto the fish and expose no pins of their own"
        );
    }

    #[test]
    fn processing_always_implies_wired() {
        let mut bot = programmed(&["/feed 5"]);
        bot.start();
        assert!(bot.is_processing());
        assert!(bot.is_wired());
    }

    #[test]
    fn start_requires_a_script() {
        let mut bot = programmed(&[]);
        bot.start();
        assert!(!bot.is_processing());
    }

    #[test]
    fn first_command_fires_after_exactly_process_ticks() {
        let mut bot = programmed(&["/feed 5"]);
        bot.start();
        for _ in 0..(PROCESS_TICKS as u32 - 1) {
            assert!(bot.due_command(0).is_none());
        }
        assert_eq!(bot.due_command(0).as_deref(), Some("/feed 5"));
    }

    #[test]
    fn coffee_accelerates_processing() {
        let mut bot = programmed(&["/feed 5"]);
        bot.start();
        assert_eq!(
            bot.due_command(PROCESS_TICKS as u32),
            Some("/feed 5".to_string())
        );
    }

    #[test]
    fn runs_lines_in_order_then_goes_idle() {
        let mut bot = programmed(&["/feed 1", "/feed 2"]);
        bot.start();
        assert_eq!(drain_one(&mut bot).as_deref(), Some("/feed 1"));
        bot.report(true);
        assert_eq!(drain_one(&mut bot).as_deref(), Some("/feed 2"));
        bot.report(true);
        assert!(bot.is_processing());
        assert!(drain_one(&mut bot).is_none());
        assert!(!bot.is_processing());
    }

    #[test]
    fn failure_aborts_the_script() {
        let mut bot = programmed(&["/bad", "/feed 2"]);
        bot.start();
        assert_eq!(drain_one(&mut bot).as_deref(), Some("/bad"));
        bot.report(false);
        assert!(!bot.is_processing());
        assert!(drain_one(&mut bot).is_none());
    }

    fn board_with(channel: &str, level: bool) -> ChannelRegistry {
        let mut channels = ChannelRegistry::new();
        channels.set_level(channel, level);
        channels
    }

    #[test]
    fn an_unwired_bot_sits_dark() {
        let bot = BotfishState::new();
        assert_eq!(bot.eye_color(), LOW_COLOR);
        assert_eq!(bot.tip_color(), LOW_COLOR);
    }

    #[test]
    fn a_high_listened_channel_lights_the_eyes() {
        let mut bot = BotfishState::new();
        bot.listen("x");

        bot.sense(&board_with("x", false));
        assert_eq!(bot.eye_color(), LOW_COLOR, "nothing is arriving");

        bot.sense(&board_with("x", true));
        assert_eq!(bot.eye_color(), HIGH_COLOR, "a signal is arriving");
    }

    #[test]
    fn a_high_output_lights_the_antenna_tip() {
        let mut bot = BotfishState::new();

        bot.emit(false);
        assert_eq!(bot.tip_color(), LOW_COLOR, "nothing is leaving");

        bot.emit(true);
        assert_eq!(bot.tip_color(), HIGH_COLOR, "a signal is leaving");
    }

    #[test]
    fn a_coiled_bot_lights_its_antenna_while_its_eyes_stay_dark() {
        let mut bot = BotfishState::new();
        bot.listen("x");
        bot.install(Part::InverterCoil);

        let sensed = bot.sense(&board_with("x", false));
        bot.emit(sensed);

        assert_eq!(bot.eye_color(), LOW_COLOR, "no input arrived");
        assert_eq!(bot.tip_color(), HIGH_COLOR, "but the denial left");
    }

    #[test]
    fn a_spool_lights_the_eyes_a_stage_before_the_antenna() {
        let mut bot = BotfishState::new();
        bot.listen("x");
        bot.install(Part::DelaySpool);
        let high = board_with("x", true);

        let sensed = bot.sense(&high);
        bot.emit(sensed);
        assert_eq!(bot.eye_color(), HIGH_COLOR);
        assert_eq!(bot.tip_color(), LOW_COLOR, "the spool still holds it");

        let sensed = bot.sense(&high);
        bot.emit(sensed);
        assert_eq!(bot.tip_color(), HIGH_COLOR, "a stage later it leaves");
    }

    #[test]
    fn the_antenna_blinks_blue_while_a_script_steps() {
        let mut bot = programmed(&["/feed 5"]);
        bot.emit(true);
        bot.start();

        assert!(
            matches!(bot.tip_color(), BLUE | LIGHT_BLUE),
            "processing owns the antenna, whatever the output level is"
        );
    }

    #[test]
    fn the_eyes_keep_showing_the_input_while_a_script_steps() {
        let mut bot = programmed(&["/feed 5"]);
        bot.listen("x");
        bot.sense(&board_with("x", true));
        bot.start();

        assert_eq!(bot.eye_color(), HIGH_COLOR);
    }

    #[test]
    fn levels_survive_being_captured_for_fusion() {
        let mut bot = BotfishState::new();
        bot.listen("x");
        bot.sense(&board_with("x", true));
        bot.emit(true);

        let carried = bot.clone();

        assert!(carried.input_level());
        assert!(carried.output_level());
    }

    fn run_to_completion(bot: &mut BotfishState) {
        while drain_one(bot).is_some() {
            bot.report(true);
        }
    }

    const FIRE_PIN_NAME: &str = "fire";
    const FIRE_CHANNEL: &str = "go";

    fn commanded(script: &[&str]) -> BotfishState {
        let mut bot = programmed(script);
        assert!(bot.install(Part::CommandModule));
        assert!(bot.wire(Part::CommandModule, FIRE_PIN_NAME, FIRE_CHANNEL));
        bot
    }

    #[test]
    fn a_rising_fire_wire_runs_the_program() {
        let mut bot = commanded(&["/feed 5"]);

        bot.react(&board_with(FIRE_CHANNEL, false));
        assert!(!bot.is_processing(), "a low wire commands nothing");

        bot.react(&board_with(FIRE_CHANNEL, true));
        assert!(bot.is_processing(), "the rise runs the program");
    }

    #[test]
    fn a_wire_that_stays_high_never_runs_the_program_twice() {
        let mut bot = commanded(&["/feed 5"]);
        let high = board_with(FIRE_CHANNEL, true);

        bot.react(&high);
        run_to_completion(&mut bot);
        assert!(!bot.is_processing(), "the one line has run");

        bot.react(&high);
        assert!(
            !bot.is_processing(),
            "a level that never fell cannot rise again"
        );
    }

    #[test]
    fn a_falling_wire_runs_nothing() {
        let mut bot = commanded(&["/feed 5"]);
        bot.react(&board_with(FIRE_CHANNEL, true));
        run_to_completion(&mut bot);

        bot.react(&board_with(FIRE_CHANNEL, false));

        assert!(!bot.is_processing(), "a fall is not a trigger");
    }

    #[test]
    fn a_wire_that_falls_and_rises_again_runs_the_program_again() {
        let mut bot = commanded(&["/feed 5"]);
        bot.react(&board_with(FIRE_CHANNEL, true));
        run_to_completion(&mut bot);
        bot.react(&board_with(FIRE_CHANNEL, false));

        bot.react(&board_with(FIRE_CHANNEL, true));

        assert!(bot.is_processing(), "a fresh rise is a fresh run");
    }

    #[test]
    fn a_fish_without_a_command_module_ignores_every_wire() {
        let mut bot = programmed(&["/feed 5"]);
        bot.listen(FIRE_CHANNEL);

        bot.react(&board_with(FIRE_CHANNEL, true));

        assert!(
            !bot.is_processing(),
            "a bare fish is a gate; the module is what obeys"
        );
    }

    #[test]
    fn a_module_wired_to_nothing_never_runs_the_program() {
        let mut bot = programmed(&["/feed 5"]);
        bot.install(Part::CommandModule);

        bot.react(&board_with(FIRE_CHANNEL, true));

        assert!(!bot.is_processing(), "an unwired pin reads low forever");
    }

    #[test]
    fn a_command_module_survives_being_captured_for_fusion() {
        let bot = commanded(&["/feed 5"]);
        let mut carried = bot.clone();

        carried.react(&board_with(FIRE_CHANNEL, true));

        assert!(carried.parts().has(Part::CommandModule));
        assert!(
            carried.is_processing(),
            "the fused host still obeys the wire"
        );
    }

    #[test]
    fn cannot_restart_while_processing() {
        let mut bot = programmed(&["/feed 1"]);
        bot.start();
        let _ = bot.due_command(0);
        bot.start();
        assert!(bot.is_processing());
    }

    fn lines(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn not_chip() -> Chip {
        let mut not = BotfishState::new();
        not.listen("x");
        not.drive("nx");
        not.install(Part::InverterCoil);
        Chip::new("Not".to_string(), vec![not], lines(&["x"]), lines(&["nx"]))
    }

    fn stage(bot: &mut BotfishState, channels: &mut ChannelRegistry) {
        bot.step(channels, &WorldView::default());
        channels.commit();
    }

    #[test]
    fn etching_burns_away_the_old_wiring_and_hands_back_its_parts() {
        let mut bot = programmed(&["/feed 5"]);
        bot.listen("old");
        bot.install(Part::DelaySpool);
        bot.install(Part::DelaySpool);
        bot.install(Part::InverterCoil);
        bot.mark_printed();

        let salvaged = bot.etch(not_chip());

        assert_eq!(
            salvaged,
            vec![(Part::InverterCoil, 1), (Part::DelaySpool, 2)]
        );
        assert!(bot.trigger.is_empty() && bot.script.is_empty());
        assert!(!bot.hears("old"));
        assert!(bot.parts().is_empty());
        assert!(bot.is_wired(), "a chip is a circuit");
        assert!(bot.is_printed(), "a printed fish stays worthless");
    }

    #[test]
    fn a_chip_declares_its_pins_and_wires_each_to_the_channel_of_its_name() {
        let mut bot = BotfishState::new();
        bot.etch(not_chip());

        let declared: Vec<(String, PinDirection)> = bot
            .declared_pins()
            .into_iter()
            .map(|pin| (pin.name.into_owned(), pin.direction))
            .collect();

        assert_eq!(
            declared,
            vec![
                ("out".to_string(), PinDirection::Out),
                ("x".to_string(), PinDirection::In),
                ("nx".to_string(), PinDirection::Out),
            ]
        );
        assert_eq!(bot.pins().channel(PinOwner::Body, "x"), Some("x"));
        assert_eq!(bot.fed_lines(), vec!["x".to_string()]);
        assert_eq!(bot.driven_lines(), vec!["nx".to_string()]);
    }

    #[test]
    fn a_chip_reads_its_input_pins_and_drives_its_output_pins() {
        let mut bot = BotfishState::new();
        bot.etch(not_chip());
        bot.wire(PinOwner::Body, "nx", "denied");
        let mut channels = ChannelRegistry::new();

        stage(&mut bot, &mut channels);
        assert!(channels.level("denied"), "a low input is denied high");
        assert!(bot.output_level(), "and the antenna shows the work inside");
        assert!(!bot.input_level());

        channels.set_level("x", true);
        stage(&mut bot, &mut channels);
        assert!(!channels.level("denied"));
        assert!(bot.input_level(), "the eyes light on a high chip input");
        assert!(
            !channels.contains("nx"),
            "a rewired pin drives only its wire"
        );
    }

    #[test]
    fn a_design_of_an_etched_fish_forgets_what_its_chip_was_doing() {
        let mut bot = BotfishState::new();
        bot.etch(not_chip());
        let mut channels = ChannelRegistry::new();
        stage(&mut bot, &mut channels);
        assert!(bot.output_level());

        let design = bot.design();

        assert!(!design.output_level(), "a blueprint's chip holds no levels");
        assert_eq!(design.chip().map(Chip::name), Some("Not"));
    }

    #[test]
    fn a_script_inside_a_chip_reports_back_to_the_line_that_ran_it() {
        let mut inner = programmed(&["/bad", "/feed 2"]);
        inner.start();
        let mut bot = BotfishState::new();
        bot.etch(Chip::new(
            "Runner".to_string(),
            vec![inner],
            BTreeSet::new(),
            BTreeSet::new(),
        ));
        assert!(bot.is_processing(), "the host blinks while its chip runs");

        let due = (0..PROCESS_TICKS as u32)
            .flat_map(|_| bot.due_lines(0))
            .next()
            .expect("the inner line comes due");
        assert_eq!(due.path, vec![0]);
        assert_eq!(due.command, "/bad");

        bot.report_line(&due.path, false);

        assert!(!bot.is_processing(), "the failure aborted the inner script");
    }

    fn receiver() -> BotfishState {
        let mut bot = commanded(&["/feed 1"]);
        bot.listen("a");
        bot.drive("q");
        bot.install(Part::InverterCoil);
        bot.install(Part::DelaySpool);
        bot
    }

    fn donor() -> BotfishState {
        let mut bot = programmed(&["/give cash"]);
        bot.trigger = "wake".to_string();
        bot.listen("b");
        bot.drive("nq");
        bot.install(Part::InverterCoil);
        bot.install(Part::DelaySpool);
        bot.install(Part::DelaySpool);
        bot.install(Part::CommandModule);
        bot.wire(Part::CommandModule, FIRE_PIN_NAME, "elsewhere");
        bot.install(Part::StartleNerve);
        bot.wire(Part::StartleNerve, "birth", "born");
        bot
    }

    #[test]
    fn fusing_unions_the_parts_and_only_spools_stack() {
        let fused = receiver().fuse(&donor());

        assert_eq!(
            fused.parts().iter().collect::<Vec<_>>(),
            vec![
                (Part::InverterCoil, 1),
                (Part::DelaySpool, 3),
                (Part::StartleNerve, 1),
                (Part::CommandModule, 1)
            ]
        );
    }

    #[test]
    fn a_fused_fish_keeps_the_receivers_wiring_and_program() {
        let fused = receiver().fuse(&donor());

        assert!(fused.hears("a") && !fused.hears("b"));
        assert_eq!(fused.drives(), Some("q"));
        assert_eq!(fused.trigger, "ping");
        assert_eq!(fused.script, vec!["/feed 1".to_string()]);
        assert_eq!(
            fused.pins().channel(Part::CommandModule, FIRE_PIN_NAME),
            Some(FIRE_CHANNEL),
            "a pin both halves declare stays on the receiver's wire"
        );
    }

    #[test]
    fn a_pin_only_the_donors_parts_declare_keeps_the_donors_wire() {
        let fused = receiver().fuse(&donor());

        assert_eq!(
            fused.pins().channel(Part::StartleNerve, "birth"),
            Some("born")
        );
        assert_eq!(
            fused.fed_lines(),
            vec!["a".to_string(), FIRE_CHANNEL.to_string()]
        );
        assert_eq!(
            fused.driven_lines(),
            vec!["q".to_string(), "born".to_string()],
            "the sense now drives from the receiver's body"
        );
    }

    #[test]
    fn a_sense_fused_into_a_command_module_senses_and_acts() {
        let mut sense = BotfishState::new();
        sense.install(Part::StartleNerve);
        sense.wire(Part::StartleNerve, "birth", FIRE_CHANNEL);
        let mut fused = commanded(&["/feed 1"]).fuse(&sense);
        let mut channels = ChannelRegistry::new();
        let birth = WorldView::unobserved(BTreeSet::from([WorldSignal::Birth]));

        fused.step(&mut channels, &birth);
        channels.commit();
        assert!(
            channels.level(FIRE_CHANNEL),
            "the fused sense drives the wire"
        );
        stage(&mut fused, &mut channels);

        assert!(fused.is_processing(), "and the fused module obeys it");
    }

    #[test]
    fn the_receivers_chip_wins_and_a_bare_receiver_takes_the_donors() {
        let mut etched_receiver = BotfishState::new();
        etched_receiver.etch(not_chip());
        let mut etched_donor = BotfishState::new();
        etched_donor.etch(Chip::new(
            "Other".to_string(),
            Vec::new(),
            BTreeSet::new(),
            BTreeSet::new(),
        ));

        let both = etched_receiver.fuse(&etched_donor);
        assert_eq!(both.chip().map(Chip::name), Some("Not"));

        let mut chipless = BotfishState::new();
        chipless.listen("a");
        let adopted = chipless.fuse(&etched_receiver);
        assert_eq!(adopted.chip().map(Chip::name), Some("Not"));
        assert_eq!(
            adopted.pins().channel(PinOwner::Body, "x"),
            Some("x"),
            "the adopted chip's pins arrive wired"
        );
        assert!(adopted.hears("a"));
    }

    #[test]
    fn a_printed_half_makes_the_fused_circuit_printed() {
        let mut printed = BotfishState::new();
        printed.mark_printed();

        assert!(receiver().fuse(&printed).is_printed());
        assert!(printed.fuse(&receiver()).is_printed());
        assert!(!receiver().fuse(&donor()).is_printed());
    }

    const KEYBOARD: &str = "kbd";
    const STROBE_CHANNEL: &str = "clk";
    const KEYBOARD_WIDTH: u8 = 8;

    fn ear() -> BotfishState {
        let mut bot = BotfishState::new();
        assert!(bot.install(Part::Cochlea));
        bot.wire(Part::Cochlea, "char", KEYBOARD);
        bot.wire(Part::Cochlea, "strobe", STROBE_CHANNEL);
        bot.wire(Part::Cochlea, "ready", "rdy");
        bot.wire(Part::Cochlea, "done", "fin");
        bot
    }

    fn keyboard(channels: &ChannelRegistry) -> u32 {
        (0..KEYBOARD_WIDTH)
            .filter(|bit| channels.level(&format!("{KEYBOARD}{bit}")))
            .map(|bit| 1u32 << bit)
            .sum()
    }

    fn clocked_out(
        bot: &mut BotfishState,
        channels: &mut ChannelRegistry,
        bytes: usize,
    ) -> Vec<u8> {
        let mut clocked = Vec::new();
        for _ in 0..bytes {
            channels.set_level(STROBE_CHANNEL, false);
            stage(bot, channels);
            channels.set_level(STROBE_CHANNEL, true);
            stage(bot, channels);
            clocked.push(keyboard(channels) as u8);
        }
        clocked
    }

    #[test]
    fn a_fish_with_a_cochlea_latches_what_it_hears_and_clocks_it_onto_the_bus() {
        let mut bot = ear();
        let mut channels = ChannelRegistry::new();
        stage(&mut bot, &mut channels);
        assert!(!channels.level("rdy"), "nothing has been said");

        bot.hear("hi");
        stage(&mut bot, &mut channels);
        assert!(channels.level("rdy"));

        assert_eq!(clocked_out(&mut bot, &mut channels, 2), b"hi".to_vec());
        assert!(!channels.level("rdy"));
        assert!(channels.level("fin"));
    }

    #[test]
    fn a_trigger_and_a_cochlea_hear_the_same_line() {
        let mut bot = ear();
        bot.program("ping".to_string(), vec!["/feed 1".to_string()]);

        bot.hear("ping");

        assert!(bot.is_processing(), "the trigger still fires");
        let mut channels = ChannelRegistry::new();
        assert_eq!(clocked_out(&mut bot, &mut channels, 4), b"ping".to_vec());
    }

    #[test]
    fn an_etched_cochlea_hears_through_its_host() {
        let mut bot = BotfishState::new();
        let outputs: BTreeSet<String> = (0..KEYBOARD_WIDTH)
            .map(|bit| format!("{KEYBOARD}{bit}"))
            .chain(["fin".to_string()])
            .collect();
        bot.etch(Chip::new(
            "Ear".to_string(),
            vec![ear()],
            lines(&[STROBE_CHANNEL]),
            outputs,
        ));
        let mut channels = ChannelRegistry::new();

        bot.hear("ok");

        assert_eq!(clocked_out(&mut bot, &mut channels, 2), b"ok".to_vec());
        assert!(channels.level("fin"));
    }

    #[test]
    fn a_design_of_a_fish_with_a_cochlea_has_heard_nothing() {
        let mut bot = ear();
        bot.hear("hi");
        let mut design = bot.design();
        let mut channels = ChannelRegistry::new();
        stage(&mut design, &mut channels);
        assert!(!channels.level("rdy"));
        assert_eq!(design.pins().channel(Part::Cochlea, "char"), Some(KEYBOARD));
    }

    const WRITE_CHANNEL: &str = "wr";

    fn echo() -> BotfishState {
        let mut bot = ear();
        assert!(bot.install(Part::GlyphPanel));
        bot.wire(Part::GlyphPanel, "char", KEYBOARD);
        bot.wire(Part::GlyphPanel, "write", WRITE_CHANNEL);
        bot
    }

    fn pulse(bot: &mut BotfishState, channels: &mut ChannelRegistry, channel: &str) {
        channels.set_level(channel, true);
        stage(bot, channels);
        channels.set_level(channel, false);
        stage(bot, channels);
    }

    fn panel(bot: &BotfishState) -> Vec<String> {
        let displays = bot.displays();
        assert!(displays.len() <= 1, "one panel on this fish");
        displays
            .first()
            .map(Display::rows)
            .unwrap_or_default()
            .iter()
            .map(|row| row.trim_end().to_string())
            .collect()
    }

    #[test]
    fn a_cochlea_and_a_glyph_panel_on_one_fish_own_a_char_pin_each() {
        let bot = echo();
        let chars: Vec<PinOwner> = bot
            .declared_pins()
            .into_iter()
            .filter(|pin| pin.name == "char")
            .map(|pin| pin.owner)
            .collect();
        assert_eq!(
            chars,
            vec![
                PinOwner::Part(Part::Cochlea),
                PinOwner::Part(Part::GlyphPanel)
            ],
            "one name, two pins, two owners"
        );
        assert!(bot.driven_lines().contains(&format!("{KEYBOARD}0")));
        assert!(bot.fed_lines().contains(&format!("{KEYBOARD}0")));
    }

    #[test]
    fn a_glyph_panel_writes_what_its_char_bus_carries_at_the_rise_of_write() {
        let mut bot = echo();
        let mut channels = ChannelRegistry::new();
        bot.hear("hi");
        for _ in 0.."hi".len() {
            pulse(&mut bot, &mut channels, STROBE_CHANNEL);
            pulse(&mut bot, &mut channels, WRITE_CHANNEL);
        }
        assert_eq!(panel(&bot), vec!["hi".to_string(), String::new()]);
    }

    #[test]
    fn a_panel_holds_its_content_with_no_signal_held() {
        let mut bot = echo();
        let mut channels = ChannelRegistry::new();
        bot.hear("ok");
        pulse(&mut bot, &mut channels, STROBE_CHANNEL);
        pulse(&mut bot, &mut channels, WRITE_CHANNEL);
        let written = panel(&bot);
        for _ in 0..16 {
            stage(&mut bot, &mut channels);
        }
        assert_eq!(panel(&bot), written);
        assert_eq!(written[0], "o");
    }

    #[test]
    fn clear_blanks_the_panel_and_takes_its_bubble_away() {
        let mut bot = echo();
        bot.wire(Part::GlyphPanel, "clear", "cls");
        let mut channels = ChannelRegistry::new();
        bot.hear("x");
        pulse(&mut bot, &mut channels, STROBE_CHANNEL);
        pulse(&mut bot, &mut channels, WRITE_CHANNEL);
        assert!(!bot.displays().is_empty());

        pulse(&mut bot, &mut channels, "cls");

        assert!(bot.displays().is_empty(), "a blank panel draws no bubble");
    }

    #[test]
    fn a_design_of_a_fish_with_a_glyph_panel_shows_nothing() {
        let mut bot = echo();
        let mut channels = ChannelRegistry::new();
        bot.hear("x");
        pulse(&mut bot, &mut channels, STROBE_CHANNEL);
        pulse(&mut bot, &mut channels, WRITE_CHANNEL);
        assert!(!bot.displays().is_empty());
        assert!(bot.design().displays().is_empty());
    }

    #[test]
    fn an_etched_glyph_panel_shows_through_its_host() {
        let mut inner = BotfishState::new();
        inner.install(Part::GlyphPanel);
        inner.wire(Part::GlyphPanel, "char", KEYBOARD);
        inner.wire(Part::GlyphPanel, "write", WRITE_CHANNEL);
        let inputs: BTreeSet<String> = (0..KEYBOARD_WIDTH)
            .map(|bit| format!("{KEYBOARD}{bit}"))
            .chain([WRITE_CHANNEL.to_string()])
            .collect();
        let mut bot = BotfishState::new();
        bot.etch(Chip::new(
            "Screen".to_string(),
            vec![inner],
            inputs,
            BTreeSet::new(),
        ));
        let mut channels = ChannelRegistry::new();
        Bus::new(KEYBOARD, KEYBOARD_WIDTH)
            .expect("a legal bus")
            .drive_into(u32::from(b'A'), &mut channels);
        channels.commit();

        pulse(&mut bot, &mut channels, WRITE_CHANNEL);

        assert_eq!(panel(&bot)[0], "A");
        assert!(bot.design().displays().is_empty());
    }

    const PAD_CHANNEL: &str = "go";

    fn pad() -> BotfishState {
        let mut bot = BotfishState::new();
        assert!(bot.install(Part::ReflexArc));
        bot.wire(Part::ReflexArc, "fire", PAD_CHANNEL);
        bot
    }

    #[test]
    fn a_held_key_drives_its_pins_wire_every_stage_until_it_is_let_go() {
        let mut bot = pad();
        let mut channels = ChannelRegistry::new();
        stage(&mut bot, &mut channels);
        assert!(!channels.level(PAD_CHANNEL));

        bot.key("space", true);
        for _ in 0..3 {
            stage(&mut bot, &mut channels);
            assert!(channels.level(PAD_CHANNEL), "held, so high");
        }
        bot.key("space", false);
        stage(&mut bot, &mut channels);
        assert!(!channels.level(PAD_CHANNEL), "let go, so low");
    }

    #[test]
    fn a_fish_lists_its_key_map_once_and_a_fish_without_an_arc_has_none() {
        assert!(BotfishState::new().bindings().is_empty());
        let bound: Vec<(String, &str)> = pad()
            .bindings()
            .into_iter()
            .map(|binding| (binding.key, binding.pin))
            .collect();
        assert_eq!(bound.len(), Part::ReflexArc.pins().len());
        assert!(bound.contains(&("space".to_string(), "fire")));
    }

    #[test]
    fn an_etched_reflex_arc_hears_the_keyboard_through_its_host() {
        let mut inner = pad();
        inner.wire(Part::ReflexArc, "fire", PAD_CHANNEL);
        let mut bot = BotfishState::new();
        bot.etch(Chip::new(
            "Pad".to_string(),
            vec![inner],
            BTreeSet::new(),
            BTreeSet::from([PAD_CHANNEL.to_string()]),
        ));
        assert!(!bot.bindings().is_empty(), "the chip's map is the host's");
        let mut channels = ChannelRegistry::new();

        bot.key("space", true);
        stage(&mut bot, &mut channels);

        assert!(channels.level(PAD_CHANNEL));
        bot.key("space", false);
        stage(&mut bot, &mut channels);
        assert!(!channels.level(PAD_CHANNEL));
    }

    #[test]
    fn a_design_holds_no_key() {
        let mut bot = pad();
        bot.key("space", true);
        let mut channels = ChannelRegistry::new();
        let mut design = bot.design();
        stage(&mut design, &mut channels);
        assert!(!channels.level(PAD_CHANNEL), "a blueprint has no keyboard");
    }

    #[test]
    fn a_trigger_inside_a_chip_is_heard_through_the_host() {
        let inner = programmed(&["/feed 1"]);
        let mut bot = BotfishState::new();
        bot.etch(Chip::new(
            "Ear".to_string(),
            vec![inner],
            BTreeSet::new(),
            BTreeSet::new(),
        ));

        bot.hear("ping");

        assert!(bot.is_processing());
    }

    const ADDRESS: &str = "a";
    const DATA: &str = "d";
    const WORD: &str = "q";
    const WRITE_ENABLE: &str = "we";
    const WORD_WIDTH: u8 = 8;

    fn core() -> BotfishState {
        let mut bot = BotfishState::new();
        assert!(bot.install(Part::CoreStack));
        bot.wire(Part::CoreStack, "addr", ADDRESS);
        bot.wire(Part::CoreStack, "data_in", DATA);
        bot.wire(Part::CoreStack, "data_out", WORD);
        bot.wire(Part::CoreStack, "write", WRITE_ENABLE);
        bot
    }

    fn set_bus(channels: &mut ChannelRegistry, channel: &str, value: u32) {
        for bit in 0..WORD_WIDTH {
            channels.set_level(&format!("{channel}{bit}"), value >> bit & 1 == 1);
        }
    }

    fn word_on_bus(channels: &ChannelRegistry) -> u32 {
        (0..WORD_WIDTH)
            .filter(|bit| channels.level(&format!("{WORD}{bit}")))
            .map(|bit| 1u32 << bit)
            .sum()
    }

    fn write_word(bot: &mut BotfishState, channels: &mut ChannelRegistry, addr: u32, word: u32) {
        set_bus(channels, ADDRESS, addr);
        set_bus(channels, DATA, word);
        pulse(bot, channels, WRITE_ENABLE);
    }

    fn read_word(bot: &mut BotfishState, channels: &mut ChannelRegistry, addr: u32) -> u32 {
        set_bus(channels, ADDRESS, addr);
        stage(bot, channels);
        word_on_bus(channels)
    }

    #[test]
    fn a_fish_with_a_core_stack_reads_back_the_word_it_wrote_at_that_address() {
        let mut bot = core();
        let mut channels = ChannelRegistry::new();
        assert_eq!(
            read_word(&mut bot, &mut channels, 64),
            0,
            "nothing written, zero read"
        );

        write_word(&mut bot, &mut channels, 64, u32::from(b'@'));

        assert_eq!(read_word(&mut bot, &mut channels, 64), u32::from(b'@'));
        assert_eq!(
            read_word(&mut bot, &mut channels, 63),
            0,
            "another address is untouched"
        );
        assert_eq!(
            read_word(&mut bot, &mut channels, 64),
            u32::from(b'@'),
            "and it holds"
        );
    }

    #[test]
    fn a_write_shows_on_data_out_the_stage_it_lands() {
        let mut bot = core();
        let mut channels = ChannelRegistry::new();
        set_bus(&mut channels, ADDRESS, 5);
        set_bus(&mut channels, DATA, 77);
        stage(&mut bot, &mut channels);
        assert_eq!(word_on_bus(&channels), 0);

        channels.set_level(WRITE_ENABLE, true);
        stage(&mut bot, &mut channels);

        assert_eq!(
            word_on_bus(&channels),
            77,
            "the stack stores at the edge, then reports: what a stage writes, that stage reads"
        );
    }

    #[test]
    fn data_out_follows_the_address_every_stage_without_a_strobe() {
        let mut bot = core();
        let mut channels = ChannelRegistry::new();
        write_word(&mut bot, &mut channels, 1, 11);
        write_word(&mut bot, &mut channels, 2, 22);
        let read: Vec<u32> = [2, 1, 0, 2]
            .into_iter()
            .map(|addr| read_word(&mut bot, &mut channels, addr))
            .collect();
        assert_eq!(
            read,
            vec![22, 11, 0, 22],
            "reading is asynchronous, like SRAM"
        );
    }

    #[test]
    fn a_fish_whose_write_never_rises_stores_nothing_whatever_its_buses_carry() {
        let mut bot = core();
        let mut channels = ChannelRegistry::new();
        for addr in 0..8 {
            set_bus(&mut channels, DATA, 100 + addr);
            set_bus(&mut channels, ADDRESS, addr);
            stage(&mut bot, &mut channels);
        }
        assert!((0..8).all(|addr| read_word(&mut bot, &mut channels, addr) == 0));
    }

    #[test]
    fn an_address_that_arrives_with_the_strobe_is_the_address_written() {
        let mut bot = core();
        let mut channels = ChannelRegistry::new();
        set_bus(&mut channels, ADDRESS, 8);
        set_bus(&mut channels, DATA, 3);
        stage(&mut bot, &mut channels);

        set_bus(&mut channels, ADDRESS, 9);
        channels.set_level(WRITE_ENABLE, true);
        stage(&mut bot, &mut channels);

        assert_eq!(read_word(&mut bot, &mut channels, 9), 3);
        assert_eq!(read_word(&mut bot, &mut channels, 8), 0);
    }

    #[test]
    fn a_word_too_big_for_data_in_is_stored_saturated_like_every_bus() {
        let mut bot = core();
        let mut channels = ChannelRegistry::new();
        set_bus(&mut channels, ADDRESS, 1);
        Bus::new(DATA, WORD_WIDTH)
            .expect("a legal bus")
            .drive_into(4000, &mut channels);
        channels.commit();
        pulse(&mut bot, &mut channels, WRITE_ENABLE);
        assert_eq!(read_word(&mut bot, &mut channels, 1), 255);
    }

    #[test]
    fn a_design_of_a_fish_with_a_core_stack_remembers_nothing() {
        let mut bot = core();
        let mut channels = ChannelRegistry::new();
        write_word(&mut bot, &mut channels, 7, 70);
        let mut design = bot.design();
        let mut fresh = ChannelRegistry::new();
        assert_eq!(read_word(&mut design, &mut fresh, 7), 0);
        assert_eq!(
            design.pins().channel(Part::CoreStack, "data_out"),
            Some(WORD)
        );
    }

    fn bus_lines(channel: &str) -> impl Iterator<Item = String> + '_ {
        (0..WORD_WIDTH).map(move |bit| format!("{channel}{bit}"))
    }

    #[test]
    fn an_etched_core_stack_keeps_its_words_through_its_host() {
        let inputs: BTreeSet<String> = bus_lines(ADDRESS)
            .chain(bus_lines(DATA))
            .chain([WRITE_ENABLE.to_string()])
            .collect();
        let mut bot = BotfishState::new();
        bot.etch(Chip::new(
            "Ram".to_string(),
            vec![core()],
            inputs,
            bus_lines(WORD).collect(),
        ));
        let mut channels = ChannelRegistry::new();

        write_word(&mut bot, &mut channels, 200, 42);

        assert_eq!(read_word(&mut bot, &mut channels, 200), 42);
        assert_eq!(read_word(&mut bot, &mut channels, 201), 0);
        let mut design = bot.design();
        let mut fresh = ChannelRegistry::new();
        assert_eq!(
            read_word(&mut design, &mut fresh, 200),
            0,
            "a blueprint's chip is blank"
        );
    }

    #[test]
    fn a_fused_host_keeps_the_words_its_donor_held() {
        let mut donor = core();
        let mut channels = ChannelRegistry::new();
        write_word(&mut donor, &mut channels, 12, 34);
        let mut fused = receiver().fuse(&donor);
        assert_eq!(
            fused.pins().channel(Part::CoreStack, "data_out"),
            Some(WORD)
        );
        assert_eq!(read_word(&mut fused, &mut channels, 12), 34);
    }

    const DOT: &str = "dot";
    const PIXEL: &str = "px";
    const PLOT: &str = "plot";
    const EIGHT: &str = "eight";
    const BLIT: &str = "blit";
    const SCREEN_WIDTH: &str = "8";

    fn screen() -> BotfishState {
        let mut bot = BotfishState::new();
        assert!(bot.install(Part::CathodeArray));
        let width = Part::CathodeArray
            .config()
            .iter()
            .find(|spec| spec.name == "width")
            .expect("the cathode has a width");
        bot.configure(Part::CathodeArray, width, SCREEN_WIDTH);
        bot.wire(Part::CathodeArray, "addr", DOT);
        bot.wire(Part::CathodeArray, "bit", PIXEL);
        bot.wire(Part::CathodeArray, "write", PLOT);
        bot.wire(Part::CathodeArray, "byte", EIGHT);
        bot.wire(Part::CathodeArray, "write_byte", BLIT);
        bot
    }

    fn surface(bot: &BotfishState) -> Vec<Vec<String>> {
        bot.displays()
            .into_iter()
            .filter_map(|display| match display {
                Display::Body(rows) => Some(rows),
                Display::Bubble(_) => None,
            })
            .collect()
    }

    fn blit_row(bot: &mut BotfishState, channels: &mut ChannelRegistry, addr: u32, byte: u32) {
        set_bus(channels, DOT, addr);
        set_bus(channels, EIGHT, byte);
        pulse(bot, channels, BLIT);
    }

    #[test]
    fn the_parallel_write_fills_a_whole_row_in_one_stage() {
        let mut bot = screen();
        let mut channels = ChannelRegistry::new();
        set_bus(&mut channels, DOT, 3);
        set_bus(&mut channels, EIGHT, 0xFF);
        stage(&mut bot, &mut channels);
        assert!(surface(&bot).is_empty(), "the bus alone writes nothing");

        channels.set_level(BLIT, true);
        stage(&mut bot, &mut channels);

        assert_eq!(
            surface(&bot),
            vec![vec!["⣀⣀⣀⣀".to_string()]],
            "byte 3 of an 8-wide surface is its bottom row, all eight dots in one stage"
        );
    }

    #[test]
    fn a_bit_is_written_at_the_dot_its_address_names_on_the_rise_of_write() {
        let mut bot = screen();
        let mut channels = ChannelRegistry::new();
        set_bus(&mut channels, DOT, 9);
        channels.set_level(PIXEL, true);
        stage(&mut bot, &mut channels);
        assert!(surface(&bot).is_empty(), "no strobe, no dot");

        pulse(&mut bot, &mut channels, PLOT);

        assert_eq!(surface(&bot), vec![vec!["⠐⠀⠀⠀".to_string()]]);
    }

    #[test]
    fn a_picture_holds_with_no_signal_held() {
        let mut bot = screen();
        let mut channels = ChannelRegistry::new();
        blit_row(&mut bot, &mut channels, 0, 0b1010_1010);
        let drawn = surface(&bot);
        set_bus(&mut channels, DOT, 0);
        set_bus(&mut channels, EIGHT, 0);
        for _ in 0..4 {
            stage(&mut bot, &mut channels);
        }
        assert_eq!(surface(&bot), drawn, "a screen latches like the panel does");
        assert_eq!(drawn, vec![vec!["⠈⠈⠈⠈".to_string()]]);
    }

    #[test]
    fn a_design_of_a_fish_with_a_cathode_shows_nothing() {
        let mut bot = screen();
        let mut channels = ChannelRegistry::new();
        blit_row(&mut bot, &mut channels, 1, 0xFF);
        assert!(!surface(&bot).is_empty());
        assert!(surface(&bot.design()).is_empty());
    }

    fn screen_lines() -> BTreeSet<String> {
        bus_lines(DOT)
            .chain(bus_lines(EIGHT))
            .chain([PIXEL.to_string(), PLOT.to_string(), BLIT.to_string()])
            .collect()
    }

    #[test]
    fn an_etched_cathode_draws_through_its_host_and_two_stack_in_board_order() {
        let mut top = screen();
        top.wire(Part::CathodeArray, "write_byte", "blit top");
        let mut bot = BotfishState::new();
        let mut inputs = screen_lines();
        inputs.insert("blit top".to_string());
        bot.etch(Chip::new(
            "Raster".to_string(),
            vec![top, screen()],
            inputs,
            BTreeSet::new(),
        ));
        let mut channels = ChannelRegistry::new();

        set_bus(&mut channels, DOT, 0);
        set_bus(&mut channels, EIGHT, 0xFF);
        pulse(&mut bot, &mut channels, "blit top");
        set_bus(&mut channels, DOT, 3);
        pulse(&mut bot, &mut channels, BLIT);

        assert_eq!(
            surface(&bot),
            vec![vec!["⠉⠉⠉⠉".to_string()], vec!["⣀⣀⣀⣀".to_string()]],
            "one surface per inner cathode, in the order the board holds them"
        );
        assert!(
            surface(&bot.design()).is_empty(),
            "a blueprint's chip is blank"
        );
    }

    #[test]
    fn a_fused_host_keeps_the_picture_its_donor_drew() {
        let mut donor = screen();
        let mut channels = ChannelRegistry::new();
        blit_row(&mut donor, &mut channels, 2, 0x0F);
        let fused = receiver().fuse(&donor);
        assert_eq!(
            fused.pins().channel(Part::CathodeArray, "byte"),
            Some(EIGHT)
        );
        assert_eq!(surface(&fused), surface(&donor));
        assert!(!surface(&fused).is_empty());
    }

    fn relay(far_tank: &str, far_channel: &str, near: &str) -> BotfishState {
        let mut bot = BotfishState::new();
        bot.install(Part::RelayMast);
        let fields = Part::RelayMast.config();
        bot.configure(Part::RelayMast, &fields[0], far_tank);
        bot.configure(Part::RelayMast, &fields[1], far_channel);
        bot.wire(Part::RelayMast, "in", near);
        bot
    }

    fn zion_y() -> Link {
        Link::new("Zion", "y").expect("a link")
    }

    #[test]
    fn a_mast_hands_its_level_back_to_the_fabric_every_stage_it_is_wired() {
        let mut bot = relay("Zion", "y", "x");
        let mut channels = ChannelRegistry::new();
        assert_eq!(bot.links(), vec![zion_y()]);

        let sent = bot.step(&mut channels, &WorldView::default());
        assert_eq!(
            sent,
            vec![Transmission {
                link: zion_y(),
                level: false
            }]
        );
        assert!(
            channels.names().next().is_none(),
            "the near tank carries nothing"
        );

        bot.unwire(Part::RelayMast, "in");
        assert!(
            bot.step(&mut channels, &WorldView::default()).is_empty(),
            "an unwired in drives nothing, here or there"
        );
    }

    #[test]
    fn an_etched_mast_transmits_through_its_host_in_the_chips_one_stage() {
        let mut buffer = BotfishState::new();
        buffer.listen("x");
        buffer.drive("tx");
        let mut bot = BotfishState::new();
        bot.etch(Chip::new(
            "Uplink".to_string(),
            vec![relay("Zion", "y", "tx"), buffer],
            lines(&["x"]),
            BTreeSet::new(),
        ));
        let mut channels = ChannelRegistry::new();
        assert_eq!(
            bot.links(),
            vec![zion_y()],
            "the host is tuned to what its chip is"
        );

        channels.set_level("x", true);
        let sent = bot.step(&mut channels, &WorldView::default());

        assert_eq!(
            sent,
            vec![Transmission {
                link: zion_y(),
                level: true
            }],
            "the inner buffer settles before the mast reads it"
        );
    }

    #[test]
    fn a_design_and_a_fused_host_keep_the_far_end_a_mast_was_aimed_at() {
        let donor = relay("Zion", "y", "x");
        assert_eq!(donor.design().links(), vec![zion_y()]);
        let fused = receiver().fuse(&donor);
        assert_eq!(fused.links(), vec![zion_y()]);
        assert_eq!(fused.pins().channel(Part::RelayMast, "in"), Some("x"));
    }

    const BITE_SECS: f32 = 2.0;
    const TICK_SECS: f32 = 0.5;

    fn angler() -> BotfishState {
        let mut bot = BotfishState::new();
        bot.install(Part::AnglerRig);
        bot.wire(Part::AnglerRig, "cast", "cast");
        bot.wire(Part::AnglerRig, "caught", "caught");
        bot
    }

    fn due(bot: &mut BotfishState, secs: f32) -> Vec<(Vec<usize>, Tackle)> {
        bot.due_casts(secs)
            .into_iter()
            .map(|cast| (cast.path, cast.tackle))
            .collect()
    }

    #[test]
    fn a_cast_asks_for_bait_and_nothing_more_until_the_line_is_down() {
        let mut bot = angler();
        let mut channels = ChannelRegistry::new();
        assert!(due(&mut bot, TICK_SECS).is_empty(), "no cast, no errand");

        pulse(&mut bot, &mut channels, "cast");

        assert_eq!(due(&mut bot, TICK_SECS), vec![(Vec::new(), Tackle::Bait)]);
        assert_eq!(
            due(&mut bot, TICK_SECS),
            vec![(Vec::new(), Tackle::Bait)],
            "the rig keeps asking until the App answers"
        );
    }

    #[test]
    fn a_baited_line_comes_up_after_its_wait_and_caught_pulses_for_exactly_one_stage() {
        let mut bot = angler();
        let mut channels = ChannelRegistry::new();
        pulse(&mut bot, &mut channels, "cast");
        bot.lower_line(&[], BITE_SECS);

        assert!(due(&mut bot, TICK_SECS).is_empty());
        assert!(due(&mut bot, TICK_SECS).is_empty());
        assert!(due(&mut bot, TICK_SECS).is_empty());
        assert_eq!(due(&mut bot, TICK_SECS), vec![(Vec::new(), Tackle::Landed)]);

        bot.reel_in(&[], true);
        assert!(due(&mut bot, TICK_SECS).is_empty(), "the line is up");
        stage(&mut bot, &mut channels);
        assert!(channels.level("caught"), "something came up");
        stage(&mut bot, &mut channels);
        assert!(
            !channels.level("caught"),
            "for one stage, and the engine says so"
        );
    }

    #[test]
    fn a_cast_while_the_line_is_down_is_dropped() {
        let mut bot = angler();
        let mut channels = ChannelRegistry::new();
        pulse(&mut bot, &mut channels, "cast");
        bot.lower_line(&[], BITE_SECS);
        pulse(&mut bot, &mut channels, "cast");

        assert!(
            due(&mut bot, TICK_SECS).is_empty(),
            "still waiting on the first bite"
        );
        assert_eq!(due(&mut bot, BITE_SECS), vec![(Vec::new(), Tackle::Landed)]);
    }

    #[test]
    fn a_cast_with_no_bait_comes_up_empty_and_raises_nothing() {
        let mut bot = angler();
        let mut channels = ChannelRegistry::new();
        pulse(&mut bot, &mut channels, "cast");

        bot.reel_in(&[], false);

        assert!(due(&mut bot, BITE_SECS).is_empty());
        stage(&mut bot, &mut channels);
        assert!(!channels.level("caught"));
    }

    #[test]
    fn an_etched_rig_fishes_through_its_host() {
        let mut bot = BotfishState::new();
        bot.etch(Chip::new(
            "Trawler".to_string(),
            vec![BotfishState::new(), angler()],
            lines(&["cast"]),
            lines(&["caught"]),
        ));
        let mut channels = ChannelRegistry::new();
        pulse(&mut bot, &mut channels, "cast");

        assert_eq!(due(&mut bot, TICK_SECS), vec![(vec![1], Tackle::Bait)]);
        bot.lower_line(&[1], TICK_SECS);
        assert_eq!(due(&mut bot, TICK_SECS), vec![(vec![1], Tackle::Landed)]);
        bot.reel_in(&[1], true);
        stage(&mut bot, &mut channels);
        assert!(
            channels.level("caught"),
            "the host's caught pin carries the pulse"
        );
    }

    #[test]
    fn a_design_forgets_a_line_in_the_water() {
        let mut bot = angler();
        let mut channels = ChannelRegistry::new();
        pulse(&mut bot, &mut channels, "cast");
        assert!(due(&mut bot.design(), TICK_SECS).is_empty());
    }

    const STRIKES: u64 = 200;

    fn board_channels(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn circuit_under_fire() -> BotfishState {
        let mut bot = BotfishState::new();
        bot.listen("a");
        bot.drive("q");
        bot.install(Part::InverterCoil);
        bot.install(Part::DelaySpool);
        bot.install(Part::DelaySpool);
        bot.program("/run Neo".to_string(), vec!["/feed 1".to_string()]);
        bot
    }

    fn wiring(bot: &BotfishState) -> Vec<String> {
        bot.listens()
            .map(|channel| format!("listens {channel}"))
            .chain(
                bot.pins()
                    .iter()
                    .map(|(owner, pin, channel)| format!("{owner:?} {pin} {channel}")),
            )
            .collect()
    }

    #[test]
    fn a_strike_retunes_one_wire_to_another_net_on_the_board_or_knocks_one_part_loose() {
        let channels = board_channels(&["a", "q", "z"]);
        let mut seen_retune = false;
        let mut seen_loose = false;
        for seed in 0..STRIKES {
            let mut bot = circuit_under_fire();
            let before = wiring(&bot);
            let parts_before: Vec<(Part, u32)> = bot.parts().iter().collect();
            let strike = bot
                .irradiate(&channels, &mut SmallRng::seed_from_u64(seed))
                .expect("a wired, fitted fish always has something to hit");
            let after = wiring(&bot);
            let parts_after: Vec<(Part, u32)> = bot.parts().iter().collect();
            assert_eq!(bot.trigger, "/run Neo", "a strike leaves the program alone");
            assert_eq!(bot.script, vec!["/feed 1".to_string()]);
            match strike {
                Strike::Retuned => {
                    seen_retune = true;
                    assert_eq!(parts_after, parts_before);
                    let changed: Vec<&String> =
                        after.iter().filter(|wire| !before.contains(wire)).collect();
                    assert_eq!(
                        changed.len(),
                        1,
                        "exactly one wire moved: {before:?} → {after:?}"
                    );
                    let net = changed[0].rsplit(' ').next().expect("a channel");
                    assert!(
                        channels.iter().any(|channel| channel == net),
                        "{net} is on the board"
                    );
                }
                Strike::KnockedLoose(part) => {
                    seen_loose = true;
                    assert_eq!(after, before, "the wiring stays where it was");
                    let lost: u32 = parts_before.iter().map(|(_, n)| n).sum::<u32>()
                        - parts_after.iter().map(|(_, n)| n).sum::<u32>();
                    assert_eq!(lost, 1, "one part, one unit of a stack");
                    assert!(parts_before.iter().any(|(p, _)| *p == part));
                }
            }
        }
        assert!(seen_retune && seen_loose, "both kinds of damage happen");
    }

    #[test]
    fn a_fish_with_no_hardware_and_no_other_net_to_cross_is_left_alone() {
        let mut bot = BotfishState::new();
        bot.listen("a");
        bot.program("wake".to_string(), vec!["/feed 1".to_string()]);
        let only_its_own = board_channels(&["a"]);
        assert_eq!(
            bot.irradiate(&only_its_own, &mut SmallRng::seed_from_u64(1)),
            None
        );
        assert!(bot.hears("a"));
    }

    #[test]
    fn a_chips_insides_are_sealed_and_only_its_host_pins_can_be_retuned() {
        let mut bot = BotfishState::new();
        bot.etch(not_chip());
        let sealed: Vec<Vec<String>> = bot
            .chip()
            .expect("etched")
            .board()
            .iter()
            .map(wiring)
            .collect();
        let channels = board_channels(&["x", "nx", "z"]);
        for seed in 0..STRIKES {
            let mut struck = bot.clone();
            assert_eq!(
                struck.irradiate(&channels, &mut SmallRng::seed_from_u64(seed)),
                Some(Strike::Retuned),
                "a chip has no loose parts to lose"
            );
            let inside: Vec<Vec<String>> = struck
                .chip()
                .expect("still etched")
                .board()
                .iter()
                .map(wiring)
                .collect();
            assert_eq!(inside, sealed);
        }
    }
}
