// METGen - The Synthesized METAR Generator
// Copyright (C) 2025 FiendishDrWu
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io::{self as io, Write};
use crossterm::{
    execute,
    style::{Color, SetForegroundColor, SetBackgroundColor, SetAttribute, Attribute},
    event::{read, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor,
};
use std::io::stdout;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const BANNER: &str = r#"
╔═══════════════════════════════════[ METGen ]══════════════════════════════════╗
║                                                                               ║
║            ███╗   ███╗███████╗████████╗ ██████╗ ███████╗███╗   ██╗            ║     Simulator
║            ████╗ ████║██╔════╝╚══██╔══╝██╔════╝ ██╔════╝████╗  ██║            ║        Use
║            ██╔████╔██║█████╗     ██║   ██║  ███╗█████╗  ██╔██╗ ██║            ║       ONLY
║            ██║╚██╔╝██║██╔══╝     ██║   ██║   ██║██╔══╝  ██║╚██╗██║            ║      NOT FOR
║            ██║ ╚═╝ ██║███████╗   ██║   ╚██████╔╝███████╗██║ ╚████║            ║      Aviation
║            ╚═╝     ╚═╝╚══════╝   ╚═╝    ╚═════╝ ╚══════╝╚═╝  ╚═══╝            ║        Use
║                                    [v{VERSION_PLACEHOLDER}]                                   ║
║            ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀             ║
╚═════════════════════════[ Synthesized METAR Generation ]══════════════════════╝"#;

// Color schemes for different UI elements
const BANNER_COLORS: [Color; 3] = [Color::Cyan, Color::Blue, Color::White];
const MENU_COLORS: [Color; 2] = [Color::Yellow, Color::DarkYellow];
const HEADER_COLORS: [Color; 2] = [Color::Magenta, Color::DarkMagenta];

pub fn clear_screen() -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show
    )?;
    stdout.flush()?;
    Ok(())
}

pub fn draw_banner() -> std::io::Result<()> {
    let mut stdout = stdout();
    let banner_with_version = BANNER.replace("{VERSION_PLACEHOLDER}", VERSION);
    
    // Apply color cycling effect to the banner
    for (i, line) in banner_with_version.lines().enumerate() {
        let color = BANNER_COLORS[i % BANNER_COLORS.len()];
        execute!(
            stdout,
            SetAttribute(Attribute::Bold),
            SetForegroundColor(color),
            SetBackgroundColor(Color::Black)
        )?;
        println!("{}", line);
    }
    
    execute!(
        stdout,
        SetAttribute(Attribute::Reset),
        SetBackgroundColor(Color::Reset)
    )?;
    Ok(())
}

pub fn draw_menu_box(title: &str, options: &[&str]) -> std::io::Result<()> {
    let mut stdout = stdout();
    let width = options.iter().map(|s| s.len()).max().unwrap_or(0) + 4;
    let width = width.max(title.len() + 4);

    // Draw top border with title using retro styling
    execute!(stdout, SetForegroundColor(MENU_COLORS[0]))?;
    println!("╔═[{}]{}╗", title, "═".repeat(width - title.len() - 3));
    
    // Draw options with alternating colors
    for (i, option) in options.iter().enumerate() {
        let color = MENU_COLORS[i % MENU_COLORS.len()];
        execute!(stdout, SetForegroundColor(color))?;
        println!("║ {} {}{} ║", 
            if i == 0 { "►" } else { "•" },
            option,
            " ".repeat(width - option.len() - 4)
        );
    }

    // Draw bottom border
    execute!(stdout, SetForegroundColor(MENU_COLORS[0]))?;
    println!("╚{}╝", "═".repeat(width));
    execute!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

pub fn draw_section_header(title: &str) -> std::io::Result<()> {
    let mut stdout = stdout();
    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let padding = (term_width - title.len() - 4).max(0) / 2;
    
    execute!(stdout, SetForegroundColor(HEADER_COLORS[0]))?;
    println!("\n╔{}╗", "═".repeat(term_width - 2));
    
    execute!(stdout, SetForegroundColor(HEADER_COLORS[1]))?;
    println!("║{}{}{} ║", 
        " ".repeat(padding),
        title,
        " ".repeat(term_width - padding - title.len() - 3)
    );
    
    execute!(stdout, SetForegroundColor(HEADER_COLORS[0]))?;
    println!("╚{}╝", "═".repeat(term_width - 2));
    execute!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

pub fn draw_input_prompt(prompt: &str) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::Show,
        SetForegroundColor(Color::Green),
        SetAttribute(Attribute::Bold)
    )?;
    print!("┌─[INPUT]─── {}\n└──╼ ", prompt);
    stdout.flush()?;
    execute!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

pub fn draw_output_box(content: &str) -> std::io::Result<()> {
    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let width = term_width.saturating_sub(4);  // Account for borders and padding safely
    
    println!("╔{}╗", "═".repeat(width));
    for line in content.lines() {
        if line.len() < width {
            println!("║ {}{} ║", line, " ".repeat(width.saturating_sub(line.len()).saturating_sub(2)));
        } else {
            // Word wrap implementation
            let mut current_line = String::new();
            
            for word in line.split_whitespace() {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + word.len() + 1 < width.saturating_sub(2) {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    // Print current line and start a new one
                    println!("║ {}{} ║", current_line, " ".repeat(width.saturating_sub(current_line.len()).saturating_sub(2)));
                    current_line = word.to_string();
                }
            }
            
            // Print any remaining text
            if !current_line.is_empty() {
                println!("║ {}{} ║", current_line, " ".repeat(width.saturating_sub(current_line.len()).saturating_sub(2)));
            }
        }
    }
    println!("╚{}╝", "═".repeat(width));
    Ok(())
}

pub fn draw_error_box(error: &str) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(Color::Red), SetAttribute(Attribute::Bold))?;
    draw_output_box(error)?;
    execute!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

pub fn draw_success_box(message: &str) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(Color::Green), SetAttribute(Attribute::Bold))?;
    draw_output_box(message)?;
    execute!(stdout, SetAttribute(Attribute::Reset))?;
    Ok(())
}

pub fn read_single_char() -> std::io::Result<char> {
    let mut stdout = stdout();
    stdout.flush()?;
    
    enable_raw_mode()?;
    
    // More aggressive input buffer clearing
    while crossterm::event::poll(std::time::Duration::from_millis(10))? {
        let _ = read()?;
    }
    
    let result: std::io::Result<char> = loop {
        // Clear any input that might have accumulated during the loop
        while crossterm::event::poll(std::time::Duration::from_millis(0))? {
            let _ = read()?;
        }
        
        if let Event::Key(key_event) = read()? {
            match key_event.code {
                KeyCode::Char(c) => {
                    break Ok(c);
                },
                KeyCode::Enter => {
                    break Err(io::Error::new(io::ErrorKind::Other, "Enter pressed"));
                },
                _ => continue
            }
        }
    };
    
    // Clear any remaining input before returning
    while crossterm::event::poll(std::time::Duration::from_millis(0))? {
        let _ = read()?;
    }
    
    disable_raw_mode()?;
    println!(); // Move to next line after character input
    result
} 