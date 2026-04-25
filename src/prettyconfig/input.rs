// Input handling for prettyconfig TUI
// Key event processing and user interaction

use crossterm::event::{KeyCode, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::prettyconfig::helpers::{next_theme, prev_theme};
use crate::prettyconfig::navigation::{App, FocusArea};
use crate::prettyconfig::save;

// Check if a point is inside a rectangle
fn point_in_rect(x: u16, y: u16, rect: Rect) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

impl App {
    pub fn handle_key(&mut self, key: KeyCode) {
        if self.editing {
            self.handle_editing_key(key);
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('s') => self.save_config(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Left => self.handle_left(),
            KeyCode::Right => self.handle_right(),
            KeyCode::Char(' ') | KeyCode::Enter => self.handle_select(),
            KeyCode::Tab => self.next_focus(),
            KeyCode::BackTab => self.prev_focus(),
            _ => {}
        }
    }

    fn handle_editing_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.editing = false;
                self.edit_buffer.clear();
            }
            KeyCode::Enter => self.finish_editing(),
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.edit_buffer.remove(self.cursor_pos);
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.edit_buffer.len() {
                    self.edit_buffer.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.edit_buffer.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => self.cursor_pos = 0,
            KeyCode::End => self.cursor_pos = self.edit_buffer.len(),
            KeyCode::Char(c) => {
                self.edit_buffer.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            _ => {}
        }
    }

    fn handle_left(&mut self) {
        match self.focus {
            FocusArea::General => match self.index {
                0 => {
                    self.theme = prev_theme(self.theme);
                    self.update_preview();
                }
                1 => {
                    self.cycle_nerd_fonts_prev();
                    self.update_preview();
                }
                2 => {
                    self.cycle_box_style_prev();
                    self.update_preview();
                }
                3 => {
                    self.cycle_border_line_style_prev();
                    self.update_preview();
                }
                4 => {
                    self.cycle_gpu_display_prev();
                    self.update_preview();
                }
                _ => {}
            },
            FocusArea::Art => {
                let builtin = !self.image && self.custom_art.is_none();
                match self.index {
                    0 => self.cycle_art_mode_prev(),
                    1 if builtin => self.cycle_os_art_prev(),
                    2 => { self.cycle_art_position(); self.update_preview(); }
                    _ => {}
                }
            }
            FocusArea::Userspace => {
                if self.index == 8 {
                    self.cycle_colors_style_prev();
                    self.update_preview();
                }
            }
            _ => {}
        }
    }

    fn handle_right(&mut self) {
        match self.focus {
            FocusArea::General => match self.index {
                0 => {
                    self.theme = next_theme(self.theme);
                    self.update_preview();
                }
                1 => {
                    self.cycle_nerd_fonts_next();
                    self.update_preview();
                }
                2 => {
                    self.cycle_box_style_next();
                    self.update_preview();
                }
                3 => {
                    self.cycle_border_line_style_next();
                    self.update_preview();
                }
                4 => {
                    self.cycle_gpu_display_next();
                    self.update_preview();
                }
                _ => {}
            },
            FocusArea::Art => {
                let builtin = !self.image && self.custom_art.is_none();
                match self.index {
                    0 => self.cycle_art_mode_next(),
                    1 if builtin => self.cycle_os_art_next(),
                    2 => { self.cycle_art_position(); self.update_preview(); }
                    _ => {}
                }
            }
            FocusArea::Userspace => {
                if self.index == 8 {
                    self.cycle_colors_style_next();
                    self.update_preview();
                }
            }
            _ => {}
        }
    }

    fn handle_select(&mut self) {
        match self.focus {
            FocusArea::General => match self.index {
                0 => {
                    self.theme = next_theme(self.theme);
                    self.update_preview();
                }
                1 => {
                    self.cycle_nerd_fonts_next();
                    self.update_preview();
                }
                2 => {
                    self.cycle_box_style_next();
                    self.update_preview();
                }
                3 => {
                    self.cycle_border_line_style_next();
                    self.update_preview();
                }
                4 => {
                    self.cycle_gpu_display_next();
                    self.update_preview();
                }
                _ => {}
            },
            FocusArea::Art => {
                let builtin = !self.image && self.custom_art.is_none();
                match self.index {
                    0 => self.cycle_art_mode_next(),
                    1 if builtin => self.cycle_os_art_next(),
                    1 => {
                        // Source row — open editor with current path
                        let initial = if self.image {
                            self.image_path.clone().unwrap_or_default()
                        } else {
                            self.custom_art.clone().unwrap_or_default()
                        };
                        self.start_editing(initial);
                    }
                    2 => { self.cycle_art_position(); self.update_preview(); }
                    _ => {}
                }
            }
            FocusArea::Core => {
                match self.index {
                    0 => self.core.os = !self.core.os,
                    1 => self.core.kernel = !self.core.kernel,
                    2 => self.core.platform = !self.core.platform,
                    3 => self.core.uptime = !self.core.uptime,
                    4 => self.core.init = !self.core.init,
                    5 => self.core.os_age = !self.core.os_age,
                    _ => {}
                }
                self.update_preview();
            }
            FocusArea::Hardware => {
                match self.index {
                    0 => self.hardware.cpu = !self.hardware.cpu,
                    1 => self.hardware.gpu = !self.hardware.gpu,
                    2 => self.hardware.memory = !self.hardware.memory,
                    3 => self.hardware.storage = !self.hardware.storage,
                    4 => self.hardware.battery = !self.hardware.battery,
                    5 => self.hardware.screen = !self.hardware.screen,
                    _ => {}
                }
                self.update_preview();
            }
            FocusArea::Userspace => {
                match self.index {
                    0 => self.userspace.user = !self.userspace.user,
                    1 => self.userspace.packages = !self.userspace.packages,
                    2 => self.userspace.terminal = !self.userspace.terminal,
                    3 => self.userspace.shell = !self.userspace.shell,
                    4 => self.userspace.wm = !self.userspace.wm,
                    5 => self.userspace.ui = !self.userspace.ui,
                    6 => self.userspace.editor = !self.userspace.editor,
                    7 => self.userspace.terminal_font = !self.userspace.terminal_font,
                    8 => self.userspace.colors = !self.userspace.colors,
                    _ => {}
                }
                self.update_preview();
            }
        }
    }

    pub fn save_config(&mut self) {
        match save::save_config(
            self.theme,
            self.nerd_fonts,
            &self.os_art,
            &self.custom_art,
            self.image,
            &self.image_path,
            self.art_position,
            self.box_style,
            self.border_line_style,
            self.gpu_display,
            &self.core,
            &self.hardware,
            &self.userspace,
        ) {
            Ok(path) => {
                self.status_message = Some(format!("Saved to {:?}", path));
                self.should_exit = true;
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }

    pub fn handle_mouse(&mut self, kind: MouseEventKind, x: u16, y: u16) {
        // Ignore mouse events while editing
        if self.editing {
            return;
        }

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(x, y);
            }
            MouseEventKind::ScrollUp => {
                self.move_up();
            }
            MouseEventKind::ScrollDown => {
                self.move_down();
            }
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, x: u16, y: u16) {
        let layout = &self.layout;

        // Check General box
        if point_in_rect(x, y, layout.general_box) {
            self.focus = FocusArea::General;
            // Calculate which item was clicked (y offset from box top, minus border)
            let item_y = y.saturating_sub(layout.general_box.y + 1);
            self.index = (item_y as usize).min(FocusArea::General.max_index());
            self.handle_select();
            return;
        }

        // Check Art box
        if point_in_rect(x, y, layout.art_box) {
            self.focus = FocusArea::Art;
            let item_y = y.saturating_sub(layout.art_box.y + 1);
            self.index = (item_y as usize).min(2);
            self.handle_select();
            return;
        }

        // Check Core box
        if point_in_rect(x, y, layout.core_box) {
            self.focus = FocusArea::Core;
            let item_y = y.saturating_sub(layout.core_box.y + 1);
            self.index = (item_y as usize).min(FocusArea::Core.max_index());
            self.handle_select();
            return;
        }

        // Check Hardware box
        if point_in_rect(x, y, layout.hardware_box) {
            self.focus = FocusArea::Hardware;
            let item_y = y.saturating_sub(layout.hardware_box.y + 1);
            self.index = (item_y as usize).min(FocusArea::Hardware.max_index());
            self.handle_select();
            return;
        }

        // Check Userspace box
        if point_in_rect(x, y, layout.userspace_box) {
            self.focus = FocusArea::Userspace;
            let item_y = y.saturating_sub(layout.userspace_box.y + 1);
            self.index = (item_y as usize).min(FocusArea::Userspace.max_index());
            self.handle_select();
            return;
        }

        // Check Save button
        if point_in_rect(x, y, layout.save_button) {
            self.save_config();
            return;
        }

        // Check Quit button
        if point_in_rect(x, y, layout.cancel_button) {
            self.should_exit = true;
        }
    }
}