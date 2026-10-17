//! The frontend boots immediately; hardware facts remain available for crash diagnostics.
use crate::common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use omega_input::{Action, Btn};
use omega_power::parse_baseos_release;
use omega_store::{write_app_state, AppState};
use omega_zero::app::Phase;
use omega_zero::opening::Facts;
const MEMINFO: &str = "MemTotal:        1003916 kB\nMemFree:          812340 kB\nMemAvailable:     893120 kB\nBuffers:            1024 kB\n";
#[test]
fn the_facts_are_the_handhelds_own() {
    let release = parse_baseos_release("BASEOS_TARGET=rg35xxsp\nBASEOS_MODEL=\"RG35XX SP\"\n");
    let f = Facts::from_parts(Some(release), Some(MEMINFO));
    assert_eq!(
        f,
        Facts {
            model: "RG35XX SP".into(),
            mem_mb: 1024,
            avail_mb: 872,
            battery_mah: Some(3300),
        },
        "the kernel's 980 MB are the 1 GB fitted"
    );
}

#[test]
fn a_battery_nobody_checked_goes_untold_and_memory_rounds_up_to_its_chips() {
    let release = parse_baseos_release("BASEOS_TARGET=rg40xxh\n");
    let f = Facts::from_parts(
        Some(release),
        Some("MemTotal: 2012345 kB\nMemAvailable: 1700000 kB\n"),
    );
    assert_eq!(
        f.model, "RG40XXH",
        "the target names it when no model is given"
    );
    assert_eq!(f.battery_mah, None);
    assert_eq!((f.mem_mb, f.avail_mb), (2048, 1660));
}

#[test]
fn a_computer_standing_in_tells_the_rg35xx_sps_facts() {
    assert_eq!(Facts::from_parts(None, None), Facts::default());
    let f = Facts::default();
    assert_eq!(f.model, "RG35XX SP");
    assert_eq!((f.mem_mb, f.battery_mah), (1024, Some(3300)));
}

#[test]
fn no_intro_intercepts_the_first_carousel_input() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_app_state(
        d.path(),
        &AppState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (mut app, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    app.start_opening(&Facts::default());
    assert!(!app.opening_shown());
    app.apply(Action::GbaDown(Btn::Start));
    assert!(matches!(app.phase(), Phase::Settings { .. }));
}
