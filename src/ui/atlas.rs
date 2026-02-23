use std::collections::HashMap;
use ab_glyph::{Font, FontRef, Glyph, PxScale, point};
use glam::Vec2;
use image::{GrayImage, Luma};
use slotmap::new_key_type;

use crate::render::texture::TextureHandle;

pub const PADDING: u32 = 2;
pub const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.,;:!()";
// pub const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.,;:!?()АБВГҐДЕЄЖЗИІЇЙКЛМНОПРСТУФХЦЧШЩЬЮЯабвгґдеєжзиіїйклмнопрстуфхцчшщьюя0123456789";

new_key_type! {
    pub struct FontHandle;
}

pub struct FontData {
    pub(crate) font: FontRef<'static>,
    pub(crate) atlases: AtlasSet,
}

impl FontData {
    pub fn new(font: FontRef<'static>) -> FontData {
        FontData {
            font,
            atlases: AtlasSet::new(),
        }
    }
}

#[derive(Default)]
pub struct AtlasSet {
    map: HashMap<u32, Atlas>,
}

pub const fn atlas_size(font_size: u32) -> Option<usize> {
    if font_size > 28 { Some(1024) } 
    else if font_size > 0 { Some(512) } 
    else { None }
}

impl AtlasSet {
    pub fn new() -> AtlasSet {
        AtlasSet::default()
    }

    pub fn register_atlas(
        &mut self, 
        font_size: u32, 
        font: &FontRef<'_>,
        texture: TextureHandle,
    ) {
        if self.map.contains_key(&font_size) {
            log::error!("Atlas for font size {} already exists", font_size);
            return;
        }
        let Some(atlas_size) = atlas_size(font_size) else {
            log::error!("Invalid size {} for font atlas", font_size);
            return;
        };
        let scale = PxScale::from(font_size as f32);
        
        let mut glyphs = vec![];
        let mut caret = point(0.0, scale.x);
        for c in CHARSET.chars() {
            if let Some(glyph_id) = font.glyph_id(c).into() {
                let glyph = Glyph {
                    id: glyph_id,
                    scale,
                    position: caret,
                };
                caret.x += font.h_advance_unscaled(glyph.id) * scale.x;
                glyphs.push((c, glyph));
            }
        }

        let mut atlas = Atlas {
            image: GrayImage::new(atlas_size as u32, atlas_size as u32),
            handle: texture,
            size: atlas_size as u32,
            glyphs: HashMap::new(),
        };

        let mut cursor_x = 0;
        let mut cursor_y = 0;
        let mut row_height = 0;

        for (c, glyph) in glyphs {
            let id = glyph.id;
            let outlined = font.outline_glyph(glyph).unwrap();
            let bounds = outlined.px_bounds();
            let w = bounds.width().ceil() as u32 + PADDING;
            let h = bounds.height().ceil() as u32 + PADDING;

            if cursor_x + w > atlas_size as u32 {
                cursor_x = 0;
                cursor_y += row_height;
                row_height = 0;
            }

            let uv_min = Vec2::new(
                cursor_x as f32 / atlas_size as f32, 
                cursor_y as f32 / atlas_size as f32,
            );
            let uv_max = Vec2::new(
                (cursor_x + w) as f32 / atlas_size as f32, 
                (cursor_y + h) as f32 / atlas_size as f32,
            );

            atlas.glyphs.insert(c, GlyphInfo {
                uv_min,
                uv_max,
                size: Vec2::new(w as f32, h as f32),
                bearing: Vec2::new(bounds.min.x, bounds.min.y),
                advance: font.h_advance_unscaled(id) * scale.x,
            });

            outlined.draw(|x, y, v| {
                let px = cursor_x + x;
                let py = cursor_y + y;
                atlas.image[(px, py)] = Luma([(v * 255.0) as u8]);
            });

            cursor_x += w;
            row_height = row_height.max(h);
        }

        self.map.insert(font_size, atlas);
    }

    pub fn get_atlas(&self, font_size: u32) -> Option<&Atlas> {
        self.map.get(&font_size)
    }
}

pub struct GlyphInfo {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub size: Vec2,
    pub bearing: Vec2,
    pub advance: f32,
}

pub struct Atlas {
    image: GrayImage,
    handle: TextureHandle,
    size: u32,
    glyphs: HashMap<char, GlyphInfo>,
}

impl Atlas {
    pub fn texture(&self) -> TextureHandle {
        self.handle
    }
    
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn image(&self) -> &GrayImage {
        &self.image
    }
}