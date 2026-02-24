use std::collections::HashMap;
use ab_glyph::{Font, FontRef, Glyph, PxScale, point};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use image::{GrayImage, Luma};
use slotmap::new_key_type;

use crate::render::{mesh::Mesh, texture::TextureHandle};

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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GlyphVertex {
    pos: Vec2,
    uv: Vec2,
}

impl GlyphVertex {
    const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
    ];

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
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

            let ascent_px = (font.ascent_unscaled() / 1000.0 * scale.x) as i32;
            let baseline_offset = (scale.x * 0.1) as i32;

            outlined.draw(|x, y, v| {
                let px = cursor_x + x;
                let py = cursor_y + (ascent_px + bounds.min.y as i32 + y as i32 - baseline_offset) as u32;
                atlas.image[(px, py)] = Luma([(v * 255.0) as u8]);
            });

            // let min_x = cursor_x;
            // let min_y = cursor_y;
            // let max_x = cursor_x + w - 1;
            // let max_y = cursor_y + h - 1;

            // for x in min_x..=max_x {
            //     if min_y < atlas.size {
            //         atlas.image[(x, min_y)] = Luma([0]);
            //         if x % 2 == 0 {
            //             atlas.image[(x, min_y)] = Luma([255]);
            //         }
            //     }
            //     if max_y < atlas.size {
            //         atlas.image[(x, max_y)] = Luma([0]);
            //         if x % 2 == 0 {
            //             atlas.image[(x, max_y)] = Luma([255]);
            //         }
            //     }
            // }
            // for y in min_y..=max_y {
            //     if min_x < atlas.size {
            //         atlas.image[(min_x, y)] = Luma([0]);
            //         if y % 2 == 0 {
            //             atlas.image[(min_x, y)] = Luma([255]);
            //         }
            //     }
            //     if max_x < atlas.size {
            //         atlas.image[(max_x, y)] = Luma([0]);
            //         if y % 2 == 0 {
            //             atlas.image[(max_x, y)] = Luma([255]);
            //         }
            //     }
            // }

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

    pub fn generate_mesh(&self, text: &str, start: Vec2) -> Mesh<GlyphVertex> {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut cursor_x = start[0];
        let cursor_y = start[1];
        let mut idx_offset = 0;

        for c in text.chars() {
            if let Some(glyph) = self.glyphs.get(&c) {
                let x0 = cursor_x + glyph.bearing[0];
                let y0 = cursor_y - glyph.bearing[1];
                let x1 = x0 + glyph.size[0];
                let y1 = y0 + glyph.size[1];

                vertices.push(GlyphVertex { pos: Vec2::new(x0, y0), uv: glyph.uv_min });
                vertices.push(GlyphVertex { pos: Vec2::new(x1, y0), uv: Vec2::new(glyph.uv_max[0], glyph.uv_min[1]) });
                vertices.push(GlyphVertex { pos: Vec2::new(x1, y1), uv: glyph.uv_max });
                vertices.push(GlyphVertex { pos: Vec2::new(x0, y1), uv: Vec2::new(glyph.uv_min[0], glyph.uv_max[1]) });

                indices.extend_from_slice(&[
                    idx_offset, idx_offset+1, idx_offset+2,
                    idx_offset, idx_offset+2, idx_offset+3,
                ]);
                idx_offset += 4;

                cursor_x += glyph.advance;
            }
        }

        Mesh::new(vertices, indices)
    }
}