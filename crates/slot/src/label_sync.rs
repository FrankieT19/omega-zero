//! Cart labels from ScreenScraper. Each cart without a label is looked up by its ROM's CRC
//! with the player's own ScreenScraper account, and the scan of its real label, from the
//! ROM's own region first, is saved where the shelf looks for it. A cart ScreenScraper has
//! no label scan for keeps the label slot draws.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use slot_store::Platform;

use crate::json::Json;

const API: &str = "https://api.screenscraper.fr/api2";
pub const SOFTNAME: &str = "slot-rg35xxsp";
pub const ACCOUNT_FILE: &str = "Config/screenscraper.conf";

/// The widest label scan worth fetching: the shelf's labels are narrower than this even
/// with the cart drawn large.
const LABEL_MAX_W: u32 = 600;

/// The key ScreenScraper gives a program, separate from the player's account and required
/// with it on every request. It comes from whoever builds slot, never from the source, and
/// build.rs masks it so it is not plain text in the binary: a courtesy, not a secret.
pub fn dev_key() -> Option<(&'static str, &'static str)> {
    static KEY: std::sync::OnceLock<Option<(String, String)>> = std::sync::OnceLock::new();
    KEY.get_or_init(|| {
        let (id, pw) = option_env!("SLOT_SS_KEY")?.split_once(':')?;
        Some((unmask(id)?, unmask(pw)?))
    })
    .as_ref()
    .map(|(id, pw)| (id.as_str(), pw.as_str()))
}

/// Reads back what build.rs masked: each byte, as hex, crossed with a byte that steps on.
pub fn unmask(hex: &str) -> Option<String> {
    let bytes = (0..hex.len())
        .step_by(2)
        .enumerate()
        .map(|(i, at)| {
            let b = u8::from_str_radix(hex.get(at..at + 2)?, 16).ok()?;
            Some(b ^ 0x5a_u8.wrapping_add((i as u8).wrapping_mul(31)))
        })
        .collect::<Option<Vec<u8>>>()?;
    String::from_utf8(bytes).ok()
}

/// The player's ScreenScraper account, kept in Config/screenscraper.conf.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Account {
    pub user: String,
    pub password: String,
}

impl Account {
    pub fn load(root: &Path) -> Option<Account> {
        let text = std::fs::read_to_string(root.join(ACCOUNT_FILE)).ok()?;
        let field = |key: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
                .map(str::to_string)
        };
        let account = Account {
            user: field("user")?,
            password: field("password")?,
        };
        (!account.user.is_empty()).then_some(account)
    }

    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        let path = root.join(ACCOUNT_FILE);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = format!("user={}\npassword={}\n", self.user, self.password);
        slot_store::atomic_write(&path, text.as_bytes())
    }

    pub fn forget(root: &Path) {
        let _ = std::fs::remove_file(root.join(ACCOUNT_FILE));
    }
}

/// A cart to find a label for.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Want {
    pub platform: Platform,
    pub stem: String,
    pub rom: PathBuf,
    /// The GBA header's game code, whose last letter is the cart's region.
    pub code: String,
}

impl Want {
    pub fn label_path(&self, root: &Path) -> PathBuf {
        label_path(root, self.platform, &self.stem)
    }
}

pub fn label_path(root: &Path, platform: Platform, stem: &str) -> PathBuf {
    root.join("Labels")
        .join(platform.dir_name())
        .join(format!("{stem}.png"))
}

/// ScreenScraper's number for each system.
pub fn system_id(platform: Platform) -> u32 {
    match platform {
        Platform::Gba => 12,
        Platform::Gb => 9,
        Platform::Gbc => 10,
    }
}

const TAG_REGIONS: [(&str, &str); 14] = [
    ("USA", "us"),
    ("Europe", "eu"),
    ("Japan", "jp"),
    ("World", "wor"),
    ("France", "fr"),
    ("Germany", "de"),
    ("Spain", "sp"),
    ("Italy", "it"),
    ("Australia", "au"),
    ("Brazil", "br"),
    ("Korea", "kr"),
    ("China", "cn"),
    ("Netherlands", "nl"),
    ("Sweden", "se"),
];

const CODE_REGIONS: [(char, &str); 10] = [
    ('E', "us"),
    ('P', "eu"),
    ('J', "jp"),
    ('D', "de"),
    ('F', "fr"),
    ('S', "sp"),
    ('I', "it"),
    ('U', "au"),
    ('K', "kr"),
    ('C', "cn"),
];

/// The regions to take a label from, best first: those named in the ROM's file name, then
/// the one its game code says, then a world release and the three big ones.
pub fn regions(stem: &str, code: &str) -> Vec<&'static str> {
    let mut out = Vec::new();
    let mut add = |r: &'static str| {
        if !out.contains(&r) {
            out.push(r);
        }
    };
    for tag in stem
        .split(['(', ')'])
        .skip(1)
        .step_by(2)
        .flat_map(|t| t.split(','))
    {
        if let Some((_, r)) = TAG_REGIONS.iter().find(|(name, _)| *name == tag.trim()) {
            add(r);
        }
    }
    if let Some(letter) = code.chars().nth(3) {
        if let Some((_, r)) = CODE_REGIONS.iter().find(|(c, _)| *c == letter) {
            add(r);
        }
    }
    for r in ["wor", "us", "eu", "jp"] {
        add(r);
    }
    out
}

/// The CRC-32 a ROM is known by.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    !bytes.iter().fold(!0u32, |c, &b| {
        table[((c ^ b as u32) & 0xff) as usize] ^ (c >> 8)
    })
}

fn escape(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// What ScreenScraper is asked about one ROM.
pub struct Lookup<'a> {
    pub system: u32,
    pub crc: u32,
    pub file_name: &'a str,
    pub size: u64,
}

pub fn game_url(key: (&str, &str), account: &Account, rom: &Lookup) -> String {
    let params = [
        ("devid", key.0.to_string()),
        ("devpassword", key.1.to_string()),
        ("softname", SOFTNAME.to_string()),
        ("output", "json".to_string()),
        ("ssid", account.user.clone()),
        ("sspassword", account.password.clone()),
        ("crc", format!("{:08X}", rom.crc)),
        ("systemeid", rom.system.to_string()),
        ("romtype", "rom".to_string()),
        ("romnom", rom.file_name.to_string()),
        ("romtaille", rom.size.to_string()),
    ];
    let query: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{k}={}", escape(v)))
        .collect();
    format!("{API}/jeuInfos.php?{}", query.join("&"))
}

/// A media URL from a reply, asked for as a PNG no wider than a label needs.
pub fn media_url(url: &str) -> String {
    format!("{url}&maxwidth={LABEL_MAX_W}&outputformat=png")
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Media {
    pub kind: String,
    pub region: String,
    pub url: String,
}

/// Whose credentials ScreenScraper turned away.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    Player,
    Program,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Reply {
    Game(Vec<Media>),
    NotFound,
    Refused(Who),
    /// Too many requests at once: worth one more try after a pause.
    Busy,
    /// The account's lookups for today are spent.
    Spent,
    /// The API is closed, offline, or has blacklisted this version of slot.
    Down,
    Other(u16),
}

/// How many lookups the account has left today.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Quota {
    pub left: u64,
    pub per_day: u64,
}

pub fn read_reply(status: u16, body: &[u8]) -> (Reply, Option<Quota>) {
    let text = String::from_utf8_lossy(body);
    if text.contains("Erreur de login") {
        let who = if text.to_lowercase().contains("développeur") {
            Who::Program
        } else {
            Who::Player
        };
        return (Reply::Refused(who), None);
    }
    let reply = match status {
        200 => None,
        403 => Some(Reply::Refused(Who::Program)),
        404 | 431 => Some(Reply::NotFound),
        429 => Some(Reply::Busy),
        430 => Some(Reply::Spent),
        401 | 423 | 426 => Some(Reply::Down),
        s => Some(Reply::Other(s)),
    };
    if let Some(reply) = reply {
        return (reply, None);
    }
    let Some(json) = Json::parse(&text) else {
        return (Reply::Other(status), None);
    };
    let response = json.get("response");
    let quota = response.and_then(|r| r.get("ssuser")).and_then(|u| {
        let per_day = u.get("maxrequestsperday")?.count()?;
        let today = u.get("requeststoday").and_then(Json::count).unwrap_or(0);
        Some(Quota {
            left: per_day.saturating_sub(today),
            per_day,
        })
    });
    let Some(game) = response.and_then(|r| r.get("jeu")) else {
        return (Reply::NotFound, quota);
    };
    let medias = game
        .get("medias")
        .map(Json::list)
        .unwrap_or_default()
        .iter()
        .filter_map(|m| {
            Some(Media {
                kind: m.get("type")?.text()?.to_string(),
                region: m
                    .get("region")
                    .and_then(Json::text)
                    .unwrap_or_default()
                    .to_string(),
                url: m.get("url")?.text()?.to_string(),
            })
        })
        .collect();
    (Reply::Game(medias), quota)
}

/// The scan of the cart's own label, from the first of `regions` that has one, else from
/// any region.
pub fn pick_label<'a>(medias: &'a [Media], regions: &[&str]) -> Option<&'a Media> {
    let scans: Vec<&Media> = medias
        .iter()
        .filter(|m| m.kind == "support-texture")
        .collect();
    regions
        .iter()
        .find_map(|r| scans.iter().find(|m| m.region == *r))
        .or_else(|| scans.first())
        .copied()
}

/// How a sync went.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tally {
    pub saved: usize,
    /// On ScreenScraper, but with no scan of its label.
    pub bare: usize,
    /// Not on ScreenScraper at all, such as a ROM hack.
    pub unknown: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyncFail {
    /// This build of slot was made without ScreenScraper's key.
    NoKey,
    Refused(Who),
    Spent,
    Down,
    Offline,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LabelNews {
    Working {
        done: usize,
        total: usize,
        title: String,
    },
    Saved {
        platform: Platform,
        stem: String,
    },
    Quota(Quota),
    Finished(Tally),
    Failed(SyncFail),
}

pub trait Http {
    /// The status and body of a GET, or nothing when the server could not be reached.
    fn get(&mut self, url: &str) -> Option<(u16, Vec<u8>)>;
}

/// HTTPS through the curl BaseOS ships, with the URL, which carries both passwords, handed
/// over on stdin rather than where `ps` would show it.
pub struct Curl;

impl Http for Curl {
    fn get(&mut self, url: &str) -> Option<(u16, Vec<u8>)> {
        let mut child = Command::new("curl")
            .args([
                "-s",
                "-L",
                "-m",
                "60",
                "-A",
                SOFTNAME,
                "-w",
                "\n%{http_code}",
                "-K",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let config = format!(
            "url = \"{}\"\n",
            url.replace('\\', "\\\\").replace('"', "\\\"")
        );
        child.stdin.take()?.write_all(config.as_bytes()).ok()?;
        let out = child.wait_with_output().ok()?;
        let split = out.stdout.iter().rposition(|&b| b == b'\n')?;
        let status: u16 = std::str::from_utf8(&out.stdout[split + 1..])
            .ok()?
            .trim()
            .parse()
            .ok()?;
        (status != 0).then(|| (status, out.stdout[..split].to_vec()))
    }
}

fn is_png(bytes: &[u8]) -> bool {
    png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .is_ok_and(|r| r.info().width > 0 && r.info().height > 0)
}

fn keep(path: &Path, png: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    slot_store::atomic_write(path, png)
}

/// One sync: whose account, which carts, and the card their labels go on.
pub struct Sync<'a> {
    pub key: (&'a str, &'a str),
    pub account: &'a Account,
    pub root: &'a Path,
    pub wants: &'a [Want],
}

/// Looks up each cart and saves the label scans found, telling `news` as it goes. Stops early
/// when `stop` is set, and for good on any failure the next cart would meet too.
pub fn sync(
    job: &Sync,
    http: &mut dyn Http,
    stop: &AtomicBool,
    pause: &mut dyn FnMut(Duration),
    news: &mut dyn FnMut(LabelNews),
) {
    let Sync {
        key,
        account,
        root,
        wants,
    } = *job;
    let mut tally = Tally::default();
    for (done, want) in wants.iter().enumerate() {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        news(LabelNews::Working {
            done,
            total: wants.len(),
            title: title(&want.stem),
        });
        let Ok(bytes) = std::fs::read(&want.rom) else {
            tally.unknown += 1;
            continue;
        };
        let file_name = want
            .rom
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let url = game_url(
            key,
            account,
            &Lookup {
                system: system_id(want.platform),
                crc: crc32(&bytes),
                file_name: &file_name,
                size: bytes.len() as u64,
            },
        );
        let mut asked = None;
        for attempt in 0..2 {
            let Some((status, body)) = http.get(&url) else {
                return news(LabelNews::Failed(SyncFail::Offline));
            };
            let (reply, quota) = read_reply(status, &body);
            if let Some(q) = quota {
                news(LabelNews::Quota(q));
            }
            if reply == Reply::Busy && attempt == 0 {
                pause(Duration::from_secs(2));
                continue;
            }
            asked = Some(reply);
            break;
        }
        match asked {
            Some(Reply::Game(medias)) => {
                let regions = regions(&want.stem, &want.code);
                let scan = pick_label(&medias, &regions)
                    .and_then(|m| http.get(&media_url(&m.url)))
                    .filter(|(status, body)| *status == 200 && is_png(body));
                match scan {
                    Some((_, png)) if keep(&want.label_path(root), &png).is_ok() => {
                        tally.saved += 1;
                        news(LabelNews::Saved {
                            platform: want.platform,
                            stem: want.stem.clone(),
                        });
                    }
                    _ => tally.bare += 1,
                }
            }
            Some(Reply::NotFound | Reply::Other(_) | Reply::Busy) | None => tally.unknown += 1,
            Some(Reply::Refused(who)) => return news(LabelNews::Failed(SyncFail::Refused(who))),
            Some(Reply::Spent) => return news(LabelNews::Failed(SyncFail::Spent)),
            Some(Reply::Down) => return news(LabelNews::Failed(SyncFail::Down)),
        }
    }
    news(LabelNews::Finished(tally));
}

/// A cart's name for showing while it is looked up: its file name without the tags.
pub fn title(stem: &str) -> String {
    stem.split(" (").next().unwrap_or(stem).trim().to_string()
}

pub enum LabelJob {
    Sync { account: Account, wants: Vec<Want> },
    Stop,
}

pub trait LabelJobs: Send {
    /// This build can reach ScreenScraper at all.
    fn present(&self) -> bool;

    fn ask(&mut self, job: LabelJob);

    fn news(&mut self) -> Vec<LabelNews>;
}

/// Syncs on a thread of its own, one cart at a time, which is all a free account allows.
pub struct Worker {
    jobs: Sender<(Account, Vec<Want>)>,
    news: Receiver<LabelNews>,
    stop: Arc<AtomicBool>,
}

impl Worker {
    pub fn spawn(root: PathBuf, key: (&'static str, &'static str)) -> Worker {
        let (jobs, rx) = channel::<(Account, Vec<Want>)>();
        let (tx, news) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        std::thread::spawn(move || {
            for (account, wants) in rx {
                stopping.store(false, Ordering::Relaxed);
                let mut tell = |n| {
                    let _ = tx.send(n);
                };
                let job = Sync {
                    key,
                    account: &account,
                    root: &root,
                    wants: &wants,
                };
                sync(
                    &job,
                    &mut Curl,
                    &stopping,
                    &mut std::thread::sleep,
                    &mut tell,
                );
            }
        });
        Worker { jobs, news, stop }
    }
}

impl LabelJobs for Worker {
    fn present(&self) -> bool {
        true
    }

    fn ask(&mut self, job: LabelJob) {
        match job {
            LabelJob::Sync { account, wants } => {
                let _ = self.jobs.send((account, wants));
            }
            LabelJob::Stop => self.stop.store(true, Ordering::Relaxed),
        }
    }

    fn news(&mut self) -> Vec<LabelNews> {
        self.news.try_iter().collect()
    }
}

/// A build without ScreenScraper's key: it says so whenever asked to sync.
#[derive(Default)]
pub struct NoKey {
    news: Vec<LabelNews>,
}

impl LabelJobs for NoKey {
    fn present(&self) -> bool {
        false
    }

    fn ask(&mut self, job: LabelJob) {
        if let LabelJob::Sync { .. } = job {
            self.news.push(LabelNews::Failed(SyncFail::NoKey));
        }
    }

    fn news(&mut self) -> Vec<LabelNews> {
        std::mem::take(&mut self.news)
    }
}

pub fn label_jobs(root: &Path) -> Box<dyn LabelJobs> {
    match dev_key() {
        Some(key) => Box::new(Worker::spawn(root.to_path_buf(), key)),
        None => Box::new(NoKey::default()),
    }
}
