use std::time::{Duration, Instant};

use omega_gfx::{out_h, out_w, Bezel, Compositor, Draw, TexId};
use omega_input::{InputSource, Millis};
use omega_power::{Platform, Power};
use omega_ui::{
    arrows_hint_face, badge_face, bezel_mark_face, bolt_face, cart_face, cart_shadow, chip_face,
    chip_shadow_face, gb_cart_shadow, hint_face, icon_face, mark_height, mark_rect, menu_face, rim,
    set_clock_hint_face, socket_face, toast_face, wallpaper_face, GbShell, Icon, LinkBadge,
    PowerChoice, Toast, UndoFace, ALERT_PX, BEZEL_FILL, HUD_ICON_PX, HUD_INK, LEGEND,
};
use omega_ui::{link_face, overlay_clear, overlay_face, overlay_path, Link, LinkFaces};

use crate::app::LinkRow;
use crate::face_builder::FaceBuilder;
use crate::link_art_builder::LinkArtBuilder;
use crate::link_screen::{LinkSprites, Sprite};
use crate::link_start::{LinkFail, LinkStep};
use crate::session::Session;
use crate::wallpaper;

mod faces;
mod sync_about;
mod sync_background;
mod sync_clock;
mod sync_controls;
mod sync_core_picker;
mod sync_keyboard;
mod sync_labels;
mod sync_network;
mod sync_settings;
mod sync_ssh;
mod sync_switcher;

use faces::WifiTexes;
use sync_about::{sync_about, AboutFace};
use sync_background::{sync_background, BackdropTex};
use sync_clock::{sync_clock, Clocks};
use sync_controls::sync_controls;
use sync_core_picker::sync_core_picker;
use sync_keyboard::sync_keyboard;
use sync_labels::sync_labels;
use sync_network::{sync_network, NetworkTexes};
use sync_settings::{sync_settings, SettingsTexes};
use sync_ssh::{sync_ssh, SshTexes};
use sync_switcher::{sync_switcher, Faces, Switcher};

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
    settings: SettingsTexes,
    bezel_marks: Vec<BezelMark>,
    overlays: Vec<Overlay>,
    /// The Network screen's list and the keyboard.
    network: WifiTexes,
    /// The Network modal's own faces.
    network_modal: NetworkTexes,
    ssh: SshTexes,
    labels: WifiTexes,
    background: WifiTexes,
    backdrop: BackdropTex,
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
            settings: SettingsTexes::default(),
            bezel_marks: Vec::new(),
            overlays: Vec::new(),
            network: WifiTexes::default(),
            network_modal: NetworkTexes::default(),
            ssh: SshTexes::default(),
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
        sync_settings(self.session.app_mut(), compositor, &mut self.settings);
        sync_network(self.session.app_mut(), compositor, &mut self.network_modal);
        sync_keyboard(self.session.app_mut(), compositor, &mut self.network);
        sync_ssh(self.session.app_mut(), compositor, &mut self.ssh);
        sync_controls(self.session.app_mut(), compositor);
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
