use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::cheats::Cheat;
use crate::colors::GREEN_BRIGHT;
use crate::ui::{
    hint_bar::HintBar,
    hints::{HINT_ENTER_CHEAT, HINT_ESC_CLOSE, HINT_INVALID_CHEAT},
    layout::Screen,
    modal::{Modal, ModalSpec},
    table::visual_width,
    text_input::{TextInput, draw_text_cursor},
};

const TITLE: &str = " Enter cheat code ";
const BACKGROUND: Color = Color::Reset;
const BORDER: Color = GREEN_BRIGHT;
const INPUT_ROWS: u16 = 1;
const TEXT_PAD: u16 = 1;
const PROMPT_AND_CURSOR_W: u16 = 3;

pub struct CheatPopup<'a> {
    pub input: &'a TextInput,
    pub cursor_visible: bool,
    pub screen: Screen,
}

impl CheatPopup<'_> {
    fn hints(&self) -> HintBar {
        let typed = self.input.as_str();
        let valid = Cheat::parse(typed).is_some();
        HintBar::new(HINT_ESC_CLOSE)
            .action_if(valid, HINT_ENTER_CHEAT)
            .action_if(!valid && !typed.trim().is_empty(), HINT_INVALID_CHEAT)
    }

    fn widest_hints() -> u16 {
        [HINT_ENTER_CHEAT, HINT_INVALID_CHEAT]
            .into_iter()
            .map(|verdict| HintBar::new(HINT_ESC_CLOSE).action(verdict).natural_width())
            .max()
            .unwrap_or_default()
    }

    fn content_width() -> u16 {
        let longest = Cheat::ALL
            .iter()
            .map(|cheat| visual_width(cheat.code()))
            .max()
            .unwrap_or_default() as u16;
        (longest + PROMPT_AND_CURSOR_W + TEXT_PAD * 2).max(Self::widest_hints())
    }
}

impl Widget for CheatPopup<'_> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        let hints = self.hints();
        let modal = Modal::open(
            buf,
            self.screen,
            &ModalSpec {
                title: TITLE,
                border: BORDER,
                background: BACKGROUND,
                content_w: Self::content_width(),
                content_h: INPUT_ROWS,
                hints: &hints,
            },
        );
        if modal.body.height == 0 {
            return;
        }
        draw_text_cursor(
            buf,
            self.input,
            self.cursor_visible,
            modal.body.x + TEXT_PAD,
            modal.body.y,
            modal.body.width.saturating_sub(TEXT_PAD * 2),
            BACKGROUND,
        );
    }
}
