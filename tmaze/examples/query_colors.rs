use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};

fn query_color(index: u8) -> io::Result<Option<(u8, u8, u8)>> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    // Send OSC 4 query for this color index
    write!(stdout, "\x1b]4;{};?\x07", index)?;
    stdout.flush()?;

    // Read response (blocking). It will look like:
    // ESC ] 4 ; <index> ; rgb:RRRR/GGGG/BBBB BEL
    let mut buf = [0u8; 128];
    let n = stdin.read(&mut buf)?;

    let s = String::from_utf8_lossy(&buf[..n]);

    // Try to extract rgb:xxxx/yyyy/zzzz
    if let Some(pos) = s.find("rgb:") {
        let rgb = &s[pos + 4..];
        let parts: Vec<&str> = rgb.split(&['/', '\x07'][..]).collect();
        if parts.len() >= 3 {
            let r = u16::from_str_radix(parts[0], 16).unwrap_or(0);
            let g = u16::from_str_radix(parts[1], 16).unwrap_or(0);
            let b = u16::from_str_radix(parts[2], 16).unwrap_or(0);

            // Downscale 16-bit to 8-bit
            return Ok(Some(((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8)));
        }
    }

    Ok(None)
}

fn main() -> io::Result<()> {
    for i in 0..16 {
        enable_raw_mode()?;
        let query_color = query_color(i)?;
        disable_raw_mode()?;
        if let Some((r, g, b)) = query_color {
            println!("Color {} = #{:02X}{:02X}{:02X}", i, r, g, b);
        } else {
            println!("Color {} = (no response)", i);
        }
    }

    Ok(())
}
