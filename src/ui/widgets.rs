//! UI widgets for the notcurses interface

use super::context::{channels, NotcursesContext};
use crate::error::Result;

/// A selectable menu item
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub description: Option<String>,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            enabled: true,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// A vertical menu widget
pub struct Menu {
    items: Vec<MenuItem>,
    selected: usize,
    y: u32,
    x: u32,
    width: u32,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>, y: u32, x: u32, width: u32) -> Self {
        Self {
            items,
            selected: 0,
            y,
            x,
            width,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn select_next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
            // Skip disabled items
            while self.selected < self.items.len() && !self.items[self.selected].enabled {
                self.selected += 1;
            }
            if self.selected >= self.items.len() {
                self.selected = self.items.len() - 1;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Skip disabled items
            while self.selected > 0 && !self.items[self.selected].enabled {
                self.selected -= 1;
            }
        }
    }

    pub fn render(&self, ctx: &mut NotcursesContext) -> Result<()> {
        for (i, item) in self.items.iter().enumerate() {
            let y = self.y + i as u32;
            let is_selected = i == self.selected;

            // Determine colors
            let channels = if !item.enabled {
                channels::from_rgb(100, 100, 100, 0, 0, 0) // Gray for disabled
            } else if is_selected {
                channels::from_rgb(0, 0, 0, 0, 200, 255) // Black on cyan for selected
            } else {
                channels::WHITE_ON_BLACK
            };

            // Draw selection marker
            let marker = if is_selected { "▶ " } else { "  " };
            let text = format!("{}{}", marker, item.label);

            ctx.putstr_yx(y, self.x, &format!("{:<width$}", text, width = self.width as usize), channels)?;

            // Draw description if available
            if let Some(desc) = &item.description {
                let desc_y = y;
                let desc_x = self.x + self.width + 4;
                let desc_channels = if item.enabled {
                    channels::from_rgb(150, 150, 150, 0, 0, 0)
                } else {
                    channels::from_rgb(80, 80, 80, 0, 0, 0)
                };
                ctx.putstr_yx(desc_y, desc_x, desc, desc_channels)?;
            }
        }
        Ok(())
    }
}

/// A checkbox list widget
pub struct CheckList {
    items: Vec<String>,
    checked: Vec<bool>,
    selected: usize,
    y: u32,
    x: u32,
    height: u32,
    scroll_offset: usize,
}

impl CheckList {
    pub fn new(items: Vec<String>, y: u32, x: u32, height: u32) -> Self {
        let checked = vec![false; items.len()];
        Self {
            items,
            checked,
            selected: 0,
            y,
            x,
            height,
            scroll_offset: 0,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn is_checked(&self, index: usize) -> bool {
        self.checked.get(index).copied().unwrap_or(false)
    }

    pub fn toggle_selected(&mut self) {
        if self.selected < self.checked.len() {
            self.checked[self.selected] = !self.checked[self.selected];
        }
    }

    pub fn checked_indices(&self) -> Vec<usize> {
        self.checked
            .iter()
            .enumerate()
            .filter(|(_, &checked)| checked)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn select_next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
            // Adjust scroll if needed
            if self.selected >= self.scroll_offset + self.height as usize {
                self.scroll_offset = self.selected - self.height as usize + 1;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Adjust scroll if needed
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn render(&self, ctx: &mut NotcursesContext) -> Result<()> {
        let visible_items = self.height as usize;
        let end = (self.scroll_offset + visible_items).min(self.items.len());

        for (i, item_idx) in (self.scroll_offset..end).enumerate() {
            let y = self.y + i as u32;
            let is_selected = item_idx == self.selected;
            let is_checked = self.checked[item_idx];

            let channels = if is_selected {
                channels::from_rgb(0, 0, 0, 100, 100, 255) // Black on light blue
            } else {
                channels::WHITE_ON_BLACK
            };

            let checkbox = if is_checked { "[✓]" } else { "[ ]" };
            let marker = if is_selected { "▶" } else { " " };
            let text = format!("{} {} {}", marker, checkbox, self.items[item_idx]);

            ctx.putstr_yx(y, self.x, &text, channels)?;
        }

        // Draw scrollbar if needed
        if self.items.len() > visible_items {
            self.draw_scrollbar(ctx)?;
        }

        Ok(())
    }

    fn draw_scrollbar(&self, ctx: &mut NotcursesContext) -> Result<()> {
        let scrollbar_x = self.x + 60; // Position on the right
        let scrollbar_height = self.height;
        let total_items = self.items.len();

        // Draw scrollbar track
        for i in 0..scrollbar_height {
            ctx.putstr_yx(
                self.y + i,
                scrollbar_x,
                "│",
                channels::from_rgb(100, 100, 100, 0, 0, 0),
            )?;
        }

        // Calculate thumb position and size
        let thumb_size = (scrollbar_height as f32 * (scrollbar_height as f32 / total_items as f32))
            .max(1.0) as u32;
        let thumb_pos = (self.scroll_offset as f32
            * (scrollbar_height as f32 / total_items as f32)) as u32;

        // Draw thumb
        for i in 0..thumb_size {
            ctx.putstr_yx(
                self.y + thumb_pos + i,
                scrollbar_x,
                "█",
                channels::from_rgb(200, 200, 200, 0, 0, 0),
            )?;
        }

        Ok(())
    }
}

/// A simple button widget
pub struct Button {
    label: String,
    y: u32,
    x: u32,
    width: u32,
    selected: bool,
}

impl Button {
    pub fn new(label: impl Into<String>, y: u32, x: u32, width: u32) -> Self {
        Self {
            label: label.into(),
            y,
            x,
            width,
            selected: false,
        }
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn render(&self, ctx: &mut NotcursesContext) -> Result<()> {
        let channels = if self.selected {
            channels::from_rgb(255, 255, 255, 0, 150, 0) // White on green
        } else {
            channels::from_rgb(200, 200, 200, 50, 50, 50) // Light gray on dark gray
        };

        // Draw button
        let padding = (self.width as usize - self.label.len()) / 2;
        let text = format!("{:padding$}{}{:padding$}", "", self.label, "", padding = padding);

        ctx.putstr_yx(self.y, self.x, "┌", channels)?;
        for i in 1..self.width - 1 {
            ctx.putstr_yx(self.y, self.x + i, "─", channels)?;
        }
        ctx.putstr_yx(self.y, self.x + self.width - 1, "┐", channels)?;

        ctx.putstr_yx(self.y + 1, self.x, "│", channels)?;
        ctx.putstr_yx(self.y + 1, self.x + 1, &text[..self.width as usize - 2], channels)?;
        ctx.putstr_yx(self.y + 1, self.x + self.width - 1, "│", channels)?;

        ctx.putstr_yx(self.y + 2, self.x, "└", channels)?;
        for i in 1..self.width - 1 {
            ctx.putstr_yx(self.y + 2, self.x + i, "─", channels)?;
        }
        ctx.putstr_yx(self.y + 2, self.x + self.width - 1, "┘", channels)?;

        Ok(())
    }
}

/// A dialog box widget
pub struct Dialog {
    title: String,
    message: Vec<String>,
    buttons: Vec<String>,
    selected_button: usize,
    y: u32,
    x: u32,
    width: u32,
    height: u32,
}

impl Dialog {
    pub fn new(title: impl Into<String>, message: Vec<String>, buttons: Vec<String>) -> Self {
        let title_str: String = title.into();
        let width = message
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(40)
            .max(title_str.len() + 4)
            .max(20) as u32
            + 4;

        let height = message.len() as u32 + buttons.len() as u32 + 6;

        Self {
            title: title_str,
            message,
            buttons,
            selected_button: 0,
            y: 0,
            x: 0,
            width,
            height,
        }
    }

    pub fn center(&mut self, screen_rows: u32, screen_cols: u32) {
        self.y = (screen_rows - self.height) / 2;
        self.x = (screen_cols - self.width) / 2;
    }

    pub fn selected_button(&self) -> usize {
        self.selected_button
    }

    pub fn select_next_button(&mut self) {
        if self.selected_button < self.buttons.len() - 1 {
            self.selected_button += 1;
        }
    }

    pub fn select_prev_button(&mut self) {
        if self.selected_button > 0 {
            self.selected_button -= 1;
        }
    }

    pub fn render(&self, ctx: &mut NotcursesContext) -> Result<()> {
        // Draw box
        ctx.draw_box(
            self.y,
            self.x,
            self.height,
            self.width,
            Some(&self.title),
            channels::CYAN_ON_BLACK,
        )?;

        // Draw message lines
        let mut current_y = self.y + 2;
        for line in &self.message {
            let line_x = self.x + (self.width - line.len() as u32) / 2;
            ctx.putstr_yx(current_y, line_x, line, channels::WHITE_ON_BLACK)?;
            current_y += 1;
        }

        // Draw buttons
        current_y += 2;
        let button_width = 12u32;
        let total_button_width = self.buttons.len() as u32 * button_width
            + (self.buttons.len() - 1) as u32 * 2;
        let mut button_x = self.x + (self.width - total_button_width) / 2;

        for (i, button_label) in self.buttons.iter().enumerate() {
            let mut button = Button::new(button_label, current_y, button_x, button_width);
            button.set_selected(i == self.selected_button);
            button.render(ctx)?;
            button_x += button_width + 2;
        }

        Ok(())
    }
}

/// An input field widget
pub struct InputField {
    label: String,
    value: String,
    y: u32,
    x: u32,
    width: u32,
    cursor_pos: usize,
}

impl InputField {
    pub fn new(label: impl Into<String>, value: impl Into<String>, y: u32, x: u32, width: u32) -> Self {
        let value = value.into();
        let cursor_pos = value.len();
        Self {
            label: label.into(),
            value,
            y,
            x,
            width,
            cursor_pos,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.value.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_pos < self.value.len() {
            self.value.remove(self.cursor_pos);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.value.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn render(&self, ctx: &mut NotcursesContext) -> Result<()> {
        // Draw label
        ctx.putstr_yx(self.y, self.x, &self.label, channels::CYAN_ON_BLACK)?;

        // Draw input box
        let input_y = self.y + 1;
        ctx.draw_box(input_y, self.x, 3, self.width, None, channels::WHITE_ON_BLACK)?;

        // Draw value
        let display_value = if self.value.len() as u32 > self.width - 4 {
            &self.value[self.value.len() - (self.width - 4) as usize..]
        } else {
            &self.value
        };

        ctx.putstr_yx(input_y + 1, self.x + 2, display_value, channels::WHITE_ON_BLACK)?;

        // Draw cursor (if applicable)
        let cursor_x = self.x + 2 + self.cursor_pos as u32;
        ctx.putstr_yx(input_y + 1, cursor_x, "_", channels::GREEN_ON_BLACK)?;

        Ok(())
    }
}
