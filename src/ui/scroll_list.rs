pub fn scroll_up(selected: &mut usize, scroll: &mut usize) {
    if *selected > 0 {
        *selected -= 1;
        if *selected < *scroll {
            *scroll = *selected;
        }
    }
}

pub fn scroll_down(selected: &mut usize, scroll: &mut usize, len: usize, visible: usize) {
    if *selected + 1 < len {
        *selected += 1;
        if *selected >= *scroll + visible {
            *scroll = *selected + 1 - visible;
        }
    }
}
