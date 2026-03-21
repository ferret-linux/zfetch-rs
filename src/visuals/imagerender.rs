// Image rendering module for zfetch
// Handles layout and display of images using the Kitty graphics protocol

use crate::visuals::renderer::{build_box, build_sections_lines, visible_len, Section};
use crate::visuals::terminalsize::get_terminal_size;

// Draw a side-by-side or vertically stacked layout with an image placeholder.
// The image is rendered using Kitty graphics protocol after the box layout is printed.
// Cursor positioning is used to overlay the image inside the empty box.
pub fn draw_image_layout(sections: &[Section], image_path: Option<&std::path::Path>, art_right: bool) {
    // --- step 1: Get terminal dimensions ---
    let (terminal_width, terminal_height) = get_terminal_size()
        .map(|(cols, rows)| (cols as usize, rows as usize))
        .unwrap_or((80, 24)); // Fallback to standard 80x24 terminal

    // --- step 2: Calculate sections dimensions ---
    // Each line is "Key: Value", so width = key_len + 2 (": ") + value_len
    let sections_content_width = sections
        .iter()
        .flat_map(|section| {
            std::iter::once(section.title.chars().count()).chain(
                section
                    .lines
                    .iter()
                    .map(|(key, value)| visible_len(key) + 2 + visible_len(value)),
            )
        })
        .max()
        .unwrap_or(0);

    // Box width = content + 4 (2 for borders, 2 for internal margins)
    let sections_box_width = sections_content_width + 4;

    // Sections height = sum of (content lines + 2 borders) for each section
    let sections_total_height: usize = sections
        .iter()
        .map(|section| section.lines.len() + 2)
        .sum();

    // --- step 3: Calculate image box dimensions ---
    // Image box should be roughly square based on sections height
    // Terminal cells are typically ~2:1 height:width ratio, so multiply height by 2
    let image_content_width = (sections_total_height as f64 * 2.0) as usize;
    let image_box_width = image_content_width + 4; // Add borders + margins

    // Total width needed for side-by-side layout: image_box + gap + sections_box
    let side_by_side_total_width = image_box_width + 1 + sections_box_width;

    // --- step 4: Choose layout based on terminal width ---
    if terminal_width >= side_by_side_total_width {
        // layout 1: Side-by-side
        render_side_by_side_with_image(
            sections,
            image_path,
            image_content_width,
            art_right,
        );
    } else {
        // layout 2: Stacked (image on top, sections below) or sections only
        render_stacked_with_image(
            sections,
            image_path,
            sections_content_width,
            sections_total_height,
            terminal_height,
        );
    }
}

// ender side-by-side layout: empty image box on left, sections on right.
// After printing the layout, cursor is repositioned to overlay the image.
fn render_side_by_side_with_image(
    sections: &[Section],
    image_path: Option<&std::path::Path>,
    image_content_width: usize,
    art_right: bool,
) {
    use std::io::Write;

    // --- step 1: Build the sections box ---
    let sections_box = build_sections_lines(sections, None);
    let sections_box_height = sections_box.len();

    // --- step 2: Build empty image box (placeholder for image) ---
    // Height matches sections box for visual alignment
    let empty_content: Vec<String> = Vec::new();
    let image_box = build_box(
        &empty_content,
        None,
        Some(image_content_width),
        Some(sections_box_height),
        true, // Center content (though empty)
    );

    // --- step 3: Combine boxes into output string ---
    let total_row_count = image_box.len().max(sections_box.len());
    let image_box_visual_width = visible_len(&image_box[0]);
    let image_padding_spaces = " ".repeat(image_box_visual_width);

    let sections_box_visual_width = sections_box.first().map(|l| visible_len(l)).unwrap_or(0);
    let sections_padding = " ".repeat(sections_box_visual_width);

    let mut output = String::new();
    for row_index in 0..total_row_count {
        if art_right {
            // Left: sections, Right: image
            if row_index < sections_box.len() {
                output.push_str(&sections_box[row_index]);
            } else {
                output.push_str(&sections_padding);
            }
            output.push(' ');
            if row_index < image_box.len() {
                output.push_str(&image_box[row_index]);
            }
        } else {
            // Left: image, Right: sections
            if row_index < image_box.len() {
                output.push_str(&image_box[row_index]);
            } else {
                output.push_str(&image_padding_spaces);
            }
            output.push(' ');
            if row_index < sections_box.len() {
                output.push_str(&sections_box[row_index]);
            }
        }
        output.push('\n');
    }

    // --- step 4: Print layout and position cursor for image ---
    let total_output_lines = output.lines().count();
    let image_display_cols = image_content_width;
    let image_display_rows = sections_box_height.saturating_sub(2); // Subtract borders

    // Print the box layout first
    print!("{}", output);
    let _ = std::io::stdout().flush();

    // Save cursor position at end of output, then move into image box
    print!("\x1b7"); // Save cursor (DECSC)
    print!("\x1b[{}A", total_output_lines - 1); // Move up to first content row
    print!("\x1b[2C"); // Move right past left border
    let _ = std::io::stdout().flush();

    // --- step 5: Display the image using Kitty protocol ---
    let result = match image_path {
        Some(path) => crate::modules::image::display_image(path, image_display_cols as u16, image_display_rows as u16),
        None => crate::modules::image::display_default_image(image_display_cols as u16, image_display_rows as u16),
    };
    match result {
        Ok(image_output) => {
            print!("{}", image_output);
            let _ = std::io::stdout().flush();
        }
        Err(image_error) => eprintln!("Image error: {}", image_error),
    }

    // --- step 6: Restore cursor to end of layout ---
    print!("\x1b8"); // Restore cursor (DECRC)
    let _ = std::io::stdout().flush();
}

// Render stacked layout: image box on top, sections below.
// Falls back to sections-only if terminal is too small.
fn render_stacked_with_image(
    sections: &[Section],
    image_path: Option<&std::path::Path>,
    sections_content_width: usize,
    sections_total_height: usize,
    terminal_height: usize,
) {
    use std::io::Write;

    // --- step 1: Calculate image box dimensions for stacked layout ---
    // Image box width matches sections width for visual consistency
    let image_content_width = sections_content_width;

    // Calculate image box height to maintain ~1:1 aspect ratio
    // Terminal cells are ~2:1 height:width, so divide total visual width by 2
    // Visual width = content + 6 (2 borders + 2 margins + 2 for padding)
    let image_box_total_height = ((sections_content_width + 6) as f64 / 2.0).ceil() as usize;
    let image_content_height = image_box_total_height.saturating_sub(2); // Subtract borders

    // --- step 2: Check if we have enough vertical space ---
    let stacked_total_height = image_box_total_height + sections_total_height;

    // Minimum content width of 8 ensures image is visible
    if terminal_height >= stacked_total_height && image_content_width > 8 {
        // --- step 3: Build image box (empty placeholder) ---
        let empty_content: Vec<String> = Vec::new();
        let image_box = build_box(
            &empty_content,
            None,
            Some(image_content_width),
            Some(image_box_total_height),
            true,
        );

        // --- step 4: Build sections box with matching width ---
        let sections_box = build_sections_lines(sections, Some(image_content_width));

        // --- step 5: Combine into output string (stacked vertically) ---
        let mut output = String::new();

        // Image box on top
        for line in &image_box {
            output.push_str(line);
            output.push('\n');
        }

        // Sections box below
        for line in &sections_box {
            output.push_str(line);
            output.push('\n');
        }

        // --- step 6: Print layout and position cursor for image ---
        let total_output_lines = output.lines().count();

        print!("{}", output);
        let _ = std::io::stdout().flush();

        // Save cursor position at end of output, then move into image box
        print!("\x1b7"); // Save cursor (DECSC)
        print!("\x1b[{}A", total_output_lines - 1); // Move up to first content row
        print!("\x1b[2C"); // Move right past left border
        let _ = std::io::stdout().flush();

        // --- step 7: Display the image ---
        let result = match image_path {
            Some(path) => crate::modules::image::display_image(path, image_content_width as u16, image_content_height as u16),
            None => crate::modules::image::display_default_image(image_content_width as u16, image_content_height as u16),
        };
        match result {
            Ok(image_output) => {
                print!("{}", image_output);
                let _ = std::io::stdout().flush();
            }
            Err(image_error) => eprintln!("Image error: {}", image_error),
        }

        // --- step 8: Restore cursor to end of layout ---
        print!("\x1b8"); // Restore cursor (DECRC)
        let _ = std::io::stdout().flush();
    } else {
        // --- fallback: Terminal too small, show sections only ---
        let sections_box = build_sections_lines(sections, None);

        for line in &sections_box {
            println!("{}", line);
        }
    }
}
