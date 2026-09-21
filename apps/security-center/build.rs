use std::{fs, io, path::Path};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        if let Err(error) = ensure_windows_icon() {
            panic!("failed to prepare Windows Security Center icon: {error}");
        }
    }

    tauri_build::build();
}

fn ensure_windows_icon() -> io::Result<()> {
    let directory = Path::new("icons");
    let icon_path = directory.join("icon.ico");
    fs::create_dir_all(directory)?;

    if icon_path.exists() {
        return Ok(());
    }

    const WIDTH: u8 = 32;
    const HEIGHT: u8 = 32;
    const PIXEL_BYTES: u32 = 32 * 32 * 4;
    const MASK_BYTES: u32 = 32 * 4;
    const DIB_BYTES: u32 = 40 + PIXEL_BYTES + MASK_BYTES;

    let mut icon = Vec::with_capacity((22 + DIB_BYTES) as usize);

    icon.extend_from_slice(&0_u16.to_le_bytes());
    icon.extend_from_slice(&1_u16.to_le_bytes());
    icon.extend_from_slice(&1_u16.to_le_bytes());

    icon.push(WIDTH);
    icon.push(HEIGHT);
    icon.push(0);
    icon.push(0);
    icon.extend_from_slice(&1_u16.to_le_bytes());
    icon.extend_from_slice(&32_u16.to_le_bytes());
    icon.extend_from_slice(&DIB_BYTES.to_le_bytes());
    icon.extend_from_slice(&22_u32.to_le_bytes());

    icon.extend_from_slice(&40_u32.to_le_bytes());
    icon.extend_from_slice(&32_i32.to_le_bytes());
    icon.extend_from_slice(&64_i32.to_le_bytes());
    icon.extend_from_slice(&1_u16.to_le_bytes());
    icon.extend_from_slice(&32_u16.to_le_bytes());
    icon.extend_from_slice(&0_u32.to_le_bytes());
    icon.extend_from_slice(&PIXEL_BYTES.to_le_bytes());
    icon.extend_from_slice(&0_i32.to_le_bytes());
    icon.extend_from_slice(&0_i32.to_le_bytes());
    icon.extend_from_slice(&0_u32.to_le_bytes());
    icon.extend_from_slice(&0_u32.to_le_bytes());

    for y in (0_i32..32).rev() {
        for x in 0_i32..32 {
            let inset = x >= 4 && x <= 27 && y >= 4 && y <= 27;
            let forge_core = (x - 16).abs() + (y - 16).abs() <= 9;
            let (red, green, blue) = if forge_core {
                (32_u8, 38_u8, 50_u8)
            } else if inset {
                (255_u8, 138_u8, 43_u8)
            } else {
                (243_u8, 107_u8, 21_u8)
            };
            icon.extend_from_slice(&[blue, green, red, 255]);
        }
    }

    icon.extend(std::iter::repeat_n(0_u8, MASK_BYTES as usize));
    fs::write(icon_path, icon)
}
