use std::collections::HashMap;
use ab_glyph::{Font, FontRef, Glyph, Point, PxScale, ScaleFont, point};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use image::{GrayImage, Luma};
use slotmap::new_key_type;

use crate::render::{mesh::Mesh, texture::TextureHandle};

pub const PADDING: u32 = 2;
pub const CHARSET: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.,;:!()";
// TODO: implement other glyphs in the font
// pub const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.,;:!?()АБВГҐДЕЄЖЗИІЇЙКЛМНОПРСТУФХЦЧШЩЬЮЯабвгґдеєжзиіїйклмнопрстуфхцчшщьюя0123456789 ";

new_key_type! {
    pub struct FontHandle;
}

pub struct FontData {
    pub(crate) font: FontRef<'static>,
    pub(crate) atlases: AtlasSet,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct GlyphVertex {
    pub pos: Vec2,
    pub uv: Vec2,
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
        let scaled_font = font.as_scaled(scale);
        
        let mut glyphs = vec![];
        layout(&scaled_font, point(0.0, 0.0), atlas_size as f32, CHARSET, &mut glyphs);

        let mut atlas = Atlas {
            image: GrayImage::new(atlas_size as u32, atlas_size as u32),
            handle: texture,
            size: atlas_size as u32,
            glyphs: HashMap::new(),
        };

        for (c, line, g) in glyphs {
            if let Some(og) = scaled_font.outline_glyph(g.clone()) {
                let bounds = og.px_bounds();
                    
                og.draw(|x, y, v| {
                    let x = x as f32 + bounds.min.x;
                    let y = y as f32 + bounds.min.y;
                    if x >= 0.0 && (x as usize) < atlas_size && y >= 0.0 && (y as usize) < atlas_size {
                        atlas.image[(x as u32, y as u32)] = Luma([(v * 255.0) as u8]);
                    }
                });

                let uv_min = Vec2::new(
                    bounds.min.x / atlas_size as f32, 
                    bounds.min.y / atlas_size as f32,
                );

                let uv_max = Vec2::new(
                    (bounds.min.x + bounds.width()) / atlas_size as f32, 
                    (bounds.min.y + bounds.height()) / atlas_size as f32,
                );

                let v_advance = scaled_font.height() + scaled_font.line_gap();
                let offset = scaled_font.ascent() + line as f32 * v_advance;

                atlas.glyphs.insert(c, GlyphInfo {
                    uv_min,
                    uv_max,
                    size: Vec2::new(bounds.width(), bounds.height()),
                    advance: scaled_font.h_advance(g.id),
                    bearing: Vec2::new(bounds.min.x, bounds.min.y),
                    offset,
                });
            } else {
                if c == ' ' {
                    atlas.glyphs.insert(c, GlyphInfo {
                        uv_min: Vec2::ZERO,
                        uv_max: Vec2::ZERO,
                        size: Vec2::ZERO,
                        advance: scaled_font.h_advance(g.id),
                        bearing: Vec2::ZERO,
                        offset: 0.0,
                    });
                } else {
                    log::warn!("Failed to outline glyph `{c}`");
                }
            }
        }

        self.map.insert(font_size, atlas);
    }

    pub fn get_atlas(&self, font_size: u32) -> Option<&Atlas> {
        self.map.get(&font_size)
    }
}

pub fn layout<F, SF>(
    font: SF,
    position: Point,
    atlas_size: f32,
    text: &str,
    target: &mut Vec<(char, u32, Glyph)>,
) where
    F: Font,
    SF: ScaleFont<F>,
{
    let mut line = 0;
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = point(position.x, caret.y + v_advance);
                last_glyph = None;
            }
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);

        if !c.is_whitespace() && caret.x > position.x + atlas_size {
            caret = point(position.x, caret.y + v_advance);
            glyph.position = caret;
            last_glyph = None;
            caret.x += font.h_advance(glyph.id);
            line += 1;
        }
        
        target.push((c, line, glyph));
    }
}

pub struct GlyphInfo {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub size: Vec2,
    pub advance: f32,
    pub bearing: Vec2,
    pub offset: f32,
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
        let mut vertices = Vec::with_capacity(text.len() * 4);
        let mut indices = Vec::with_capacity(text.len() * 6);
        let mut cursor_x = start[0];
        let cursor_y = start[1];
        let mut idx_offset = 0;

        for c in text.chars() {
            if let Some(glyph) = self.glyphs.get(&c) {
                if glyph.size == Vec2::ZERO {
                    cursor_x += glyph.advance;
                    continue;
                }

                let x0 = cursor_x;
                let y0 = cursor_y - ((glyph.bearing.y) + glyph.size[1]) + glyph.offset;
                let x1 = x0 + glyph.size[0];
                let y1 = y0 + glyph.size[1];

                vertices.push(GlyphVertex { pos: Vec2::new(x0, y0) / self.size as f32, uv: Vec2::new(glyph.uv_min[0], glyph.uv_max[1]) });
                vertices.push(GlyphVertex { pos: Vec2::new(x1, y0) / self.size as f32, uv: Vec2::new(glyph.uv_max[0], glyph.uv_max[1]) });
                vertices.push(GlyphVertex { pos: Vec2::new(x1, y1) / self.size as f32, uv: Vec2::new(glyph.uv_max[0], glyph.uv_min[1]) });
                vertices.push(GlyphVertex { pos: Vec2::new(x0, y1) / self.size as f32, uv: Vec2::new(glyph.uv_min[0], glyph.uv_min[1]) });

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