//! Dune Client: the launcher's background is the brand kit's night picture (`fondo`), not the
//! upstream pixel sunset. New code written for Dune Client.
//!
//! The picture is drawn "cover" over the whole window by the GPU; the glass plates (`glass.rs`)
//! need a blurred copy of what is behind them, which is made here from the same picture, one
//! texel per `TEXEL` logical pixels, whenever the window size or the blur changes.
use crate::scene::Backdrop;
use gpui::{img, prelude::*, AnyElement, ObjectFit, RenderImage, Window};
use image::RgbaImage;
use std::sync::Arc;

const SOURCE: &[u8] = include_bytes!("../../assets/brand/dune-fondo-noche-1920.webp");
/// Logical pixels per texel of the blurred copy (as `glass::PIECE`).
const TEXEL: f64 = 4.;

pub struct Picture {
    source: RgbaImage,
    shown: Option<Arc<RenderImage>>,
    /// The window size (logical px, rounded) and blur the plates' backdrop was made for.
    backdrop_for: Option<((u32, u32), u32)>,
}

impl Picture {
    pub fn new() -> Option<Picture> {
        let source = image::load_from_memory_with_format(SOURCE, image::ImageFormat::WebP)
            .ok()?
            .to_rgba8();
        Some(Picture {
            source,
            shown: None,
            backdrop_for: None,
        })
    }

    fn render_image(&self) -> Arc<RenderImage> {
        let mut buf = self.source.clone();
        // GPUI wants BGRA.
        for px in buf.pixels_mut() {
            px.0.swap(0, 2);
        }
        Arc::new(RenderImage::new([image::Frame::new(buf)]))
    }

    /// The colour of the picture at a point of the window (logical px), "cover" fitted.
    fn sample(&self, x: f64, y: f64, win: (f64, f64)) -> [f64; 3] {
        let (sw, sh) = (self.source.width() as f64, self.source.height() as f64);
        let scale = (win.0 / sw).max(win.1 / sh);
        let (ox, oy) = ((win.0 - sw * scale) / 2., (win.1 - sh * scale) / 2.);
        let sx = ((x - ox) / scale).clamp(0., sw - 1.);
        let sy = ((y - oy) / scale).clamp(0., sh - 1.);
        let p = self.source.get_pixel(sx as u32, sy as u32).0;
        [p[0] as f64 / 255., p[1] as f64 / 255., p[2] as f64 / 255.]
    }

    /// The element behind the launcher; the plates' backdrop is updated on the way.
    pub fn element(&mut self, window: &mut Window, blur: f32, fast: bool) -> AnyElement {
        let vp = window.viewport_size();
        let win = (
            f64::from(f32::from(vp.width)).max(1.),
            f64::from(f32::from(vp.height)).max(1.),
        );
        let key = ((win.0.round() as u32, win.1.round() as u32), blur.to_bits());
        if !fast && self.backdrop_for != Some(key) {
            let w = (win.0 / TEXEL).ceil() as usize + 1;
            let h = (win.1 / TEXEL).ceil() as usize + 1;
            let mut base = vec![[0f64; 3]; w * h];
            for (i, c) in base.iter_mut().enumerate() {
                let (x, y) = (
                    ((i % w) as f64 + 0.5) * TEXEL,
                    ((i / w) as f64 + 0.5) * TEXEL,
                );
                *c = self.sample(x, y, win);
            }
            let backdrop = Backdrop::from_texels(&base, w, h, TEXEL, blur as f64, false);
            crate::glass::put_backdrop(backdrop, None);
            self.backdrop_for = Some(key);
        }
        if self.shown.is_none() {
            self.shown = Some(self.render_image());
        }
        let image = self.shown.clone().expect("made above");
        img(image)
            .object_fit(ObjectFit::Cover)
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .into_any_element()
    }
}
