use std::cell::Cell;
use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

const SCROLL_THUMB: char = '┃';
const PAN_THUMB: char = '━';

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Screen {
    pub whole: Rect,
    pub stage: Rect,
}

impl Screen {
    pub fn new(whole: Rect, stage: Rect) -> Self {
        Self { whole, stage }
    }

    pub fn only(area: Rect) -> Self {
        Self::new(area, area)
    }

    pub fn place(&self, width: u16, height: u16) -> Rect {
        let width = width.min(self.whole.width);
        let height = height.min(self.whole.height);
        Rect::new(
            centred_within(
                self.stage.x,
                self.stage.width,
                width,
                self.whole.x,
                self.whole.width,
            ),
            centred_within(
                self.stage.y,
                self.stage.height,
                height,
                self.whole.y,
                self.whole.height,
            ),
            width,
            height,
        )
    }
}

pub fn body_rows(content: u16, room: u16, padding: u16) -> u16 {
    if content <= room {
        return content;
    }
    room.saturating_sub(padding).max(room.min(1))
}

fn centred_within(
    stage_start: u16,
    stage_len: u16,
    len: u16,
    whole_start: u16,
    whole_len: u16,
) -> u16 {
    let preferred = stage_start as i32 + (stage_len as i32 - len as i32) / 2;
    let last = whole_start as i32 + whole_len as i32 - len as i32;
    preferred.clamp(whole_start as i32, last.max(whole_start as i32)) as u16
}

const GIVES_EARLY: u8 = 0;
const GIVES_NORMALLY: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlexItem {
    pub basis: u16,
    pub min: u16,
    pub gives: u8,
}

impl FlexItem {
    pub fn new(basis: u16, min: u16) -> Self {
        Self {
            basis,
            min: min.min(basis),
            gives: GIVES_NORMALLY,
        }
    }

    pub fn fixed(basis: u16) -> Self {
        Self::new(basis, basis)
    }

    pub fn gives_first(self) -> Self {
        self.gives_in_turn(GIVES_EARLY)
    }

    pub fn gives_in_turn(mut self, turn: u8) -> Self {
        self.gives = turn;
        self
    }
}

pub fn shrink(items: &[FlexItem], available: u16) -> Vec<u16> {
    let mut widths: Vec<u16> = items.iter().map(|item| item.basis).collect();
    let mut orders: Vec<u8> = items.iter().map(|item| item.gives).collect();
    orders.sort_unstable();
    orders.dedup();
    for order in orders {
        let group: Vec<usize> = (0..items.len())
            .filter(|&index| items[index].gives == order)
            .collect();
        let others: u32 = (0..items.len())
            .filter(|index| !group.contains(index))
            .map(|index| widths[index] as u32)
            .sum();
        let room = (available as u32)
            .saturating_sub(others)
            .min(u16::MAX as u32) as u16;
        let sizes: Vec<u16> = group.iter().map(|&index| widths[index]).collect();
        let floors: Vec<u16> = group.iter().map(|&index| items[index].min).collect();
        for (&index, width) in group.iter().zip(take_from(&sizes, &floors, room)) {
            widths[index] = width;
        }
    }
    let total: u32 = widths.iter().map(|&w| w as u32).sum();
    if total <= available as u32 {
        return widths;
    }
    take_from(&widths, &vec![0; items.len()], available)
}

fn take_from(sizes: &[u16], floors: &[u16], available: u16) -> Vec<u16> {
    let total: u32 = sizes.iter().map(|&w| w as u32).sum();
    let mut result = sizes.to_vec();
    if total <= available as u32 {
        return result;
    }
    let deficit = total - available as u32;
    let capacity: u32 = sizes
        .iter()
        .zip(floors)
        .map(|(&size, &floor)| size.saturating_sub(floor) as u32)
        .sum();
    if capacity == 0 {
        return result;
    }
    let wanted = deficit.min(capacity);
    let mut taken = 0u32;
    for (index, (&size, &floor)) in sizes.iter().zip(floors).enumerate() {
        let room = size.saturating_sub(floor) as u32;
        let share = wanted * room / capacity;
        result[index] -= share as u16;
        taken += share;
    }
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&index| std::cmp::Reverse(result[index].saturating_sub(floors[index])));
    for index in order.into_iter().cycle().take(sizes.len() * 2) {
        if taken >= wanted {
            break;
        }
        if result[index] > floors[index] {
            result[index] -= 1;
            taken += 1;
        }
    }
    result
}

#[derive(Default, Debug)]
pub struct Scroll {
    offset: Cell<usize>,
}

impl Clone for Scroll {
    fn clone(&self) -> Self {
        Self {
            offset: Cell::new(self.offset.get()),
        }
    }
}

impl Scroll {
    pub fn offset(&self) -> usize {
        self.offset.get()
    }

    pub fn reset(&self) {
        self.offset.set(0);
    }

    pub fn follow(&self, heights: &[usize], selected: usize, room: usize) -> Range<usize> {
        if heights.is_empty() {
            self.offset.set(0);
            return 0..0;
        }
        let selected = selected.min(heights.len() - 1);
        let mut first = self.offset.get().min(selected);
        while first < selected && heights[first..=selected].iter().sum::<usize>() > room {
            first += 1;
        }
        while first > 0 && heights[first - 1..].iter().sum::<usize>() <= room {
            first -= 1;
        }
        self.offset.set(first);
        first..first + shown_from(&heights[first..], room)
    }

    pub fn reveal(&self, span: Range<usize>, room: usize, total: usize) -> Range<usize> {
        let mut first = self.offset.get();
        if span.end > first + room {
            first = span.end - room;
        }
        if span.start < first {
            first = span.start;
        }
        first = first.min(total.saturating_sub(room));
        self.offset.set(first);
        first..(first + room).min(total)
    }

    pub fn window(&self, total: usize, room: usize) -> Range<usize> {
        let first = self.offset.get().min(total.saturating_sub(room));
        self.offset.set(first);
        first..(first + room).min(total)
    }

    pub fn nudge(&self, delta: isize) {
        let next = self.offset.get() as isize + delta;
        self.offset.set(next.max(0) as usize);
    }

    pub fn settle(&self, last: usize) -> usize {
        let offset = self.offset.get().min(last);
        self.offset.set(offset);
        offset
    }

    pub fn scroll_by(&self, delta: isize, total: usize, room: usize) {
        let last = total.saturating_sub(room) as isize;
        let next = (self.offset.get() as isize + delta).clamp(0, last.max(0));
        self.offset.set(next as usize);
    }
}

fn shown_from(heights: &[usize], room: usize) -> usize {
    let mut used = 0;
    let mut count = 0;
    for &height in heights {
        if used + height > room && count > 0 {
            break;
        }
        used += height;
        count += 1;
    }
    count
}

pub struct Scrollbar {
    pub x: u16,
    pub top: u16,
    pub height: u16,
}

impl Scrollbar {
    pub fn draw(&self, buf: &mut Buffer, shown: Range<usize>, total: usize, color: Color) {
        let style = Style::default().fg(color);
        for row in thumb(self.height, shown, total) {
            buf[(self.x, self.top + row as u16)]
                .set_char(SCROLL_THUMB)
                .set_style(style);
        }
    }
}

pub struct PanBar {
    pub y: u16,
    pub left: u16,
    pub width: u16,
}

impl PanBar {
    pub fn draw(&self, buf: &mut Buffer, shown: Range<usize>, total: usize, color: Color) {
        let style = Style::default().fg(color);
        for column in thumb(self.width, shown, total) {
            buf[(self.left + column as u16, self.y)]
                .set_char(PAN_THUMB)
                .set_style(style);
        }
    }
}

fn thumb(track: u16, shown: Range<usize>, total: usize) -> Range<usize> {
    if track == 0 || total <= shown.len() {
        return 0..0;
    }
    let track = track as usize;
    let thumb_len = (track * shown.len() / total).clamp(1, track);
    let travel = track - thumb_len;
    let hidden = total - shown.len();
    let start = (travel * shown.start + hidden / 2) / hidden;
    start..start + thumb_len
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Screen {
        Screen::new(Rect::new(0, 0, 80, 24), Rect::new(0, 0, 80, 20))
    }

    #[test]
    fn a_modal_that_fits_the_tank_is_centred_on_the_tank() {
        assert_eq!(screen().place(20, 10), Rect::new(30, 5, 20, 10));
    }

    #[test]
    fn a_modal_taller_than_the_tank_grows_over_the_bars_instead_of_shrinking() {
        let placed = screen().place(20, 23);
        assert_eq!((placed.y, placed.height), (0, 23));
    }

    #[test]
    fn a_modal_bigger_than_the_whole_screen_takes_the_whole_screen() {
        assert_eq!(screen().place(200, 90), Rect::new(0, 0, 80, 24));
    }

    #[test]
    fn items_that_fit_keep_their_basis() {
        let items = [FlexItem::new(10, 4), FlexItem::new(20, 5)];
        assert_eq!(shrink(&items, 40), vec![10, 20]);
    }

    #[test]
    fn a_deficit_is_taken_in_proportion_to_what_each_item_can_give() {
        let items = [FlexItem::new(10, 5), FlexItem::new(30, 5)];
        let widths = shrink(&items, 30);
        assert_eq!(widths.iter().sum::<u16>(), 30);
        assert!(widths[1] - 5 > widths[0] - 5, "{widths:?}");
    }

    #[test]
    fn an_item_that_gives_first_shrinks_to_its_minimum_before_the_others_are_touched() {
        let items = [
            FlexItem::new(12, 3),
            FlexItem::new(8, 3),
            FlexItem::new(180, 8).gives_first(),
        ];
        let widths = shrink(&items, 60);
        assert_eq!(widths, vec![12, 8, 40]);
        let squeezed = shrink(&items, 20);
        assert_eq!(squeezed[2], 8, "the long column reached its floor");
        assert_eq!(squeezed.iter().sum::<u16>(), 20);
    }

    #[test]
    fn below_every_minimum_the_minimums_shrink_too_and_nothing_disappears_first() {
        let items = [FlexItem::new(10, 6), FlexItem::fixed(6)];
        let widths = shrink(&items, 8);
        assert_eq!(widths.iter().sum::<u16>(), 8);
        assert!(widths.iter().all(|&w| w > 0), "{widths:?}");
    }

    #[test]
    fn following_a_selection_keeps_it_on_screen_with_tall_rows() {
        let scroll = Scroll::default();
        assert_eq!(scroll.follow(&[6, 5, 4], 1, 8), 1..2);
        assert_eq!(scroll.follow(&[6, 5, 4], 0, 8), 0..1);
    }

    #[test]
    fn a_larger_screen_pulls_the_window_back_up_to_fill_it() {
        let scroll = Scroll::default();
        scroll.follow(&[1; 10], 9, 3);
        assert_eq!(scroll.follow(&[1; 10], 9, 10), 0..10);
    }

    #[test]
    fn revealing_a_span_pans_just_far_enough_to_show_it() {
        let pan = Scroll::default();
        assert_eq!(pan.reveal(30..36, 10, 50), 26..36);
        assert_eq!(
            pan.reveal(28..32, 10, 50),
            26..36,
            "already in view, nothing moves"
        );
        assert_eq!(pan.reveal(2..6, 10, 50), 2..12);
    }

    #[test]
    fn a_window_clamps_to_the_end_of_its_content() {
        let scroll = Scroll::default();
        scroll.scroll_by(50, 10, 4);
        assert_eq!(scroll.window(10, 4), 6..10);
    }
}
