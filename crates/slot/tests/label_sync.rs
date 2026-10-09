use std::collections::VecDeque;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use slot::json::Json;
use slot::label_sync::{
    crc32, game_url, media_url, pick_label, read_reply, regions, sync, title, Account, Http,
    LabelNews, Lookup, Media, Quota, Reply, Sync, SyncFail, Tally, Want, Who,
};
use slot_store::Platform;

#[test]
fn json_reads_what_screenscraper_sends_and_forgives_a_stray_backslash() {
    let j = Json::parse(
        r#"{"response": {"jeu": {"id": "3767", "noms": [{"text": "Café 🎮"}],
            "synopsis": "Samus\'s ship \n lands", "ok": true, "none": null, "n": -1.5e2}}}"#,
    )
    .expect("did not parse");
    let jeu = j.get("response").and_then(|r| r.get("jeu")).unwrap();
    assert_eq!(jeu.get("id").and_then(Json::count), Some(3767));
    assert_eq!(
        jeu.get("noms").unwrap().list()[0]
            .get("text")
            .and_then(Json::text),
        Some("Café 🎮")
    );
    assert_eq!(
        jeu.get("synopsis").and_then(Json::text),
        Some("Samus\\'s ship \n lands"),
        "a backslash JSON has no escape for is kept"
    );
    assert_eq!(jeu.get("ok"), Some(&Json::Bool(true)));
    assert_eq!(jeu.get("none"), Some(&Json::Null));
    assert_eq!(jeu.get("n"), Some(&Json::Number(-150.0)));
    assert!(Json::parse(r#"{"a": 1"#).is_none(), "an unclosed object");
    assert!(Json::parse(r#"{"a": 1} x"#).is_none(), "trailing text");
}

#[test]
fn a_cart_is_known_by_its_crc32() {
    assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    assert_eq!(crc32(b""), 0);
}

#[test]
fn a_label_is_taken_from_the_roms_own_region_first() {
    assert_eq!(
        regions("Metroid Fusion (USA)", "AMTE"),
        ["us", "wor", "eu", "jp"]
    );
    assert_eq!(
        regions("Golden Sun (USA, Europe)", "AGSE"),
        ["us", "eu", "wor", "jp"]
    );
    assert_eq!(
        regions(
            "Kirby & The Amazing Mirror (Europe) (En,Fr,De,Es,It)",
            "B8KP"
        ),
        ["eu", "wor", "us", "jp"],
        "language tags are not regions"
    );
    assert_eq!(
        regions("Mother 3 v1.2", "A3UJ"),
        ["jp", "wor", "us", "eu"],
        "a hack's name has no tags, but its game code does"
    );
    assert_eq!(regions("Tetris", ""), ["wor", "us", "eu", "jp"]);
}

#[test]
fn a_lookup_carries_both_credentials_and_the_rom_escaped() {
    let account = Account {
        user: "middle baws".into(),
        password: "p&ss=word".into(),
    };
    let url = game_url(
        ("devid", "devpw"),
        &account,
        &Lookup {
            system: 12,
            crc: 0xabc,
            file_name: "Mario & Luigi (Europe).gba",
            size: 16_777_216,
        },
    );
    assert!(url.starts_with("https://api.screenscraper.fr/api2/jeuInfos.php?devid=devid&"));
    for part in [
        "devpassword=devpw",
        "softname=slot-rg35xxsp",
        "output=json",
        "ssid=middle%20baws",
        "sspassword=p%26ss%3Dword",
        "crc=00000ABC",
        "systemeid=12",
        "romtype=rom",
        "romnom=Mario%20%26%20Luigi%20%28Europe%29.gba",
        "romtaille=16777216",
    ] {
        assert!(url.contains(part), "{part} is missing from {url}");
    }
    assert!(media_url("https://x/mediaJeu.php?a=1").ends_with("&maxwidth=600&outputformat=png"));
}

const GAME: &str = r#"{"header": {"success": "true"}, "response": {
    "ssuser": {"id": "middlebaws", "maxrequestsperday": "20000", "requeststoday": "47"},
    "jeu": {"id": "3767", "medias": [
        {"type": "box-2D", "parent": "jeu", "region": "us", "url": "https://ss/box-us", "format": "png"},
        {"type": "support-texture", "parent": "jeu", "region": "jp", "url": "https://ss/tex-jp", "format": "png"},
        {"type": "support-texture", "parent": "jeu", "region": "us", "url": "https://ss/tex-us", "format": "png"},
        {"type": "wheel", "parent": "jeu", "region": "wor", "url": "https://ss/wheel", "format": "png"}
    ]}}}"#;

#[test]
fn a_reply_names_the_game_its_media_and_the_lookups_left_today() {
    let (reply, quota) = read_reply(200, GAME.as_bytes());
    let Reply::Game(medias) = reply else {
        panic!("{reply:?}");
    };
    assert_eq!(medias.len(), 4);
    assert_eq!(
        quota,
        Some(Quota {
            left: 19_953,
            per_day: 20_000
        })
    );
    let picked = pick_label(&medias, &regions("Metroid Fusion (USA)", "AMTE")).unwrap();
    assert_eq!(picked.url, "https://ss/tex-us");
    let picked = pick_label(&medias, &["eu", "wor"]).unwrap();
    assert_eq!(picked.url, "https://ss/tex-jp", "any region beats none");
    let boxes: Vec<Media> = medias
        .into_iter()
        .filter(|m| m.kind != "support-texture")
        .collect();
    assert_eq!(pick_label(&boxes, &["us"]), None, "a box is not a label");
}

#[test]
fn a_refusal_says_whose_login_was_wrong() {
    let dev = "Erreur de login : Vérifier vos identifiants développeur !";
    assert_eq!(
        read_reply(403, dev.as_bytes()).0,
        Reply::Refused(Who::Program)
    );
    assert_eq!(
        read_reply(200, dev.as_bytes()).0,
        Reply::Refused(Who::Program),
        "a refusal can come back as a 200"
    );
    let player = "Erreur de login : Vérifier vos identifiants !";
    assert_eq!(
        read_reply(403, player.as_bytes()).0,
        Reply::Refused(Who::Player)
    );
    assert_eq!(
        read_reply(404, b"Erreur : Rom non trouvee").0,
        Reply::NotFound
    );
    assert_eq!(read_reply(431, b"").0, Reply::NotFound);
    assert_eq!(read_reply(429, b"").0, Reply::Busy);
    assert_eq!(read_reply(430, b"").0, Reply::Spent);
    for down in [401, 423, 426] {
        assert_eq!(read_reply(down, b"").0, Reply::Down);
    }
    assert_eq!(read_reply(200, br#"{"response": {}}"#).0, Reply::NotFound);
    assert_eq!(read_reply(200, b"<html>").0, Reply::Other(200));
}

#[test]
fn an_account_is_kept_in_config_and_can_be_forgotten() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(Account::load(root.path()), None);
    let account = Account {
        user: "middlebaws".into(),
        password: "pa=ss word".into(),
    };
    account.save(root.path()).unwrap();
    assert_eq!(Account::load(root.path()), Some(account));
    Account::forget(root.path());
    assert_eq!(Account::load(root.path()), None);
}

#[test]
fn a_title_drops_the_tags() {
    assert_eq!(title("Metroid Fusion (USA)"), "Metroid Fusion");
    assert_eq!(title("Mother 3 v1.2"), "Mother 3 v1.2");
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().unwrap();
        wr.write_image_data(&vec![0x80; (w * h * 3) as usize])
            .unwrap();
    }
    out
}

/// Answers each GET from a queue, and remembers what it was asked.
struct Server {
    replies: VecDeque<Option<(u16, Vec<u8>)>>,
    asked: Vec<String>,
}

impl Server {
    fn new(replies: Vec<Option<(u16, Vec<u8>)>>) -> Server {
        Server {
            replies: replies.into(),
            asked: Vec::new(),
        }
    }
}

impl Http for Server {
    fn get(&mut self, url: &str) -> Option<(u16, Vec<u8>)> {
        self.asked.push(url.to_string());
        self.replies.pop_front().expect("asked more than expected")
    }
}

fn want(root: &Path, stem: &str) -> Want {
    let rom = root.join("Games/GBA").join(format!("{stem}.gba"));
    std::fs::create_dir_all(rom.parent().unwrap()).unwrap();
    std::fs::write(&rom, stem.as_bytes()).unwrap();
    Want {
        platform: Platform::Gba,
        stem: stem.into(),
        rom,
        code: String::new(),
    }
}

fn run(root: &Path, wants: &[Want], server: &mut Server) -> (Vec<LabelNews>, Vec<Duration>) {
    let account = Account {
        user: "u".into(),
        password: "p".into(),
    };
    let job = Sync {
        key: ("id", "pw"),
        account: &account,
        root,
        wants,
    };
    let (mut news, mut pauses) = (Vec::new(), Vec::new());
    sync(
        &job,
        server,
        &AtomicBool::new(false),
        &mut |d| pauses.push(d),
        &mut |n| news.push(n),
    );
    (news, pauses)
}

fn ok(body: &str) -> Option<(u16, Vec<u8>)> {
    Some((200, body.as_bytes().to_vec()))
}

#[test]
fn a_found_label_is_saved_where_the_shelf_looks_and_the_rest_are_counted() {
    let root = tempfile::tempdir().unwrap();
    let wants = [
        want(root.path(), "Metroid Fusion (USA)"),
        want(root.path(), "Box Only (USA)"),
        want(root.path(), "Mother 3 v1.2"),
    ];
    let bare = r#"{"response": {"jeu": {"medias": [
        {"type": "box-2D", "region": "us", "url": "https://ss/box"}]}}}"#;
    let mut server = Server::new(vec![
        ok(GAME),
        Some((200, png(600, 310))),
        ok(bare),
        Some((404, b"Erreur : Rom/Iso/Dossier non trouvee !".to_vec())),
    ]);
    let (news, _) = run(root.path(), &wants, &mut server);
    let label = root.path().join("Labels/GBA/Metroid Fusion (USA).png");
    assert_eq!(std::fs::read(&label).unwrap(), png(600, 310));
    assert!(server.asked[0].contains(&format!("crc={:08X}", crc32(b"Metroid Fusion (USA)"))));
    assert!(server.asked[0].contains("romtaille=20"));
    assert_eq!(
        server.asked[1],
        "https://ss/tex-us&maxwidth=600&outputformat=png"
    );
    assert!(news.contains(&LabelNews::Saved {
        platform: Platform::Gba,
        stem: "Metroid Fusion (USA)".into()
    }));
    assert!(news.contains(&LabelNews::Working {
        done: 2,
        total: 3,
        title: "Mother 3 v1.2".into()
    }));
    assert_eq!(
        news.last(),
        Some(&LabelNews::Finished(Tally {
            saved: 1,
            bare: 1,
            unknown: 1
        }))
    );
    assert!(!root.path().join("Labels/GBA/Box Only (USA).png").exists());
}

#[test]
fn a_scan_that_is_not_a_picture_is_not_saved() {
    let root = tempfile::tempdir().unwrap();
    let wants = [want(root.path(), "Metroid Fusion (USA)")];
    let mut server = Server::new(vec![ok(GAME), ok("NOMEDIA")]);
    let (news, _) = run(root.path(), &wants, &mut server);
    assert!(!root
        .path()
        .join("Labels/GBA/Metroid Fusion (USA).png")
        .exists());
    assert_eq!(
        news.last(),
        Some(&LabelNews::Finished(Tally {
            bare: 1,
            ..Tally::default()
        }))
    );
}

#[test]
fn a_busy_server_is_asked_once_more_after_a_pause() {
    let root = tempfile::tempdir().unwrap();
    let wants = [want(root.path(), "Metroid Fusion (USA)")];
    let mut server = Server::new(vec![
        Some((429, Vec::new())),
        ok(GAME),
        Some((200, png(4, 2))),
    ]);
    let (news, pauses) = run(root.path(), &wants, &mut server);
    assert_eq!(pauses, [Duration::from_secs(2)]);
    assert!(matches!(news.last(), Some(LabelNews::Finished(t)) if t.saved == 1));
}

#[test]
fn a_refused_login_or_a_spent_quota_ends_the_sync_at_once() {
    for (reply, fail) in [
        (
            (403, "Erreur de login : Vérifier vos identifiants !"),
            SyncFail::Refused(Who::Player),
        ),
        ((430, ""), SyncFail::Spent),
        ((423, ""), SyncFail::Down),
    ] {
        let root = tempfile::tempdir().unwrap();
        let wants = [want(root.path(), "A (USA)"), want(root.path(), "B (USA)")];
        let mut server = Server::new(vec![Some((reply.0, reply.1.as_bytes().to_vec()))]);
        let (news, _) = run(root.path(), &wants, &mut server);
        assert_eq!(news.last(), Some(&LabelNews::Failed(fail)));
        assert_eq!(
            server.asked.len(),
            1,
            "{fail:?}: the next cart was still asked about"
        );
    }
}

#[test]
fn no_connection_ends_the_sync_as_offline() {
    let root = tempfile::tempdir().unwrap();
    let wants = [want(root.path(), "A (USA)")];
    let mut server = Server::new(vec![None]);
    let (news, _) = run(root.path(), &wants, &mut server);
    assert_eq!(news.last(), Some(&LabelNews::Failed(SyncFail::Offline)));
}

#[test]
fn the_quota_is_passed_on_as_it_comes() {
    let root = tempfile::tempdir().unwrap();
    let wants = [want(root.path(), "A (USA)")];
    let mut server = Server::new(vec![ok(GAME), Some((200, png(2, 1)))]);
    let (news, _) = run(root.path(), &wants, &mut server);
    assert!(news.contains(&LabelNews::Quota(Quota {
        left: 19_953,
        per_day: 20_000
    })));
}

#[test]
fn the_key_build_rs_masks_reads_back() {
    // build.rs masks "devid" byte by byte with 0x5a, 0x79, 0x98, ...
    let masked: String = "devid"
        .bytes()
        .enumerate()
        .map(|(i, b)| {
            format!(
                "{:02x}",
                b ^ 0x5a_u8.wrapping_add((i as u8).wrapping_mul(31))
            )
        })
        .collect();
    assert_eq!(slot::label_sync::unmask(&masked).as_deref(), Some("devid"));
    assert_eq!(slot::label_sync::unmask("zz"), None);
    assert_eq!(
        slot::label_sync::dev_key(),
        None,
        "this test build has a key"
    );
}
