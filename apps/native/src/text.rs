use crate::model::Point;
use memmap2::Mmap;
use std::{fs::File, path::Path, sync::OnceLock};
use ttf_parser::{Face, GlyphId};

struct MappedFont {
    data: Mmap,
    collection_index: u32,
}

impl MappedFont {
    fn open(path: &Path) -> Option<Self> {
        let file = File::open(path).ok()?;
        let data = unsafe { Mmap::map(&file).ok()? };
        Face::parse(&data, 0).ok()?;
        Some(Self {
            data,
            collection_index: 0,
        })
    }

    fn face(&self) -> Option<Face<'_>> {
        Face::parse(&self.data, self.collection_index).ok()
    }
}

fn load_system_fonts() -> Vec<MappedFont> {
    font_candidates()
        .iter()
        .filter_map(|path| MappedFont::open(path))
        .collect()
}

#[cfg(target_os = "macos")]
fn font_candidates() -> Vec<&'static Path> {
    vec![
        Path::new("/System/Library/Fonts/SFNS.ttf"),
        Path::new("/System/Library/Fonts/Hiragino Sans GB.ttc"),
    ]
}

#[cfg(target_os = "windows")]
fn font_candidates() -> Vec<&'static Path> {
    vec![
        Path::new(r"C:\Windows\Fonts\segoeui.ttf"),
        Path::new(r"C:\Windows\Fonts\YuGothM.ttc"),
    ]
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn font_candidates() -> Vec<&'static Path> {
    vec![
        Path::new("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
        Path::new("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
    ]
}

fn fonts() -> &'static [MappedFont] {
    static FONTS: OnceLock<Vec<MappedFont>> = OnceLock::new();
    FONTS.get_or_init(load_system_fonts)
}

pub(crate) fn glyph_for(character: char) -> Option<(Face<'static>, GlyphId)> {
    fonts().iter().find_map(|font| {
        let face = font.face()?;
        let glyph = face.glyph_index(character)?;
        Some((face, glyph))
    })
}

/// Use the same glyph advances as rendering so Japanese labels can be grabbed
/// across their entire visible width and are not cropped during Send.
pub(crate) fn measure_text(text: &str, font_size: f32) -> Point {
    let mut width = font_size * 0.5;
    let mut line_width = 0.0_f32;
    let mut lines = 1;
    for character in text.chars() {
        if character == '\n' {
            width = width.max(line_width);
            line_width = 0.0;
            lines += 1;
        } else {
            line_width += glyph_for(character)
                .and_then(|(face, glyph)| {
                    face.glyph_hor_advance(glyph)
                        .map(|advance| advance as f32 * font_size / face.units_per_em() as f32)
                })
                .unwrap_or(font_size * 0.58);
        }
    }
    Point::new(width.max(line_width), lines as f32 * font_size * 1.25)
}
