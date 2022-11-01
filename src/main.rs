use std::io::{self, Read, Write};

use dll_syringe::{process::OwnedProcess, Syringe};
use tracing::{info, metadata::LevelFilter};

fn eject() {
    let process_name = "PwnAdventure3-Win32-Shipping";
    let target_process = OwnedProcess::find_first_by_name(process_name).unwrap();
    let syringe = Syringe::for_process(target_process);
    let injected_payload = syringe
        .find_or_inject("./target/i686-pc-windows-msvc/release/injectee.dll")
        .unwrap();

    let remote_unhook =
        unsafe { syringe.get_raw_procedure::<extern "system" fn()>(injected_payload, "unhook") }
            .unwrap()
            .unwrap();
    remote_unhook.call().unwrap();
    info!("Ejecting...");
    syringe.eject(injected_payload).unwrap();
    std::process::exit(0);
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();
    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let process_name = "PwnAdventure3-Win32-Shipping";
    let target_process = OwnedProcess::find_first_by_name(process_name).unwrap();
    let syringe = Syringe::for_process(target_process);

    let _injected_payload = syringe
        .find_or_inject("./target/i686-pc-windows-msvc/release/injectee.dll")
        .unwrap();

    info!("it starts...");
    ctrlc::set_handler(eject)?;
    pause();
    eject();
    Ok(())
}
