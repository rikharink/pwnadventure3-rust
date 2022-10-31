use std::{
    io::{BufReader, BufWriter},
    net::TcpListener,
};

use dll_syringe::{process::OwnedProcess, Syringe};
use tracing::{info, metadata::LevelFilter};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let process_name = "PwnAdventure3-Win32-Shipping";
    let target_process = OwnedProcess::find_first_by_name(process_name).unwrap();
    let syringe = Syringe::for_process(target_process);
    let listener = TcpListener::bind("127.0.0.1:7331")?;
    let injected_payload = syringe
        .inject("./target/i686-pc-windows-msvc/release/injectee.dll")
        .unwrap();

    info!("it starts...");
    let (stream, _addr) = listener.accept()?;
    let mut reader = BufReader::new(stream);
    let mut writer = BufWriter::new(std::io::stdout());
    std::io::copy(&mut reader, &mut writer).unwrap_or_default();
    info!("it is done...");
    syringe.eject(injected_payload).expect("Couldn't eject");
    Ok(())
}
