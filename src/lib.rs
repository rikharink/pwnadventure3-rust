use detour::static_detour;
use std::{
    ffi::{c_char, c_void, CString},
    iter,
    mem::{self, size_of},
    net::TcpStream,
    sync::Mutex,
    thread,
};
use tracing::{info, metadata::LevelFilter};
use winapi::um::libloaderapi::GetModuleHandleW;

static_detour! {
    static ChatHook: unsafe extern "system" fn(*const c_void, *const c_char);
}

type FnChat = unsafe extern "system" fn(*const c_void, *const c_char);

/// Called when the DLL is attached to the process.
#[ctor::ctor()]
fn ctor() {
    thread::spawn(|| {
        let stream = TcpStream::connect("127.0.0.1:7331").unwrap();
        tracing_subscriber::fmt()
            .with_writer(Mutex::new(stream))
            .with_max_level(LevelFilter::DEBUG)
            .init();
        trainer().unwrap();
    });
}

fn game_module_address() -> *const usize {
    let module = "GameLogic.dll"
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    unsafe {
        let h = GetModuleHandleW(module.as_ptr());
        h as *const usize
    }
}

fn chat_detour(this: *const c_void, msg: *const c_char) {
    info!("HACKS");
    unsafe { ChatHook.call(this, msg) }
}

fn trainer() -> color_eyre::Result<()> {
    info!("I'm inside");
    info!("Searching for detour address....");
    let game_logic = game_module_address();
    info!("Found logic {:?}", game_logic);
    info!("Trying to hook...");
    unsafe {
        let offset: usize = 0x551A0;
        let chat_address = game_logic.add(offset / 4);
        let chat: FnChat = mem::transmute(chat_address);
        info!("Initialize hook for addr {:?}...", chat_address);
        let init_hook = ChatHook.initialize(chat, chat_detour);
        match init_hook {
            Ok(init) => {
                info!("INIT OK");
                let enable_hook = init.enable();
                match enable_hook {
                    Ok(_) => info!("ENABLE OK"),
                    Err(_) => info!("ENABLE NOT OK"),
                };
            }
            Err(e) => info!("INIT NOT OK {}", e),
        };
    }
    info!("detoured");
    Ok(())
}
