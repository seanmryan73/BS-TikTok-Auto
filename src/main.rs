// Author  : Sean Ryan <seanmryan@gmail.com>
// Company : BagPipes
// Version : 2026.06.28

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]
#![allow(dead_code, unused_variables, unused_imports)]

mod app;
mod auth;
mod pipeline;
mod settings;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    settings::ensure_dirs();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("BS TikTok Auto")
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 560.0])
            .with_icon(build_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "BS TikTok Auto",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}

fn build_icon() -> eframe::egui::IconData {
    const W: usize = 32;
    const H: usize = 32;
    let mut pixels = vec![0u8; W * H * 4];

    let bg = [0x07u8, 0x07, 0x0B, 0xFF];
    let fg = [0xFFu8, 0x00, 0x44, 0xFF];

    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&bg);
    }

    #[rustfmt::skip]
    let b_glyph: [[u8; 5]; 7] = [
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
        [1,0,0,0,1],
        [1,0,0,0,1],
        [1,1,1,1,0],
    ];

    #[rustfmt::skip]
    let t_glyph: [[u8; 5]; 7] = [
        [1,1,1,1,1],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
        [0,0,1,0,0],
    ];

    draw_glyph(&mut pixels, W, &b_glyph, 3, 9, &fg);
    draw_glyph(&mut pixels, W, &t_glyph, 16, 9, &fg);

    eframe::egui::IconData {
        rgba: pixels,
        width: W as u32,
        height: H as u32,
    }
}

fn draw_glyph(pixels: &mut [u8], stride: usize, glyph: &[[u8; 5]; 7], ox: usize, oy: usize, color: &[u8; 4]) {
    for (row, bits) in glyph.iter().enumerate() {
        for (col, &lit) in bits.iter().enumerate() {
            if lit == 0 {
                continue;
            }
            for dy in 0..2usize {
                for dx in 0..2usize {
                    let x = ox + col * 2 + dx;
                    let y = oy + row * 2 + dy;
                    if x < stride && y < 32 {
                        let idx = (y * stride + x) * 4;
                        pixels[idx..idx + 4].copy_from_slice(color);
                    }
                }
            }
        }
    }
}
