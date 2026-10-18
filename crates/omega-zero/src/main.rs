#[cfg(not(feature = "host"))]
mod device_app;
#[cfg(feature = "host")]
mod host_app;

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next();
    #[cfg(not(feature = "host"))]
    if first.as_deref() == Some("--retroarch-session") {
        omega_zero::retroarch::run(args.collect());
        return;
    }
    if first.as_deref() == Some("--init-root") {
        let Some(dir) = args.next() else {
            eprintln!("omega-zero: --init-root needs a directory");
            std::process::exit(2);
        };
        omega_zero::root::ensure(std::path::Path::new(&dir));
        return;
    }

    #[cfg(feature = "host")]
    host_app::run();

    #[cfg(not(feature = "host"))]
    device_app::run();
}
