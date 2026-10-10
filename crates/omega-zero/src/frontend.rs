use std::time::{Duration, Instant};

use omega_gfx::{out_h, out_w, Bezel, Compositor, Draw, TexId};
use omega_input::{InputSource, Millis};
use omega_power::{Platform, Power};
use omega_store::format_stamp;
use omega_ui::{
    arrows_hint_face, badge_face, bezel_mark_face, bolt_face, cart_face, cart_shadow, chip_face,
    chip_shadow_face, date_time_text, gb_cart_shadow, hhmm, hint_face, icon_face, mark_height,
    mark_rect, menu_face, photo_face, quick_caret_face, quick_label_face, quick_legend_faces,
    quick_value_face, rim, set_clock_hint_face, socket_face, sticker_face, title_face, toast_face,
    wallpaper_face, word_face, GbShell, Icon, LinkBadge, PowerChoice, QuickMenuFaces, QuickRow,
    QuickValue, StickerFields, Toast, UndoFace, ALERT_PX, BEZEL_FILL, HUD_ICON_PX, HUD_INK, LEGEND,
};
use omega_ui::{
    bottom_key_face, bottom_label, field_face, key_glyph_face, key_rect, network_name_face,
    plain_face, prompt_face, signal_face, wifi_status_face, KeyboardFaces, WifiMenuFaces,
    WifiRowFaces, BOTTOM, KEY_ROWS, SHIFT_INK,
};
use omega_ui::{link_face, overlay_clear, overlay_face, overlay_path, Link, LinkFaces};

use crate::app::{App, LinkRow, Phase};
use crate::build_info::Build;
use crate::face_builder::FaceBuilder;
use crate::link_art_builder::LinkArtBuilder;
use crate::link_screen::{LinkSprites, Sprite};
use crate::link_start::{LinkFail, LinkStep};
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue};
use crate::session::Session;
use crate::wallpaper;
use crate::wifi_screen::Summary;

const DOZE_TIMEOUT: Duration = Duration::from_secs(180);

const ALERT_INK: [u8; 3] = [0xf0, 0xb4, 0x3c];

pub struct Frontend {
    session: Session,
    start: Instant,
    last: Instant,
    draws: Vec<Draw>,
    polaroid_texes: Vec<TexId>,
    title_tex: Option<TexId>,
    faces: FaceBuilder,
    link_art: LinkArtBuilder,
    link_art_done: bool,
    core_asked: Option<String>,
    core_board_tex: Option<TexId>,
    core_lid_tex: Option<TexId>,
    core_built: Option<String>,
    undo_tex: Option<TexId>,
    switcher: Switcher,
    clocks: Clocks,
    about: AboutFace,
    quick_clock: QuickClock,
    bezel_marks: Vec<BezelMark>,
    overlays: Vec<Overlay>,
    wifi: WifiTexes,
    labels: WifiTexes,
    background: WifiTexes,
    backdrop: BackdropTex,
}

/// The picture of the pattern the shelf is drawn over, made again when the style or colour
/// changes.
#[derive(Default)]
struct BackdropTex {
    tex: Option<TexId>,
    built: Option<(omega_store::BackdropStyle, u8)>,
}

/// Textures for a list screen, Wi-Fi's or Labels', and for the quick menu's Wi-Fi row and
/// the keyboard, each reused as what it shows changes.
#[derive(Default)]
struct WifiTexes {
    summary: Option<Summary>,
    summary_tex: [Option<TexId>; 2],
    view: Option<ListView>,
    rows: Vec<[Option<TexId>; 3]>,
    status: Vec<Option<TexId>>,
    legend: Vec<(TexId, u32)>,
    carets: Option<[(TexId, u32, u32); 2]>,
    typed: Option<(String, String, bool)>,
    prompt: Option<TexId>,
    field: Option<TexId>,
}

/// A card's own overlay for one system, read once and checked against each layout it is
/// laid over: one that would hide the game is refused rather than drawn.
struct Overlay {
    platform: omega_store::Platform,
    face: Option<UndoFace>,
    tex: Option<TexId>,
    checked: Vec<([i32; 4], bool)>,
}

/// A console's wordmark, uploaded once for each system and height it has been printed at.
struct BezelMark {
    platform: omega_store::Platform,
    height: u32,
    tex: TexId,
    w: u32,
    h: u32,
}

#[derive(Default)]
struct QuickClock {
    dim: Option<TexId>,
    lit: Option<TexId>,
    shown: String,
}

#[derive(Default)]
struct AboutFace {
    tex: Option<TexId>,
    battery: Option<u8>,
}

#[derive(Default)]
struct Clocks {
    line: Option<TexId>,
    hint: Option<TexId>,
    shelf: Option<TexId>,
    picked: Option<String>,
    shown: String,
    battery: String,
    battery_tex: Option<TexId>,
    platform: String,
    platform_tex: Option<TexId>,
}

#[derive(Default)]
struct Switcher {
    open: bool,
    titled: Option<String>,
}

impl Frontend {
    pub fn boot(platform: Box<dyn Platform>) -> Self {
        let now = Instant::now();
        let mut session = Session::boot(platform.root().to_path_buf());
        session
            .app_mut()
            .set_power(Power::new(platform, DOZE_TIMEOUT));
        Frontend {
            session,
            start: now,
            last: now,
            draws: Vec::new(),
            polaroid_texes: Vec::new(),
            title_tex: None,
            faces: FaceBuilder::spawn(),
            link_art: LinkArtBuilder::spawn(),
            link_art_done: false,
            core_asked: None,
            core_board_tex: None,
            core_lid_tex: None,
            core_built: None,
            undo_tex: None,
            switcher: Switcher::default(),
            clocks: Clocks::default(),
            about: AboutFace::default(),
            quick_clock: QuickClock::default(),
            bezel_marks: Vec::new(),
            overlays: Vec::new(),
            wifi: WifiTexes::default(),
            labels: WifiTexes::default(),
            background: WifiTexes::default(),
            backdrop: BackdropTex::default(),
        }
    }

    pub fn upload_faces(&mut self, compositor: &mut Compositor) {
        let faces = self
            .session
            .app()
            .carts()
            .map(|c| {
                let f = cart_face(c);
                compositor.create_texture(f.w, f.h, &f.rgba)
            })
            .collect();
        self.session.app_mut().set_faces(faces);
        let icons = Icon::ALL
            .iter()
            .map(|i| {
                let f = icon_face(*i, HUD_ICON_PX, HUD_INK);
                compositor.create_texture(f.w, f.h, &f.rgba)
            })
            .collect();
        self.session.app_mut().set_icon_faces(icons);
        let link_badges = LinkBadge::FACES
            .iter()
            .map(|b| {
                let (badge, ink) = (
                    b.badge().expect("a face has a glyph"),
                    b.colour().expect("and a colour"),
                );
                let f = badge_face(badge, HUD_ICON_PX, ink);
                compositor.create_texture(f.w, f.h, &f.rgba)
            })
            .collect();
        self.session.app_mut().set_link_badge_faces(link_badges);
        let alert = icon_face(Icon::Alert, ALERT_PX, ALERT_INK);
        let alert = compositor.create_texture(alert.w, alert.h, &alert.rgba);
        self.session.app_mut().set_alert_face(alert);
        let lines = PowerChoice::ALL
            .iter()
            .map(|c| {
                let f = menu_face(match c {
                    PowerChoice::Restart => "Restarting",
                    PowerChoice::PowerOff => "Powering Down",
                });
                (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h)
            })
            .collect();
        self.session.app_mut().set_shutdown_faces(lines);
        let menu = PowerChoice::ALL
            .iter()
            .map(|c| {
                let f = menu_face(c.text());
                (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h)
            })
            .collect();
        self.session.app_mut().set_power_menu_faces(menu);
        let mut up = |f: UndoFace| (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h);
        let labels = QuickRow::ALL
            .iter()
            .map(|r| up(quick_label_face(*r)))
            .collect();
        let values = QuickValue::ALL
            .iter()
            .map(|v| [false, true].map(|lit| up(quick_value_face(v.text(), lit))))
            .collect();
        let carets = [false, true].map(|right| up(quick_caret_face(right)));
        let legend = quick_legend_faces().map(|f| {
            let (tex, w, _) = up(f);
            (tex, w)
        });
        self.session.app_mut().set_quick_menu_faces(QuickMenuFaces {
            labels,
            values,
            carets,
            legend,
        });
        let sockets = omega_store::Core::ALL
            .iter()
            .map(|c| {
                let f = socket_face(*c);
                compositor.create_texture(f.w, f.h, &f.rgba)
            })
            .collect();
        let chips = omega_store::Core::ALL
            .iter()
            .map(|c| {
                let f = chip_face(Some(*c));
                compositor.create_texture(f.w, f.h, &f.rgba)
            })
            .collect();
        let blank = chip_face(None);
        let blank = compositor.create_texture(blank.w, blank.h, &blank.rgba);
        let shadow = chip_shadow_face();
        let shadow = compositor.create_texture(shadow.w, shadow.h, &shadow.rgba);
        self.session
            .app_mut()
            .set_core_part_faces(sockets, chips, blank, shadow);
        let legend = [
            hint_face("B", "Cancel"),
            arrows_hint_face("Swap"),
            hint_face("A", "Choose"),
        ]
        .into_iter()
        .map(|f| (compositor.create_texture(f.w, f.h, &f.rgba), f.w))
        .collect();
        self.session.app_mut().set_core_legend_faces(legend);
        let roles = menu_faces(compositor, LinkRow::ALL.iter().map(|r| r.text()));
        self.session.app_mut().set_link_menu_faces(roles);
        if let Some(linked) = menu_faces(compositor, ["Linked"].into_iter()).pop() {
            self.session.app_mut().set_link_linked_face(linked);
        }
        let legend = [
            hint_face("B", "Cancel"),
            hint_face("SELECT", "Mode"),
            arrows_hint_face("Swap"),
            hint_face("A", "Link"),
            hint_face("A", "OK"),
            hint_face("B", "Back"),
            hint_face("A", "End Link"),
        ]
        .into_iter()
        .map(|f| (compositor.create_texture(f.w, f.h, &f.rgba), f.w))
        .collect();
        self.session.app_mut().set_link_legend_faces(legend);
        let steps = menu_faces(compositor, LinkStep::ALL.iter().map(|s| s.line()));
        self.session.app_mut().set_link_step_faces(steps);
        let fails = menu_faces(compositor, LinkFail::SHOWN.iter().map(|f| f.line()));
        self.session.app_mut().set_link_fail_faces(fails);
        let toasts = Toast::ALL
            .iter()
            .map(|t| {
                let f = toast_face(*t);
                compositor.create_texture(f.w, f.h, &f.rgba)
            })
            .collect();
        self.session.app_mut().set_toast_faces(toasts);
        let legend = legend_faces(compositor, &LEGEND);
        self.session.app_mut().set_legend_faces(legend);
        let hint = set_clock_hint_face();
        self.clocks.hint = Some(compositor.create_texture(hint.w, hint.h, &hint.rgba));
        let shadow = cart_shadow();
        let id = compositor.create_texture(shadow.w, shadow.h, &shadow.rgba);
        self.session.app_mut().set_cart_shadow(id);
        let notched = gb_cart_shadow(GbShell::Notched);
        let notched = compositor.create_texture(notched.w, notched.h, &notched.rgba);
        let rounded = gb_cart_shadow(GbShell::Rounded);
        let rounded = compositor.create_texture(rounded.w, rounded.h, &rounded.rgba);
        self.session.app_mut().set_gb_cart_shadows(notched, rounded);
        let bolt = bolt_face();
        let bolt_id = compositor.create_texture(bolt.w, bolt.h, &bolt.rgba);
        self.session.app_mut().set_bolt_face(bolt_id);
        let mut link = |l: Link| {
            [false, true].map(|on| {
                let f = link_face(l, on);
                (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h)
            })
        };
        let links = LinkFaces {
            wifi: link(Link::Wifi),
            bluetooth: link(Link::Bluetooth),
            ssh: link(Link::Ssh),
        };
        self.session.app_mut().set_link_faces(links);
        self.upload_wallpaper(compositor);
    }

    fn upload_wallpaper(&mut self, compositor: &mut Compositor) {
        let app = self.session.app();
        let seed = app.wall_secs().unsigned_abs();
        let Some(rgba) = app
            .root()
            .and_then(|root| wallpaper::pick(root, seed))
            .and_then(|path| wallpaper_face(&path))
        else {
            return;
        };
        let id = compositor.create_texture(out_w(), out_h(), &rgba);
        self.session.app_mut().set_wallpaper(id);
    }

    pub fn render(&mut self, compositor: &mut Compositor, window: (u32, u32)) {
        self.compose(compositor);
        compositor.end_frame(window);
    }

    pub fn compose(&mut self, compositor: &mut Compositor) {
        compositor.set_blue_light(self.session.app().blue_light());
        compositor.set_shake(self.session.app().screen_shake());
        compositor.set_screen_power(self.session.app().screen_power());
        let screen = self.session.app().screen();
        let platform = self.session.app().platform();
        compositor.set_game_view(screen.view);
        compositor.set_still_view(screen.still);
        let picture = crate::video_mode::picture_rect(platform, screen.view);
        let overlay = self.overlay(platform, picture, compositor);
        let bezel = self.bezel(platform, screen.bands, picture, overlay, compositor);
        compositor.set_bezel(bezel);
        compositor.begin_frame();
        if let Some(frame) = self.session.frame() {
            compositor.upload_game(&frame);
            crate::latency::taken();
        }
        sync_clock(self.session.app_mut(), compositor, &mut self.clocks);
        sync_about(self.session.app_mut(), compositor, &mut self.about);
        sync_quick_clock(self.session.app_mut(), compositor, &mut self.quick_clock);
        sync_wifi(self.session.app_mut(), compositor, &mut self.wifi);
        sync_labels(self.session.app_mut(), compositor, &mut self.labels);
        sync_background(
            self.session.app_mut(),
            compositor,
            &mut self.backdrop,
            &mut self.background,
        );
        self.relabel(compositor);
        sync_core_picker(
            self.session.app_mut(),
            compositor,
            &self.faces,
            &mut self.core_asked,
            &mut self.core_board_tex,
            &mut self.core_lid_tex,
            &mut self.core_built,
        );
        if !self.link_art_done {
            if let Some(art) = self.link_art.take() {
                let mut up = |f: &omega_ui::CartFace| Sprite {
                    tex: compositor.create_texture(f.w, f.h, &f.rgba),
                    w: f.w,
                    h: f.h,
                };
                let sprites = LinkSprites {
                    port: up(&art.port),
                    plug_host: up(&art.plug_host),
                    plug_join: up(&art.plug_join),
                    adapter: up(&art.adapter),
                    arcs_right: [
                        up(&art.arcs_right[0]),
                        up(&art.arcs_right[1]),
                        up(&art.arcs_right[2]),
                    ],
                    arcs_left: [
                        up(&art.arcs_left[0]),
                        up(&art.arcs_left[1]),
                        up(&art.arcs_left[2]),
                    ],
                    clicks: up(&art.clicks),
                    arrow_left: up(&art.arrow_left),
                    arrow_right: up(&art.arrow_right),
                };
                self.session.app_mut().set_link_sprites(sprites);
                self.link_art_done = true;
            }
        }
        sync_switcher(
            self.session.app_mut(),
            compositor,
            Faces {
                pool: &mut self.polaroid_texes,
                title: &mut self.title_tex,
                undo: &mut self.undo_tex,
            },
            &mut self.switcher,
        );
        self.draws.clear();
        self.session.app().draw(&mut self.draws);
        compositor.draw_list(&self.draws);
    }

    /// Draws again each cart a label arrived for, on the face it already has, and drops the
    /// core picker's lid when it was made for one of them.
    fn relabel(&mut self, compositor: &mut Compositor) {
        for (cart, tex) in self.session.app_mut().take_stale_faces() {
            let face = cart_face(&cart);
            compositor.update_texture(tex, face.w, face.h, &face.rgba);
            if [&self.core_asked, &self.core_built]
                .iter()
                .any(|s| s.as_deref() == Some(cart.stem.as_str()))
            {
                self.core_asked = None;
                self.core_built = None;
            }
        }
    }

    /// The console's body for the bands the picture leaves, rimmed in black where they meet
    /// it, with its wordmark printed on the deepest band that runs the width of the canvas.
    fn bezel(
        &mut self,
        platform: omega_store::Platform,
        bands: Vec<[f32; 4]>,
        picture: [f32; 4],
        overlay: Option<TexId>,
        compositor: &mut Compositor,
    ) -> Option<Bezel> {
        if overlay.is_some() {
            return Some(Bezel {
                bands,
                fill: BEZEL_FILL,
                rims: Vec::new(),
                marks: Vec::new(),
                overlay,
            });
        }
        if bands.is_empty() {
            return None;
        }
        let rims = bands.iter().filter_map(|b| rim(*b, picture).1).collect();
        let printed = bands
            .iter()
            .copied()
            .filter(|b| b[2] >= out_w() as f32)
            .max_by(|a, b| a[3].total_cmp(&b[3]))
            .map(|b| rim(b, picture).0);
        let marks = printed
            .and_then(|band| {
                let m = self.bezel_mark(platform, mark_height(band[3]), compositor)?;
                Some((m.tex, mark_rect(band, m.w, m.h)))
            })
            .into_iter()
            .collect();
        Some(Bezel {
            bands,
            fill: BEZEL_FILL,
            rims,
            marks,
            overlay: None,
        })
    }

    fn overlay(
        &mut self,
        platform: omega_store::Platform,
        picture: [f32; 4],
        compositor: &mut Compositor,
    ) -> Option<TexId> {
        let at = match self.overlays.iter().position(|o| o.platform == platform) {
            Some(at) => at,
            None => {
                let face = self
                    .session
                    .app()
                    .root()
                    .and_then(|root| overlay_face(root, platform));
                self.overlays.push(Overlay {
                    platform,
                    face,
                    tex: None,
                    checked: Vec::new(),
                });
                self.overlays.len() - 1
            }
        };
        let root = self.session.app().root().map(|r| r.to_path_buf());
        let o = &mut self.overlays[at];
        let face = o.face.as_ref()?;
        let key = picture.map(|v| v.round() as i32);
        let clear = match o.checked.iter().find(|(k, _)| *k == key) {
            Some((_, clear)) => *clear,
            None => {
                let clear = overlay_clear(&face.rgba, face.w, face.h, picture);
                if !clear {
                    let path = root.map(|r| overlay_path(&r, platform));
                    eprintln!(
                        "omega-zero: overlay: {} covers the game at {picture:?}, so it is not drawn",
                        path.map_or_else(String::new, |p| p.display().to_string())
                    );
                }
                o.checked.push((key, clear));
                clear
            }
        };
        if !clear {
            return None;
        }
        if o.tex.is_none() {
            o.tex = Some(compositor.create_texture(face.w, face.h, &face.rgba));
        }
        o.tex
    }

    fn bezel_mark(
        &mut self,
        platform: omega_store::Platform,
        height: u32,
        compositor: &mut Compositor,
    ) -> Option<&BezelMark> {
        let known = self
            .bezel_marks
            .iter()
            .position(|m| m.platform == platform && m.height == height);
        let at = match known {
            Some(at) => at,
            None => {
                let face = bezel_mark_face(platform, height)?;
                self.bezel_marks.push(BezelMark {
                    platform,
                    height,
                    tex: compositor.create_texture(face.w, face.h, &face.rgba),
                    w: face.w,
                    h: face.h,
                });
                self.bezel_marks.len() - 1
            }
        };
        self.bezel_marks.get(at)
    }

    pub fn drive_emulator(&mut self) {
        self.session.set_driven(true);
    }

    pub fn step_emulator(&self, present: Duration, timeout: Duration) -> bool {
        self.session.step_emulator(present, timeout)
    }

    pub fn advance(&mut self, input: &mut dyn InputSource) {
        let now = self.now();
        let events = input.poll(now);
        self.session.feed(events, now);
        let dt = self.last.elapsed().as_secs_f32();
        self.last = Instant::now();
        self.session.update(dt);
    }

    pub fn app(&self) -> &crate::app::App {
        self.session.app()
    }

    pub fn app_mut(&mut self) -> &mut crate::app::App {
        self.session.app_mut()
    }

    pub fn core_settling(&self) -> bool {
        self.session.core_settling()
    }

    /// Whether the worker threads have handed back all they were asked for: the link art, and
    /// the core picker's faces for the cart the shelf is on. compose takes them in.
    pub fn settled(&self) -> bool {
        self.link_art_done && self.core_asked == self.core_built && !self.core_settling()
    }

    pub fn advance_at(&mut self, input: &mut dyn InputSource, now: Millis, dt: f32) {
        let events = input.poll(now);
        self.session.feed(events, now);
        self.session.update(dt);
    }

    fn now(&self) -> Millis {
        self.start.elapsed().as_millis() as Millis
    }

    pub fn powering_off(&self) -> bool {
        self.session.app().ready_to_power_off()
    }

    pub fn restarting(&self) -> bool {
        self.session.app().ready_to_restart()
    }

    pub fn restart(&mut self) {
        self.session.app_mut().restart();
    }

    pub fn poweroff(&mut self) {
        self.session.app_mut().poweroff();
    }
}

fn menu_faces<'a>(
    compositor: &mut Compositor,
    labels: impl Iterator<Item = &'a str>,
) -> Vec<(TexId, u32, u32)> {
    labels
        .map(|label| {
            let f = menu_face(label);
            (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h)
        })
        .collect()
}

fn legend_faces(compositor: &mut Compositor, legend: &[(&str, &str)]) -> Vec<TexId> {
    legend
        .iter()
        .map(|(key, label)| {
            let f = hint_face(key, label);
            compositor.create_texture(f.w, f.h, &f.rgba)
        })
        .collect()
}

struct Faces<'a> {
    pool: &'a mut Vec<TexId>,
    title: &'a mut Option<TexId>,
    undo: &'a mut Option<TexId>,
}

fn sync_switcher(app: &mut App, compositor: &mut Compositor, texes: Faces, state: &mut Switcher) {
    if !matches!(app.phase(), Phase::Polaroids { .. }) {
        state.open = false;
        return;
    }
    if !state.open {
        state.open = true;
        state.titled = None;
        let faces: Vec<_> = app.polaroid_entries().iter().map(photo_face).collect();
        let ids = faces
            .iter()
            .enumerate()
            .map(|(i, f)| match texes.pool.get(i) {
                Some(id) => {
                    compositor.update_texture(*id, f.w, f.h, &f.rgba);
                    *id
                }
                None => {
                    let id = compositor.create_texture_nearest(f.w, f.h, &f.rgba);
                    texes.pool.push(id);
                    id
                }
            })
            .collect();
        app.set_polaroid_faces(ids);

        let label = app
            .undo_label()
            .map(|l| upload(compositor, texes.undo, hint_face("X", l)));
        app.set_undo_face(label);
    }
    if state.titled.as_deref() != app.polaroid_stamp() {
        state.titled = app.polaroid_stamp().map(str::to_string);
        let face = title_face(&app.polaroid_title(&format_stamp(app.wall_secs())));
        let id = upload(compositor, texes.title, face);
        app.set_polaroid_title_face(id);
    }
}

fn sync_clock(app: &mut App, compositor: &mut Compositor, clocks: &mut Clocks) {
    let picked = app.picker().map(|p| p.text());
    if picked != clocks.picked {
        clocks.picked = picked;
        if let (Some(face), Some(hint)) = (app.picker().map(|p| p.face()), clocks.hint) {
            let line = upload(compositor, &mut clocks.line, face);
            app.set_clock_faces(line, hint);
        }
    }
    let shown = hhmm(app.wall_secs());
    if shown != clocks.shown {
        let face = word_face(&shown);
        clocks.shown = shown;
        let w = face.w;
        let id = upload(compositor, &mut clocks.shelf, face);
        app.set_shelf_clock_face(id, w);
    }
    let platform_shown = app.slot_text().unwrap_or_default();
    if platform_shown != clocks.platform {
        clocks.platform = platform_shown.clone();
        if platform_shown.is_empty() {
            app.clear_shelf_platform();
        } else {
            let face = word_face(&platform_shown);
            let w = face.w;
            let id = upload(compositor, &mut clocks.platform_tex, face);
            app.set_shelf_platform_face(id, w);
        }
    }
    let battery_shown = app
        .battery()
        .map(|b| format!("{}%", b.percent))
        .unwrap_or_default();
    if battery_shown != clocks.battery {
        clocks.battery = battery_shown.clone();
        if !battery_shown.is_empty() {
            let face = word_face(&battery_shown);
            let w = face.w;
            let id = upload(compositor, &mut clocks.battery_tex, face);
            app.set_battery_percent_face(id, w);
        }
    }
}

fn sync_quick_clock(app: &mut App, compositor: &mut Compositor, state: &mut QuickClock) {
    if app.quick_menu().is_none() {
        return;
    }
    let text = date_time_text(app.wall_secs());
    if text == state.shown {
        return;
    }
    let (dim, lit) = (
        quick_value_face(&text, false),
        quick_value_face(&text, true),
    );
    let (dim_size, lit_size) = ((dim.w, dim.h), (lit.w, lit.h));
    let dim = upload(compositor, &mut state.dim, dim);
    let lit = upload(compositor, &mut state.lit, lit);
    app.set_quick_clock_faces((dim, dim_size.0, dim_size.1), (lit, lit_size.0, lit_size.1));
    state.shown = text;
}

fn sync_wifi(app: &mut App, compositor: &mut Compositor, state: &mut WifiTexes) {
    if app.quick_menu().is_some() {
        let summary = app.wifi_summary();
        if state.summary.as_ref() != Some(&summary) {
            let face = |lit: bool| match &summary {
                Summary::Off => quick_value_face("Off", lit),
                Summary::On => quick_value_face("On", lit),
                Summary::Joined(name) => {
                    plain_face(name, out_w() as f32 * 0.45, 27.0, omega_ui::MENU_H, lit)
                }
            };
            let [dim, lit] = [false, true].map(face);
            let (dim_size, lit_size) = ((dim.w, dim.h), (lit.w, lit.h));
            let dim = upload(compositor, &mut state.summary_tex[0], dim);
            let lit = upload(compositor, &mut state.summary_tex[1], lit);
            app.set_quick_wifi_faces((dim, dim_size.0, dim_size.1), (lit, lit_size.0, lit_size.1));
            state.summary = Some(summary);
        }
    }
    if let Some(view) = app.wifi_view() {
        if state.view.as_ref() != Some(&view) {
            let faces = wifi_faces(&view, compositor, state);
            app.set_wifi_faces(faces);
            state.view = Some(view);
        }
    } else {
        state.view = None;
    }
    let Some(typing) = app.typing() else {
        state.typed = None;
        return;
    };
    let typed = (typing.prompt, typing.keys.text.clone(), typing.refused);
    if !app.keyboard_faces_ready() {
        let faces = keyboard_faces(compositor);
        app.set_keyboard_faces(faces);
    }
    if state.typed.as_ref() != Some(&typed) {
        let prompt = prompt_face(&typed.0);
        let field = field_face(&typed.1);
        let (p, f) = ((prompt.w, prompt.h), (field.w, field.h));
        let prompt = upload(compositor, &mut state.prompt, prompt);
        let field = upload(compositor, &mut state.field, field);
        app.set_typing_faces((prompt, p.0, p.1), (field, f.0, f.1));
        state.typed = Some(typed);
    }
}

fn sync_background(
    app: &mut App,
    compositor: &mut Compositor,
    pattern: &mut BackdropTex,
    list: &mut WifiTexes,
) {
    let want = app.backdrop_pattern();
    if pattern.built != want {
        if let Some((style, colour)) = want {
            if let Some(face) = omega_ui::backdrop_pattern_face(style, colour) {
                let w = face.w;
                let tex = upload(compositor, &mut pattern.tex, face);
                app.set_backdrop_face(tex, w, style, colour);
            }
        }
        pattern.built = want;
    }
    let Some(view) = app.background_view() else {
        list.view = None;
        return;
    };
    if list.view.as_ref() != Some(&view) {
        let faces = wifi_faces(&view, compositor, list);
        app.set_background_faces(faces);
        list.view = Some(view);
    }
}

fn sync_labels(app: &mut App, compositor: &mut Compositor, state: &mut WifiTexes) {
    let Some(view) = app.labels_view() else {
        state.view = None;
        return;
    };
    if state.view.as_ref() != Some(&view) {
        let faces = wifi_faces(&view, compositor, state);
        app.set_labels_faces(faces);
        state.view = Some(view);
    }
}

fn wifi_faces(
    view: &ListView,
    compositor: &mut Compositor,
    state: &mut WifiTexes,
) -> WifiMenuFaces {
    if state.legend.is_empty() {
        state.legend = LegendHint::ALL
            .iter()
            .map(|h| {
                let f = match h {
                    LegendHint::Back => hint_face("B", "Back"),
                    LegendHint::Change => arrows_hint_face("Change"),
                    LegendHint::Join => hint_face("A", "Join"),
                    LegendHint::Forget => hint_face("Y", "Forget"),
                    LegendHint::SignIn => hint_face("A", "Sign in"),
                    LegendHint::SignOut => hint_face("Y", "Sign out"),
                    LegendHint::Start => hint_face("A", "Start"),
                    LegendHint::Stop => hint_face("A", "Stop"),
                    LegendHint::Open => hint_face("A", "Open"),
                };
                (compositor.create_texture(f.w, f.h, &f.rgba), f.w)
            })
            .collect();
    }
    let carets = *state.carets.get_or_insert_with(|| {
        [false, true].map(|right| {
            let f = quick_caret_face(right);
            (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h)
        })
    });
    if state.rows.len() < view.rows.len() {
        state.rows.resize(view.rows.len(), [None; 3]);
    }
    let rows = view
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let lit = view.lit == Some(i);
            let held = &mut state.rows[i];
            let label = match &row.label {
                RowLabel::Setting(name) => quick_value_face(name, true),
                RowLabel::Name(name) => network_name_face(name, true),
            };
            let label = sized(compositor, &mut held[0], label);
            let value = row.value.as_ref().map(|v| {
                let face = match v {
                    RowValue::Setting(true) => quick_value_face("On", lit),
                    RowValue::Setting(false) => quick_value_face("Off", lit),
                    RowValue::Said(text) | RowValue::Choice(text) => quick_value_face(text, lit),
                    RowValue::Name(name) => {
                        plain_face(name, out_w() as f32 * 0.45, 27.0, omega_ui::MENU_H, lit)
                    }
                };
                sized(compositor, &mut held[1], face)
            });
            let icon = row.signal.map(|(bars, locked)| {
                sized(compositor, &mut held[2], signal_face(bars, locked, lit))
            });
            WifiRowFaces {
                label,
                value,
                icon,
                setting: matches!(row.value, Some(RowValue::Setting(_) | RowValue::Choice(_))),
            }
        })
        .collect();
    if state.status.len() < view.status.len() {
        state.status.resize(view.status.len(), None);
    }
    let status = view
        .status
        .iter()
        .enumerate()
        .map(|(i, line)| sized(compositor, &mut state.status[i], wifi_status_face(line)))
        .collect();
    WifiMenuFaces {
        rows,
        lit: view.lit,
        status,
        legend: view
            .legend
            .iter()
            .map(|h| state.legend[h.index()])
            .collect(),
        carets,
    }
}

fn keyboard_faces(compositor: &mut Compositor) -> KeyboardFaces {
    let mut up = |f: UndoFace| (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h);
    let glyphs = (' '..='~').map(|c| up(key_glyph_face(c))).collect();
    let mut bottom = Vec::new();
    for (col, key) in BOTTOM.iter().enumerate() {
        let w = key_rect(KEY_ROWS - 1, col)[2] as u32;
        for layer in [omega_ui::Layer::Lower, omega_ui::Layer::Symbols] {
            let label = bottom_label(*key, layer);
            if !bottom.iter().any(|(l, _)| *l == label) {
                bottom.push((label, up(bottom_key_face(label, w, omega_ui::MENU_INK))));
            }
        }
    }
    let shift_w = key_rect(KEY_ROWS - 1, 0)[2] as u32;
    let shift_on = up(bottom_key_face("Shift", shift_w, SHIFT_INK));
    let legend = [
        hint_face("B", "Delete"),
        hint_face("X", "Shift"),
        hint_face("Y", "Space"),
        hint_face("START", "Done"),
    ]
    .into_iter()
    .map(|f| (compositor.create_texture(f.w, f.h, &f.rgba), f.w))
    .collect();
    KeyboardFaces {
        glyphs,
        bottom,
        shift_on,
        legend,
    }
}

fn sized(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    face: UndoFace,
) -> (TexId, u32, u32) {
    let (w, h) = (face.w, face.h);
    (upload(compositor, held, face), w, h)
}

fn sync_about(app: &mut App, compositor: &mut Compositor, state: &mut AboutFace) {
    if !matches!(app.phase(), Phase::About) {
        return;
    }
    let battery = app.battery().map(|b| b.percent);
    if state.tex.is_some() && state.battery == battery {
        return;
    }
    state.battery = battery;
    let build = Build::current();
    let face = sticker_face(&StickerFields {
        battery,
        serial: &build.serial(),
        dirty_digit: build.dirty_digit(),
    });
    let id = upload(compositor, &mut state.tex, face);
    app.set_sticker_face(id);
}

fn sync_core_picker(
    app: &mut App,
    compositor: &mut Compositor,
    builder: &FaceBuilder,
    asked: &mut Option<String>,
    board: &mut Option<TexId>,
    lid: &mut Option<TexId>,
    built: &mut Option<String>,
) {
    let highlighted = app.selected_stem().map(str::to_string);
    if highlighted.is_some() && *asked != highlighted {
        if let Some(cart) = app
            .carts()
            .find(|c| highlighted.as_deref() == Some(c.stem.as_str()))
        {
            builder.request(cart.clone());
        }
        *asked = highlighted.clone();
    }
    let Some(faces) = builder.take() else {
        return;
    };
    if highlighted.as_deref() != Some(faces.stem.as_str()) || *built == highlighted {
        return;
    }
    let board_id = upload_rgba(
        compositor,
        board,
        faces.board.w,
        faces.board.h,
        &faces.board.rgba,
    );
    let lid_id = upload_rgba(compositor, lid, faces.lid.w, faces.lid.h, &faces.lid.rgba);
    app.set_core_board_faces(board_id, lid_id);
    *built = Some(faces.stem);
}

fn upload(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    face: omega_ui::UndoFace,
) -> TexId {
    upload_rgba(compositor, held, face.w, face.h, &face.rgba)
}

fn upload_rgba(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    w: u32,
    h: u32,
    rgba: &[u8],
) -> TexId {
    match *held {
        Some(id) => {
            compositor.update_texture(id, w, h, rgba);
            id
        }
        None => {
            let id = compositor.create_texture(w, h, rgba);
            *held = Some(id);
            id
        }
    }
}
