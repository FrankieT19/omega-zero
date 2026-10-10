//! The Network modal and the SSH panel: where flow.html puts their panels, rows and notes, how
//! many networks the list shows and how it scrolls, and the one face the modal is baked as.

use omega_gfx::{out_h, out_w};
use omega_ui::kit::layout::{modal_rect, page, PANEL_CHROME_H};
use omega_ui::kit::modal::modal_face;
use omega_ui::kit::palette::palette;
use omega_ui::kit::panel::{panel_face, title_face};
use omega_ui::kit::row::{row_rect, row_width, ROW_H};
use omega_ui::network::{
    network_chrome, network_panels, network_row_widths, network_title, network_window,
    networks_shown, ssh_chrome, ssh_rect, NetworkFaces, SshFaces, AVAILABLE_TITLE, NOTE_IN,
    NOTE_TOP, SSH_W,
};
use omega_ui::{Draw, TexId};

const AMBER: u8 = 7;

#[test]
fn at_640x480_everything_is_where_flow_puts_it() {
    if (out_w(), out_h()) != (640, 480) {
        return;
    }
    assert_eq!(modal_rect(), [16, 14, 288, 200]);
    // NETWORK hugging three rows, AVAILABLE NETWORKS a pixel over it to the frame
    assert_eq!(network_panels(3), [[4, 2, 280, 62], [4, 63, 280, 133]]);
    assert_eq!(networks_shown(3, 0), 8, "110 pixels of well hold 8 rows");
    assert_eq!(networks_shown(3, 1), 7);
    assert_eq!(networks_shown(3, 2), 6);
    assert_eq!(ssh_rect(5), [60, 57, 200, 104]);
    assert_eq!(ssh_rect(0), [60, 88, 200, 42]);
}

#[test]
fn network_hugs_its_rows_and_the_list_takes_the_rest() {
    let [_, _, w, h] = modal_rect();
    for settings in [1, 3] {
        let [top, list] = network_panels(settings);
        assert_eq!(top, [4, 2, w - 8, PANEL_CHROME_H + ROW_H * settings as u32]);
        assert_eq!(list[1], top[1] + top[3] - 1, "the panels share a stripe");
        assert_eq!(list[1] + list[3], h - 4, "the list runs down to the frame");
        let well = list[3] - PANEL_CHROME_H;
        assert_eq!(networks_shown(settings, 0), (well / ROW_H) as usize);
    }
    assert!(
        networks_shown(1, 0) > networks_shown(3, 0),
        "with Wi-Fi off the list has more room"
    );
}

#[test]
fn notes_take_room_from_the_list_but_leave_a_row() {
    let well = network_panels(3)[1][3] - PANEL_CHROME_H;
    for notes in 0..4 {
        let shown = networks_shown(3, notes) as u32;
        let room = if notes == 0 {
            0
        } else {
            NOTE_TOP + 10 * notes as u32 + 2
        };
        assert!(
            shown * ROW_H + room <= well,
            "{notes} notes overflow the well"
        );
        assert!(
            (shown + 1) * ROW_H + room > well,
            "{notes} notes leave a row of room"
        );
    }
    assert_eq!(networks_shown(3, 99), 1);
}

#[test]
fn the_list_scrolls_a_whole_row_at_a_time() {
    // all in view: no scrolling
    assert_eq!(network_window(4, 5, 0, 8), 0);
    // down past the bottom: one row
    assert_eq!(network_window(8, 20, 0, 8), 1);
    assert_eq!(network_window(9, 20, 1, 8), 2);
    // within view: it stays
    assert_eq!(network_window(5, 20, 2, 8), 2);
    // up past the top: to the cursor
    assert_eq!(network_window(1, 20, 2, 8), 1);
    // round to the last: the end of the list
    assert_eq!(network_window(19, 20, 0, 8), 12);
    // the list shrinking under it: never past its end
    assert_eq!(network_window(3, 10, 6, 8), 2);
}

#[test]
fn the_chrome_is_the_modal_with_both_panels_dim_laid_in_it() {
    let pal = palette(AMBER);
    let chrome = network_chrome(3, pal);
    let [_, _, w, h] = modal_rect();
    assert_eq!((chrome.face.w, chrome.face.h), (w, h));
    let modal = modal_face(w, h, pal);
    let names = ["NETWORK", AVAILABLE_TITLE];
    let px = |f: &omega_ui::UndoFace, x: u32, y: u32| {
        let i = ((y * f.w + x) * 4) as usize;
        f.rgba[i..i + 4].to_vec()
    };
    for (i, [x, y, pw, ph]) in network_panels(3).into_iter().enumerate() {
        let panel = panel_face(pw, ph, names[i], false, pal);
        let [wx, wy, ww, wh] = panel.well;
        assert_eq!(chrome.wells[i], [x + wx, y + wy, ww, wh]);
        let [tx, ty, tw, th] = panel.title;
        assert_eq!(chrome.titles[i], [x + tx, y + ty, tw, th]);
        for py in 0..ph {
            for qx in 0..pw {
                let mine = px(&panel.face, qx, py);
                let want = if mine[3] == 0 {
                    px(&modal, x + qx, y + py)
                } else {
                    mine
                };
                assert_eq!(
                    px(&chrome.face, x + qx, y + py),
                    want,
                    "panel {i} at ({qx}, {py})"
                );
            }
        }
    }
    // the lit title covers the dim one exactly
    for (i, name) in names.iter().enumerate() {
        let lit = network_title(i, pal);
        let dim = title_face(name, false, pal);
        assert_eq!((lit.w, lit.h), (dim.w, dim.h));
        let alpha = |f: &omega_ui::UndoFace| f.rgba.chunks(4).map(|p| p[3]).collect::<Vec<_>>();
        assert_eq!(alpha(&lit), alpha(&dim));
        assert_ne!(lit.rgba, dim.rgba);
    }
}

#[test]
fn every_row_reaches_a_pixel_into_its_well() {
    let chrome = network_chrome(3, palette(0));
    let [top, list] = network_row_widths(&chrome.wells);
    assert_eq!(top, row_width(chrome.wells[0][2]));
    assert_eq!(list, row_width(chrome.wells[1][2]));
    assert_eq!(top, chrome.wells[0][2] + 2);
    let ssh = ssh_chrome(5, palette(0));
    assert_eq!((ssh.face.w, ssh.face.h), (SSH_W, ssh_rect(5)[3]));
}

fn tex(n: usize) -> TexId {
    TexId::from_raw(n)
}

/// Where each face of `out` was drawn, in art pixels, and its texture.
fn placed(out: &[Draw]) -> Vec<(TexId, u32, u32)> {
    out.iter()
        .map(|d| match *d {
            Draw::Tex { x, y, tex, .. } => {
                assert!(
                    x % 2.0 == 0.0 && y % 2.0 == 0.0,
                    "off the art grid at ({x}, {y})"
                );
                (tex, x as u32 / 2, y as u32 / 2)
            }
            other => panic!("not a face: {other:?}"),
        })
        .collect()
}

#[test]
fn the_modal_draws_its_rows_down_its_wells_and_notes_under_the_list() {
    let chrome = network_chrome(3, palette(0));
    let faces = NetworkFaces {
        chrome: (tex(1), chrome.face.w, chrome.face.h),
        wells: chrome.wells,
        title_at: chrome.titles,
        titles: [(tex(2), 1, 1), (tex(3), 1, 1)],
        active: 1,
        settings: (10..13).map(|n| (tex(n), 10, ROW_H)).collect(),
        networks: (20..22).map(|n| (tex(n), 10, ROW_H)).collect(),
        notes: (30..32).map(|n| (tex(n), 10, 11)).collect(),
    };
    let mut out = Vec::new();
    faces.draw(&mut out);
    let [mx, my, _, _] = modal_rect();
    let [tx, ty, _, _] = chrome.titles[1];
    let mut want = vec![(tex(1), mx, my), (tex(3), mx + tx, my + ty)];
    for (i, n) in (10..13).enumerate() {
        let [x, y, _, _] = row_rect(chrome.wells[0], i as u32);
        want.push((tex(n), mx + x, my + y));
    }
    for (i, n) in (20..22).enumerate() {
        let [x, y, _, _] = row_rect(chrome.wells[1], i as u32);
        want.push((tex(n), mx + x, my + y));
    }
    let [wx, wy, _, _] = chrome.wells[1];
    for (i, n) in (30..32).enumerate() {
        let y = wy + 2 * ROW_H + NOTE_TOP + 10 * i as u32;
        want.push((tex(n), mx + wx + NOTE_IN, my + y));
    }
    assert_eq!(placed(&out), want);
}

#[test]
fn ssh_draws_its_lines_then_its_note_in_the_middle_of_the_page() {
    for lines in [5, 0] {
        let panel = ssh_chrome(lines, palette(0));
        let at = ssh_rect(lines);
        let faces = SshFaces {
            chrome: (tex(1), panel.face.w, panel.face.h),
            at,
            well: panel.well,
            lines: (10..10 + lines).map(|n| (tex(n), 10, ROW_H)).collect(),
            note: Some((tex(9), 10, 11)),
        };
        let mut out = Vec::new();
        faces.draw(&mut out);
        let [px, py, w, h] = at;
        let [pw, ph] = page();
        assert_eq!((px, py), ((pw - w) / 2, (ph - h) / 2), "not in the middle");
        let mut want = vec![(tex(1), px, py)];
        for i in 0..lines {
            let [x, y, _, _] = row_rect(panel.well, i as u32);
            want.push((tex(10 + i), px + x, py + y));
        }
        let [wx, wy, _, wh] = panel.well;
        let note_y = wy + lines as u32 * ROW_H + NOTE_TOP;
        want.push((tex(9), px + wx + NOTE_IN, py + note_y));
        assert_eq!(placed(&out), want, "{lines} lines");
        // the note's line ends 2 above the well's bottom under rows, 5 on its own
        let foot = if lines == 0 { 5 } else { 2 };
        assert_eq!(note_y + 10 + foot, wy + wh, "{lines} lines");
    }
}
