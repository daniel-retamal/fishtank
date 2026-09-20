use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::colors::{BLACK, DARK_GRAY, STEEL, WHITE};
use crate::tank::{Blueprint, BlueprintPins, FabricationQuote, FabricationRefusal};
use crate::ui::circuit_overlay::{
    CircuitFish, Schematic, fits_schematic, sorted_by_depth, stacked_label, stacked_name,
    too_big_to_draw,
};
use crate::ui::hint_bar::HintBar;
use crate::ui::hints::{
    HINT_CLOSE, HINT_E_ETCH, HINT_ENTER_SAVE, HINT_ESC_CANCEL, HINT_P_PRINT, HINT_PAN, HINT_SCROLL,
    HINT_SELECT, HINT_TAB_LIST, HINT_TAB_PREVIEW, HINT_X_EXPORTS,
};
use crate::ui::layout::{PanBar, Screen, Scroll, Scrollbar};
use crate::ui::panels::{PanelSpec, Panels, Reach};
use crate::ui::table::{NOTHING, ellipsize, visual_width, wrap_words};
use crate::ui::text_input::{TextInput, draw_text_cursor};

const TITLE: &str = " Foundry ";
const LIST_HEADER: &str = "Blueprints";
const LIST_HEADER_ROWS: u16 = 1;
const LIST_NAME_MAX_W: usize = 24;
const FISH_GAP: usize = 2;
const TEXT_PAD: u16 = 1;
const LABEL_W: usize = 9;
const VALUE_NATURAL_MAX_W: usize = 36;
const MIN_BODY_W: u16 = 20;
const PREVIEW_GAP_ROWS: usize = 1;
const PAN_STEP: isize = 4;
const BACKGROUND: Color = Color::Reset;
const CHANNEL_JOIN: &str = ", ";
const PIN_FLOW: &str = " → ";
const PINS_LABEL: &str = "pins";
const EXPORTS_LABEL: &str = "exports";
const PARTS_LABEL: &str = "parts";
const PRINT_LABEL: &str = "print";
const ETCH_LABEL: &str = "etch";

pub type FabricationStatus = Result<FabricationQuote, FabricationRefusal>;

pub struct Quotes {
    pub print: FabricationStatus,
    pub etch: FabricationStatus,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Focus {
    List,
    Preview,
}

pub struct FoundryState {
    pub selected: usize,
    count: usize,
    focus: Focus,
    editing: Option<TextInput>,
    list: Scroll,
    preview: Scroll,
    pan: Scroll,
}

impl FoundryState {
    pub fn new(blueprints: &[Blueprint]) -> Option<Self> {
        if blueprints.is_empty() {
            return None;
        }
        Some(Self {
            selected: 0,
            count: blueprints.len(),
            focus: Focus::List,
            editing: None,
            list: Scroll::default(),
            preview: Scroll::default(),
            pan: Scroll::default(),
        })
    }

    pub fn scroll_up(&mut self) {
        if self.focus == Focus::Preview {
            self.preview.nudge(-1);
            return;
        }
        self.select(self.selected.saturating_sub(1));
    }

    pub fn scroll_down(&mut self) {
        if self.focus == Focus::Preview {
            self.preview.nudge(1);
            return;
        }
        self.select((self.selected + 1).min(self.count - 1));
    }

    fn select(&mut self, index: usize) {
        if index == self.selected {
            return;
        }
        self.selected = index;
        self.preview.reset();
        self.pan.reset();
    }

    pub fn pan(&mut self, right: bool) {
        if self.focus != Focus::Preview {
            return;
        }
        self.pan.nudge(if right { PAN_STEP } else { -PAN_STEP });
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::List => Focus::Preview,
            Focus::Preview => Focus::List,
        };
    }

    pub fn start_editing(&mut self, blueprint: &Blueprint) {
        self.editing = Some(TextInput::with_value(blueprint.exports_text()));
        self.preview.reset();
    }

    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    pub fn editor_mut(&mut self) -> Option<&mut TextInput> {
        self.editing.as_mut()
    }

    pub fn take_edit(&mut self) -> Option<String> {
        self.editing.take().map(|input| input.value)
    }

    pub fn cancel_edit(&mut self) {
        self.editing = None;
    }

    fn widest_hints(&self) -> u16 {
        let editing = HintBar::new(HINT_ESC_CANCEL).action(HINT_ENTER_SAVE);
        [Focus::List, Focus::Preview]
            .into_iter()
            .map(|focus| self.browsing_hints(focus, true).natural_width())
            .chain(std::iter::once(editing.natural_width()))
            .max()
            .unwrap_or(0)
    }

    fn hints(&self, list_hidden: bool) -> HintBar {
        if self.is_editing() {
            return HintBar::new(HINT_ESC_CANCEL).action(HINT_ENTER_SAVE);
        }
        self.browsing_hints(self.focus, list_hidden)
    }

    fn browsing_hints(&self, focus: Focus, list_hidden: bool) -> HintBar {
        match focus {
            Focus::List => HintBar::new(HINT_CLOSE)
                .counted(
                    HINT_SELECT,
                    list_hidden.then_some((self.selected + 1, self.count)),
                )
                .action(HINT_P_PRINT)
                .action(HINT_E_ETCH)
                .action(HINT_X_EXPORTS)
                .action(HINT_TAB_PREVIEW),
            Focus::Preview => HintBar::new(HINT_CLOSE)
                .action(HINT_SCROLL)
                .action(HINT_PAN)
                .action(HINT_P_PRINT)
                .action(HINT_E_ETCH)
                .action(HINT_X_EXPORTS)
                .action(HINT_TAB_LIST),
        }
    }
}

fn joined_or_nothing<'a>(items: impl Iterator<Item = &'a String>) -> String {
    let joined = items
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(CHANNEL_JOIN);
    if joined.is_empty() {
        return NOTHING.to_string();
    }
    joined
}

fn pins_text(pins: &BlueprintPins) -> String {
    if pins.inputs.is_empty() && pins.outputs.is_empty() {
        return NOTHING.to_string();
    }
    format!(
        "{}{}{}",
        joined_or_nothing(pins.inputs.iter()),
        PIN_FLOW,
        joined_or_nothing(pins.outputs.iter())
    )
}

fn parts_text(blueprint: &Blueprint) -> String {
    let totals = blueprint.part_totals();
    if totals.is_empty() {
        return NOTHING.to_string();
    }
    totals
        .into_iter()
        .map(|(part, count)| stacked_name(part, count))
        .collect::<Vec<_>>()
        .join(CHANNEL_JOIN)
}

fn fish_label(blueprint: &Blueprint) -> String {
    format!("{} fish", blueprint.fish.len())
}

fn quote_text(quote: &FabricationQuote) -> String {
    let materials = quote
        .materials
        .iter()
        .map(|material| stacked_label(material.kind.display_name(), material.needed))
        .collect::<Vec<_>>()
        .join(CHANNEL_JOIN);
    if quote.cost == 0 {
        return materials;
    }
    format!("{materials} (buys ${})", quote.cost)
}

fn status_row(status: Option<&FabricationStatus>) -> (String, Tone) {
    match status {
        Some(Ok(quote)) => (quote_text(quote), Tone::Plain),
        Some(Err(refusal)) => (refusal.reason(), Tone::Disabled),
        None => (NOTHING.to_string(), Tone::Disabled),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tone {
    Plain,
    Disabled,
    Field,
}

struct DetailRow {
    label: &'static str,
    text: String,
    tone: Tone,
}

enum Graph {
    Drawn(Schematic),
    TooBig(Vec<String>),
}

impl Graph {
    fn height(&self) -> usize {
        match self {
            Graph::Drawn(schematic) => schematic.height(),
            Graph::TooBig(lines) => lines.len(),
        }
    }
}

struct Preview {
    rows: Vec<DetailRow>,
    graph: Graph,
    focus_rows: Range<usize>,
}

impl Preview {
    fn of(blueprint: &Blueprint, quotes: Option<&Quotes>, width: u16, editing: bool) -> Self {
        let value_w = value_width(width);
        let mut rows = Vec::new();
        push_wrapped(
            &mut rows,
            PINS_LABEL,
            &pins_text(&blueprint.pins()),
            value_w,
            Tone::Plain,
        );
        let exports_start = rows.len();
        if editing {
            rows.push(DetailRow {
                label: EXPORTS_LABEL,
                text: String::new(),
                tone: Tone::Field,
            });
        } else {
            let exports = joined_or_nothing(blueprint.exports.iter());
            push_wrapped(&mut rows, EXPORTS_LABEL, &exports, value_w, Tone::Plain);
        }
        let exports_end = rows.len();
        push_wrapped(
            &mut rows,
            PARTS_LABEL,
            &parts_text(blueprint),
            value_w,
            Tone::Plain,
        );
        let print_start = rows.len();
        let (print_text, print_tone) = status_row(quotes.map(|q| &q.print));
        push_wrapped(&mut rows, PRINT_LABEL, &print_text, value_w, print_tone);
        let print_end = rows.len();
        let (etch_text, etch_tone) = status_row(quotes.map(|q| &q.etch));
        push_wrapped(&mut rows, ETCH_LABEL, &etch_text, value_w, etch_tone);
        let focus_rows = if editing {
            exports_start..exports_end
        } else {
            print_start..print_end
        };
        let fish = board_of(blueprint);
        let graph = if fits_schematic(&fish) {
            Graph::Drawn(Schematic::of(&fish))
        } else {
            let room = width.saturating_sub(TEXT_PAD * 2) as usize;
            Graph::TooBig(wrap_words(&too_big_to_draw(&fish), room))
        };
        Self {
            rows,
            graph,
            focus_rows,
        }
    }

    fn height(&self) -> usize {
        self.rows.len() + PREVIEW_GAP_ROWS + self.graph.height()
    }

    fn graph_start(&self) -> usize {
        self.rows.len() + PREVIEW_GAP_ROWS
    }

    fn natural_width(blueprint: &Blueprint, quotes: Option<&Quotes>) -> u16 {
        let widest_value = [
            pins_text(&blueprint.pins()),
            joined_or_nothing(blueprint.exports.iter()),
            parts_text(blueprint),
            status_row(quotes.map(|q| &q.print)).0,
            status_row(quotes.map(|q| &q.etch)).0,
        ]
        .iter()
        .map(|text| visual_width(text))
        .max()
        .unwrap_or(0)
        .min(VALUE_NATURAL_MAX_W);
        let schematic_w = Schematic::of(&board_of(blueprint)).width();
        (LABEL_W + widest_value).max(schematic_w) as u16 + TEXT_PAD * 2
    }
}

fn board_of(blueprint: &Blueprint) -> Vec<CircuitFish> {
    sorted_by_depth(
        blueprint
            .fish
            .iter()
            .map(|fish| CircuitFish::from_design(&fish.name, &fish.design))
            .collect(),
    )
}

fn value_width(width: u16) -> usize {
    (width.saturating_sub(TEXT_PAD * 2) as usize)
        .saturating_sub(LABEL_W)
        .max(1)
}

fn push_wrapped(
    rows: &mut Vec<DetailRow>,
    label: &'static str,
    text: &str,
    width: usize,
    tone: Tone,
) {
    let lines = wrap_words(text, width);
    if lines.is_empty() {
        rows.push(DetailRow {
            label,
            text: String::new(),
            tone,
        });
        return;
    }
    for (index, line) in lines.into_iter().enumerate() {
        rows.push(DetailRow {
            label: if index == 0 { label } else { "" },
            text: line,
            tone,
        });
    }
}

pub struct FoundryOverlay<'a> {
    state: &'a FoundryState,
    blueprints: &'a [Blueprint],
    quotes: Option<Quotes>,
    cursor_visible: bool,
    screen: Screen,
}

impl<'a> FoundryOverlay<'a> {
    pub fn new(
        state: &'a FoundryState,
        blueprints: &'a [Blueprint],
        quotes: Option<Quotes>,
        cursor_visible: bool,
        screen: Screen,
    ) -> Self {
        Self {
            state,
            blueprints,
            quotes,
            cursor_visible,
            screen,
        }
    }

    fn list_width(&self) -> u16 {
        let widest = self
            .blueprints
            .iter()
            .map(|blueprint| {
                visual_width(&blueprint.name).min(LIST_NAME_MAX_W)
                    + FISH_GAP
                    + visual_width(&fish_label(blueprint))
            })
            .max()
            .unwrap_or(0)
            .max(visual_width(LIST_HEADER));
        widest as u16 + TEXT_PAD * 2
    }

    fn list_height(&self) -> u16 {
        LIST_HEADER_ROWS + self.blueprints.len() as u16
    }

    fn draw_list(&self, buf: &mut Buffer, side: Rect) {
        if side.height == 0 || side.width == 0 {
            return;
        }
        let room = side.width.saturating_sub(TEXT_PAD * 2) as usize;
        let bold = Style::default()
            .fg(WHITE)
            .add_modifier(Modifier::BOLD)
            .bg(BACKGROUND);
        let header_rows = if side.height as usize > self.blueprints.len() {
            buf.set_stringn(side.x + TEXT_PAD, side.y, LIST_HEADER, room, bold);
            LIST_HEADER_ROWS
        } else {
            0
        };
        let rows_room = (side.height - header_rows) as usize;
        let heights = vec![1; self.blueprints.len()];
        let shown = self
            .state
            .list
            .follow(&heights, self.state.selected, rows_room);
        for (row, index) in shown.enumerate().take(rows_room) {
            let y = side.y + header_rows + row as u16;
            self.draw_list_row(buf, side, y, index);
        }
    }

    fn draw_list_row(&self, buf: &mut Buffer, side: Rect, y: u16, index: usize) {
        let blueprint = &self.blueprints[index];
        let selected = index == self.state.selected;
        let row_bg = match (selected, self.state.focus) {
            (false, _) => BACKGROUND,
            (true, Focus::List) => WHITE,
            (true, Focus::Preview) => STEEL,
        };
        let fg = if selected { BLACK } else { STEEL };
        let style = Style::default().fg(fg).bg(row_bg);
        for x in side.x..side.right() {
            buf[(x, y)].set_char(' ').set_bg(row_bg);
        }
        let room = side.width.saturating_sub(TEXT_PAD * 2) as usize;
        let label = fish_label(blueprint);
        let label_w = visual_width(&label);
        let name_room = room.saturating_sub(label_w + FISH_GAP).min(LIST_NAME_MAX_W);
        let name = ellipsize(&blueprint.name, name_room.max(1));
        buf.set_stringn(side.x + TEXT_PAD, y, &name, room, style);
        let label_x = side.right().saturating_sub(TEXT_PAD + label_w as u16);
        let name_end = side.x + TEXT_PAD + visual_width(&name) as u16;
        if label_x > name_end {
            buf.set_stringn(label_x, y, &label, label_w, style);
        }
    }

    fn draw_preview(&self, buf: &mut Buffer, panels: &Panels, blueprint: &Blueprint) {
        let body = panels.body;
        if body.height == 0 || body.width == 0 {
            return;
        }
        let state = self.state;
        let preview = Preview::of(
            blueprint,
            self.quotes.as_ref(),
            body.width,
            state.is_editing(),
        );
        let total = preview.height();
        let room = body.height as usize;
        let shown = if state.focus == Focus::Preview && !state.is_editing() {
            state.preview.window(total, room)
        } else {
            state
                .preview
                .reveal(preview.focus_rows.clone(), room, total)
        };
        for (row, line) in shown.clone().enumerate() {
            let y = body.y + row as u16;
            if let Some(detail) = preview.rows.get(line) {
                self.draw_detail(buf, body, y, detail);
            }
        }
        let graph_rows = overlap(&shown, preview.graph_start()..total);
        if !graph_rows.is_empty() {
            let first_y = body.y + (graph_rows.start - shown.start) as u16;
            let graph = (graph_rows.start - preview.graph_start())
                ..(graph_rows.end - preview.graph_start());
            self.draw_graph(buf, panels, &preview.graph, first_y, graph);
        }
        Scrollbar {
            x: panels.right_x(),
            top: body.y,
            height: body.height,
        }
        .draw(buf, shown, total, WHITE);
    }

    fn draw_detail(&self, buf: &mut Buffer, body: Rect, y: u16, detail: &DetailRow) {
        let (label_fg, text_fg) = match detail.tone {
            Tone::Disabled => (DARK_GRAY, DARK_GRAY),
            Tone::Plain | Tone::Field => (STEEL, WHITE),
        };
        let x = body.x + TEXT_PAD;
        let room = body.width.saturating_sub(TEXT_PAD * 2) as usize;
        buf.set_stringn(
            x,
            y,
            detail.label,
            room,
            Style::default().fg(label_fg).bg(BACKGROUND),
        );
        let value_x = x + LABEL_W.min(room) as u16;
        let value_room = room.saturating_sub(LABEL_W) as u16;
        if value_room == 0 {
            return;
        }
        if detail.tone == Tone::Field
            && let Some(input) = &self.state.editing
        {
            draw_text_cursor(
                buf,
                input,
                self.cursor_visible,
                value_x,
                y,
                value_room,
                BACKGROUND,
            );
            return;
        }
        buf.set_stringn(
            value_x,
            y,
            &detail.text,
            value_room as usize,
            Style::default().fg(text_fg).bg(BACKGROUND),
        );
    }

    fn draw_graph(
        &self,
        buf: &mut Buffer,
        panels: &Panels,
        graph: &Graph,
        first_y: u16,
        rows: Range<usize>,
    ) {
        let body = panels.body;
        let room = body.width.saturating_sub(TEXT_PAD * 2);
        match graph {
            Graph::TooBig(lines) => {
                for (offset, line) in lines[rows].iter().enumerate() {
                    buf.set_stringn(
                        body.x + TEXT_PAD,
                        first_y + offset as u16,
                        line,
                        room as usize,
                        Style::default().fg(STEEL).bg(BACKGROUND),
                    );
                }
            }
            Graph::Drawn(schematic) => {
                let width = schematic.width();
                let margin = (room as usize).saturating_sub(width) / 2;
                let view = Rect::new(
                    body.x + TEXT_PAD + margin as u16,
                    first_y,
                    room.min(width as u16),
                    rows.len() as u16,
                );
                let columns = self.state.pan.window(width, view.width as usize);
                schematic.draw(buf, view, (rows, columns.clone()), None);
                PanBar {
                    y: panels.rect.bottom().saturating_sub(1),
                    left: view.x,
                    width: view.width,
                }
                .draw(buf, columns, width, WHITE);
            }
        }
    }
}

fn overlap(a: &Range<usize>, b: Range<usize>) -> Range<usize> {
    let start = a.start.max(b.start);
    let end = a.end.min(b.end);
    start..end.max(start)
}

impl Widget for FoundryOverlay<'_> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        let Some(blueprint) = self.blueprints.get(
            self.state
                .selected
                .min(self.blueprints.len().saturating_sub(1)),
        ) else {
            return;
        };
        let editing = self.state.is_editing();
        let quotes = self.quotes.as_ref();
        let rows_for = |width: u16| Preview::of(blueprint, quotes, width, editing).height() as u16;
        let side = (self.list_width(), self.list_height());
        let body_w = Preview::natural_width(blueprint, quotes)
            .max(self.state.widest_hints())
            .max(MIN_BODY_W);
        let spec = |hints| PanelSpec {
            title: TITLE,
            title_style: Style::default()
                .fg(WHITE)
                .add_modifier(Modifier::BOLD)
                .bg(BACKGROUND),
            border: Style::default().fg(WHITE).bg(BACKGROUND),
            background: BACKGROUND,
            side,
            body_w,
            body_min_w: MIN_BODY_W,
            body_rows: &rows_for,
            hints,
            reach: Reach::Full,
        };
        let calm = self.state.hints(false);
        let list_hidden = (Panels::measure(self.screen, &spec(&calm)).side.height as usize)
            < self.blueprints.len();
        let hints = self.state.hints(list_hidden);
        let panels = Panels::open(buf, self.screen, &spec(&hints));
        self.draw_list(buf, panels.side);
        self.draw_preview(buf, &panels, blueprint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::components::Position;
    use crate::fishes::botfish::BotfishState;
    use crate::fishes::fish::Direction;
    use crate::fishes::parts::Part;
    use crate::loot::ConsumableKind;
    use crate::tank::{BlueprintFish, Material};
    use std::collections::BTreeSet;

    fn gate(name: &str, listens: &[&str], drives: &str) -> BlueprintFish {
        let mut design = BotfishState::new();
        for channel in listens {
            design.listen(channel);
        }
        design.drive(drives);
        design.install(Part::InverterCoil);
        BlueprintFish {
            name: name.to_string(),
            design,
            offset: Position { x: 0.0, y: 0.0 },
            facing: Direction::Right,
            frozen: true,
        }
    }

    fn latch() -> Blueprint {
        Blueprint {
            name: "Latch".to_string(),
            fish: vec![gate("Q", &["r", "qn"], "q"), gate("Qn", &["s", "q"], "qn")],
            exports: BTreeSet::new(),
        }
    }

    fn rendered(blueprints: &[Blueprint], state: &FoundryState, cols: u16, rows: u16) -> String {
        rendered_with(
            blueprints,
            state,
            Some(Quotes {
                print: Err(FabricationRefusal::NeedsFabricator),
                etch: Err(FabricationRefusal::NeedsFabricator),
            }),
            (cols, rows),
        )
    }

    fn rendered_with(
        blueprints: &[Blueprint],
        state: &FoundryState,
        quotes: Option<Quotes>,
        (cols, rows): (u16, u16),
    ) -> String {
        let area = Rect::new(0, 0, cols, rows);
        let mut buffer = Buffer::empty(area);
        FoundryOverlay::new(state, blueprints, quotes, true, Screen::only(area))
            .render(area, &mut buffer);
        (0..rows)
            .map(|y| {
                (0..cols)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_foundry_with_nothing_to_show_does_not_open() {
        assert!(FoundryState::new(&[]).is_none());
    }

    #[test]
    fn a_latch_reads_its_inputs_and_nothing_out_until_q_is_exported() {
        let mut blueprint = latch();
        assert_eq!(pins_text(&blueprint.pins()), "r, s → -");
        blueprint.export("q");
        assert_eq!(pins_text(&blueprint.pins()), "r, s → q");
    }

    #[test]
    fn the_parts_row_totals_every_fish_on_the_board() {
        assert_eq!(parts_text(&latch()), "Inverter Coil ×2");
    }

    #[test]
    fn selection_stops_at_both_ends_and_resets_the_preview() {
        let blueprints = [latch(), latch()];
        let mut state = FoundryState::new(&blueprints).expect("two blueprints");
        state.scroll_up();
        assert_eq!(state.selected, 0);
        state.preview.nudge(3);
        state.scroll_down();
        state.scroll_down();
        assert_eq!(state.selected, 1);
        assert_eq!(
            state.preview.offset(),
            0,
            "a new blueprint is read from the top"
        );
    }

    #[test]
    fn arrows_scroll_the_preview_once_it_has_the_focus() {
        let blueprints = [latch(), latch()];
        let mut state = FoundryState::new(&blueprints).expect("two blueprints");
        state.toggle_focus();
        state.scroll_down();
        assert_eq!(state.selected, 0, "the list keeps its selection");
        assert_eq!(state.preview.offset(), 1);
    }

    #[test]
    fn editing_exports_opens_on_what_the_blueprint_already_exports() {
        let mut blueprint = latch();
        blueprint.export("qn");
        let blueprints = [blueprint];
        let mut state = FoundryState::new(&blueprints).expect("one blueprint");
        state.start_editing(&blueprints[0]);
        assert_eq!(state.take_edit().as_deref(), Some("qn"));
        assert!(!state.is_editing());
    }

    #[test]
    fn print_and_etch_are_shown_disabled_with_their_reason() {
        let blueprints = [latch()];
        let state = FoundryState::new(&blueprints).expect("one blueprint");
        let text = rendered(&blueprints, &state, 100, 30);
        assert!(text.contains("print    needs a Fabricator"), "{text}");
        assert!(text.contains("etch     needs a Fabricator"), "{text}");
        assert!(
            text.contains("[Q]"),
            "the schematic preview draws the board:\n{text}"
        );
    }

    fn quote_of(wafers: u32, held_wafers: u32, cost: u32) -> FabricationQuote {
        FabricationQuote {
            materials: vec![
                Material {
                    kind: ConsumableKind::BlankWafer,
                    needed: wafers,
                    in_stock: held_wafers,
                },
                Material {
                    kind: ConsumableKind::Part(Part::InverterCoil),
                    needed: 2,
                    in_stock: 2,
                },
            ],
            cost,
        }
    }

    #[test]
    fn a_ready_print_and_etch_list_what_they_spend_and_what_they_buy() {
        let blueprints = [latch()];
        let state = FoundryState::new(&blueprints).expect("one blueprint");
        let quotes = Quotes {
            print: Ok(quote_of(2, 1, 50)),
            etch: Ok(quote_of(4, 1, 150)),
        };
        let text = rendered_with(&blueprints, &state, Some(quotes), (100, 30));
        assert!(
            text.contains("print    Blank Wafer ×2, Inverter Coil ×2 (buys $50)"),
            "{text}"
        );
        assert!(
            text.contains("etch     Blank Wafer ×4, Inverter Coil ×2 (buys $150)"),
            "{text}"
        );
    }

    #[test]
    fn a_refused_print_says_why() {
        let blueprints = [latch()];
        let state = FoundryState::new(&blueprints).expect("one blueprint");
        let refused = Some(Quotes {
            print: Err(FabricationRefusal::TankFull { needed: 2 }),
            etch: Err(FabricationRefusal::NoHost),
        });
        let text = rendered_with(&blueprints, &state, refused, (100, 30));
        assert!(
            text.contains("print    tank full: needs room for 2 fish"),
            "{text}"
        );
        assert!(
            text.contains("etch     needs a botfish to etch into"),
            "{text}"
        );
    }

    const QUARTER_SCREEN: (u16, u16) = (28, 10);
    const WHOLE_SCREEN: (u16, u16) = (100, 30);
    const FRAME: char = '\u{2502}';

    fn shows_field(text: &str, label: &str) -> bool {
        text.lines().any(|line| {
            line.split(FRAME)
                .any(|cell| cell.trim_start().starts_with(label))
        })
    }

    fn quarter(blueprints: &[Blueprint], state: &FoundryState) -> String {
        rendered(blueprints, state, QUARTER_SCREEN.0, QUARTER_SCREEN.1)
    }

    #[test]
    fn a_quarter_screen_shows_why_p_does_nothing_before_it_shows_anything_else() {
        let blueprints = [latch()];
        let state = FoundryState::new(&blueprints).expect("one blueprint");
        let text = quarter(&blueprints, &state);
        assert!(
            shows_field(&text, PRINT_LABEL),
            "the refused action fell below the fold:\n{text}"
        );
        assert!(
            !shows_field(&text, PINS_LABEL),
            "pins outranked the reason on a quarter screen:\n{text}"
        );
        let wide = rendered(&blueprints, &state, WHOLE_SCREEN.0, WHOLE_SCREEN.1);
        assert!(
            shows_field(&wide, PINS_LABEL) && shows_field(&wide, PRINT_LABEL),
            "a screen with room must still start at the top:\n{wide}"
        );
    }

    #[test]
    fn the_exports_field_is_pulled_into_view_from_wherever_the_preview_sat() {
        let blueprints = [latch()];
        let mut state = FoundryState::new(&blueprints).expect("one blueprint");
        state.toggle_focus();
        for _ in 0..8 {
            state.scroll_down();
        }
        let scrolled = quarter(&blueprints, &state);
        assert!(
            !shows_field(&scrolled, EXPORTS_LABEL),
            "the preview was supposed to be scrolled away from the field:\n{scrolled}"
        );
        state.start_editing(&blueprints[0]);
        let editing = quarter(&blueprints, &state);
        assert!(
            shows_field(&editing, EXPORTS_LABEL),
            "a field you cannot see is a field you cannot type into:\n{editing}"
        );
    }

    #[test]
    fn taking_the_preview_lets_the_player_scroll_past_the_reason() {
        let blueprints = [latch()];
        let mut state = FoundryState::new(&blueprints).expect("one blueprint");
        assert!(shows_field(&quarter(&blueprints, &state), PRINT_LABEL));
        state.toggle_focus();
        for _ in 0..8 {
            state.scroll_down();
        }
        let scrolled = quarter(&blueprints, &state);
        assert!(
            !shows_field(&scrolled, PRINT_LABEL),
            "TAB must hand the whole panel over, reason included:\n{scrolled}"
        );
    }

    #[test]
    fn a_quarter_screen_still_draws_every_hint() {
        let blueprints = [latch()];
        let state = FoundryState::new(&blueprints).expect("one blueprint");
        let text = rendered(&blueprints, &state, 28, 10);
        for hint in [
            HINT_P_PRINT,
            HINT_E_ETCH,
            HINT_X_EXPORTS,
            HINT_TAB_PREVIEW,
            HINT_CLOSE,
        ] {
            assert!(text.contains(hint), "{hint} is missing:\n{text}");
        }
    }
}
