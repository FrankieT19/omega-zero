use std::path::Path;

use fontdue::{Font, FontSettings};

const GILL: &str = "/System/Library/Fonts/Supplemental/GillSans.ttc";
const FUTURA: &str = "/System/Library/Fonts/Supplemental/Futura.ttc";
const GILL_ITALIC: u32 = 2;
const FUTURA_MEDIUM: u32 = 0;
const FUTURA_BOLD: u32 = 2;
const FUTURA_HEAVY: u32 = 4;

const OVER: u32 = 4;

struct Run<'a> {
    font: &'a Font,
    text: &'a str,
    px: f32,
    tracking: f32,
    lean: f32,
    bold: f32,
    /// How much wider than the face's own letters these are drawn.
    wide: f32,
    /// Whether an A is drawn as an upturned V, with no bar, as in ADVANCE on a GBA.
    peaked: bool,
}

impl Run<'_> {
    fn glyph(&self, c: char) -> char {
        if self.peaked && c == 'A' {
            'V'
        } else {
            c
        }
    }

    fn scaled(&self, k: f32) -> Run<'_> {
        Run {
            font: self.font,
            text: self.text,
            px: self.px * k,
            tracking: self.tracking * k,
            lean: self.lean,
            bold: self.bold * k,
            wide: self.wide,
            peaked: self.peaked,
        }
    }
}

struct Canvas {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

impl Canvas {
    fn new(w: u32, h: u32) -> Canvas {
        Canvas {
            w: w * OVER,
            h: h * OVER,
            px: vec![0; (w * h * OVER * OVER) as usize],
        }
    }

    fn width(run: &Run) -> f32 {
        run.text
            .chars()
            .map(|c| run.font.metrics(run.glyph(c), run.px).advance_width * run.wide + run.tracking)
            .sum::<f32>()
            - run.tracking
    }

    fn line(
        &mut self,
        runs: &[Run],
        gaps: &[f32],
        cx: f32,
        baseline: f32,
        max_w: f32,
    ) -> (f32, f32) {
        let width = |k: f32| -> f32 {
            runs.iter()
                .map(|r| Canvas::width(&r.scaled(k)))
                .sum::<f32>()
                + gaps.iter().sum::<f32>() * k
        };
        let mut k = 1.0;
        while k > 0.3 && width(k) > max_w {
            k -= 0.01;
        }
        let mut pen = cx - width(k) / 2.0;
        for (i, run) in runs.iter().enumerate() {
            let run = run.scaled(k);
            self.run(&run, pen, baseline);
            pen += Canvas::width(&run) + gaps.get(i).copied().unwrap_or(0.0) * k;
        }
        (pen, k)
    }

    fn run(&mut self, run: &Run, x: f32, baseline: f32) {
        let o = OVER as f32;
        let mut pen = x * o;
        let base = baseline * o;
        let grow = (run.bold * o).round() as i32;
        for ch in run.text.chars() {
            let glyph = run.glyph(ch);
            let (m, cov) = run.font.rasterize(glyph, run.px * o);
            let top = base - (m.height as f32 + m.ymin as f32);
            let cols = (m.width as f32 * run.wide).round() as usize;
            for gy in 0..m.height {
                // An upturned V keeps its box, so it still stands on the baseline.
                let sy = if glyph == ch { gy } else { m.height - 1 - gy };
                for ox in 0..cols {
                    let gx = ((ox as f32 / run.wide) as usize).min(m.width - 1);
                    let a = cov[sy * m.width + gx];
                    if a == 0 {
                        continue;
                    }
                    let y = (top + gy as f32).round() as i32;
                    let lean = (base - y as f32) * run.lean;
                    let x0 = (pen + m.xmin as f32 * run.wide + ox as f32 + lean).round() as i32;
                    for dx in 0..=grow {
                        self.put(x0 + dx, y, a);
                    }
                }
            }
            pen += m.advance_width * run.wide + run.tracking * o;
        }
    }

    fn put(&mut self, x: i32, y: i32, a: u8) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let i = (y as u32 * self.w + x as u32) as usize;
        self.px[i] = self.px[i].max(a);
    }

    fn down(&self) -> (u32, u32, Vec<u8>) {
        let (w, h) = (self.w / OVER, self.h / OVER);
        let mut out = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0u32;
                for sy in 0..OVER {
                    for sx in 0..OVER {
                        sum += self.px[((y * OVER + sy) * self.w + x * OVER + sx) as usize] as u32;
                    }
                }
                out.push((sum / (OVER * OVER)) as u8);
            }
        }
        (w, h, out)
    }

    fn write_edged(&self, path: &Path, r: f32) {
        let (w, h, fill) = self.down();
        write_edged(path, w, h, &fill, r);
    }

    fn write(&self, path: &Path) {
        let (w, h, out) = self.down();
        write_mask(path, w, h, &out);
    }

    /// Writes only the box the ink covers, for a mark that is placed by its own size.
    fn write_trimmed(&self, path: &Path) {
        let (w, h, out) = self.down();
        let inked = |x: u32, y: u32| out[(y * w + x) as usize] > 0;
        let xs: Vec<u32> = (0..w).filter(|&x| (0..h).any(|y| inked(x, y))).collect();
        let ys: Vec<u32> = (0..h).filter(|&y| (0..w).any(|x| inked(x, y))).collect();
        let (x0, x1) = (xs[0], xs[xs.len() - 1]);
        let (y0, y1) = (ys[0], ys[ys.len() - 1]);
        let (tw, th) = (x1 - x0 + 1, y1 - y0 + 1);
        let trimmed: Vec<u8> = (y0..=y1)
            .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
            .map(|(x, y)| out[(y * w + x) as usize])
            .collect();
        write_mask(path, tw, th, &trimmed);
    }
}

/// Writes `fill` white edged in black `r` pixels wide, as grey and alpha, trimmed to the edge.
fn write_edged(path: &Path, w: u32, h: u32, fill: &[u8], r: f32) {
    let reach = r.ceil() as i32;
    let at = |x: i32, y: i32| -> u8 {
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            0
        } else {
            fill[(y as u32 * w + x as u32) as usize]
        }
    };
    let mut edge = vec![0u8; fill.len()];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mut most = 0;
            for dy in -reach..=reach {
                for dx in -reach..=reach {
                    if ((dx * dx + dy * dy) as f32) <= r * r {
                        most = most.max(at(x + dx, y + dy));
                    }
                }
            }
            edge[(y as u32 * w + x as u32) as usize] = most;
        }
    }
    let inked = |x: u32, y: u32| edge[(y * w + x) as usize] > 0;
    let xs: Vec<u32> = (0..w).filter(|&x| (0..h).any(|y| inked(x, y))).collect();
    let ys: Vec<u32> = (0..h).filter(|&y| (0..w).any(|x| inked(x, y))).collect();
    let (x0, x1) = (xs[0], xs[xs.len() - 1]);
    let (y0, y1) = (ys[0], ys[ys.len() - 1]);
    let mut out = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = (y * w + x) as usize;
            let a = edge[i].max(fill[i]);
            let grey = if a == 0 {
                0
            } else {
                (fill[i] as u32 * 255 / a as u32) as u8
            };
            out.extend([grey, a]);
        }
    }
    let file = std::fs::File::create(path).expect("create mark");
    let mut enc = png::Encoder::new(file, x1 - x0 + 1, y1 - y0 + 1);
    enc.set_color(png::ColorType::GrayscaleAlpha);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut wr| wr.write_image_data(&out))
        .expect("write mark");
    println!("wrote {}", path.display());
}

fn write_mask(path: &Path, w: u32, h: u32, out: &[u8]) {
    {
        let file = std::fs::File::create(path).expect("create mask");
        let mut enc = png::Encoder::new(file, w, h);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .and_then(|mut wr| wr.write_image_data(out))
            .expect("write mask");
        println!("wrote {}", path.display());
    }
}

fn face(path: &str, index: u32) -> Font {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}; this runs on a Mac"));
    let settings = FontSettings {
        collection_index: index,
        ..FontSettings::default()
    };
    Font::from_bytes(bytes, settings).expect("font")
}

fn main() {
    let gill = face(GILL, GILL_ITALIC);
    let futura = face(FUTURA, FUTURA_MEDIUM);
    let futura_bold = face(FUTURA, FUTURA_BOLD);
    let futura_heavy = face(FUTURA, FUTURA_HEAVY);
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let (gb_w, gb_h) = slot_ui::seated_box(slot_store::Platform::Gb);

    let game_boy = |px: f32| Run {
        font: &gill,
        text: "GAME BOY",
        px,
        tracking: -0.6,
        lean: 0.0,
        bold: 0.6,
        wide: 1.0,
        peaked: false,
    };

    let mut gb = Canvas::new(gb_w, gb_h);
    let (right, k) = gb.line(
        &[
            Run {
                font: &futura_bold,
                text: "Nintendo",
                px: 13.0,
                tracking: -0.2,
                lean: 0.0,
                bold: 0.0,
                wide: 1.0,
                peaked: false,
            },
            game_boy(24.0),
        ],
        &[5.0],
        116.0,
        47.0,
        138.0,
    );
    gb.run(
        &Run {
            font: &futura,
            text: "TM",
            px: 5.5 * k,
            tracking: 0.0,
            lean: 0.0,
            bold: 0.0,
            wide: 1.0,
            peaked: false,
        },
        right + 1.0,
        47.0 - 12.0 * k,
    );
    gb.write(&assets.join("lettering_gb.png"));

    let mut gbc = Canvas::new(gb_w, gb_h);
    gbc.line(
        &[
            game_boy(22.0),
            Run {
                font: &futura_bold,
                text: "COLOR",
                px: 20.0,
                tracking: 0.4,
                lean: 0.2,
                bold: 0.0,
                wide: 1.0,
                peaked: false,
            },
        ],
        &[6.0],
        120.0,
        43.0,
        140.0,
    );
    gbc.write(&assets.join("lettering_gbc.png"));

    let (gba_w, gba_h) = slot_ui::seated_box(slot_store::Platform::Gba);
    let mut gba = Canvas::new(gba_w, gba_h);
    gba.line(
        &[
            Run {
                font: &futura_bold,
                text: "GAME BOY",
                px: 11.0,
                tracking: 0.0,
                lean: 0.2,
                bold: 0.0,
                wide: 1.0,
                peaked: false,
            },
            Run {
                font: &futura,
                text: "ADVANCE",
                px: 8.5,
                tracking: 1.6,
                lean: 0.0,
                bold: 0.0,
                wide: 1.0,
                peaked: false,
            },
        ],
        &[4.0],
        120.0,
        25.0,
        130.0,
    );
    gba.write(&assets.join("lettering_gba.png"));

    // The marks printed on the console's face under the screen, drawn large and trimmed to
    // their ink, to be shrunk to whatever band the canvas leaves.
    // The GBA's own mark: GAME BOY heavy and leaning, ADVANCE wide with its A's upturned V's,
    // white edged in black, so it reads on the console's grey as on the real lens.
    let mut bezel_gba = Canvas::new(1100, 160);
    let gba_game_boy = Run {
        font: &futura_heavy,
        text: "GAME BOY",
        px: 100.0,
        tracking: -6.0,
        lean: 0.2,
        bold: 0.0,
        wide: 1.08,
        peaked: false,
    };
    let (left, baseline) = (30.0, 120.0);
    bezel_gba.run(&gba_game_boy, left, baseline);
    let registered = left + Canvas::width(&gba_game_boy) + 2.0;
    bezel_gba.run(
        &Run {
            font: &futura,
            text: "\u{ae}",
            px: 23.0,
            tracking: 0.0,
            lean: 0.0,
            bold: 0.0,
            wide: 1.0,
            peaked: false,
        },
        registered,
        baseline + 8.0,
    );
    bezel_gba.run(
        &Run {
            font: &futura_bold,
            text: "ADVANCE",
            px: 64.0,
            tracking: 5.0,
            lean: 0.0,
            bold: 3.0,
            wide: 1.55,
            peaked: true,
        },
        registered + 20.0,
        baseline,
    );
    bezel_gba.write_edged(&assets.join("bezel_gba.png"), 9.0);

    let mut bezel_gb = Canvas::new(1000, 160);
    let (right, k) = bezel_gb.line(
        &[
            Run {
                font: &futura_bold,
                text: "Nintendo",
                px: 52.0,
                tracking: -0.8,
                lean: 0.0,
                bold: 0.0,
                wide: 1.0,
                peaked: false,
            },
            game_boy(96.0),
        ],
        &[20.0],
        480.0,
        120.0,
        900.0,
    );
    bezel_gb.run(
        &Run {
            font: &futura,
            text: "TM",
            px: 22.0 * k,
            tracking: 0.0,
            lean: 0.0,
            bold: 0.0,
            wide: 1.0,
            peaked: false,
        },
        right + 4.0,
        120.0 - 48.0 * k,
    );
    bezel_gb.write_trimmed(&assets.join("bezel_gb.png"));

    let mut bezel_gbc = Canvas::new(1000, 160);
    bezel_gbc.line(
        &[
            game_boy(88.0),
            Run {
                font: &futura_bold,
                text: "COLOR",
                px: 80.0,
                tracking: 1.6,
                lean: 0.2,
                bold: 0.0,
                wide: 1.0,
                peaked: false,
            },
        ],
        &[24.0],
        500.0,
        120.0,
        940.0,
    );
    bezel_gbc.write_trimmed(&assets.join("bezel_gbc.png"));
}
