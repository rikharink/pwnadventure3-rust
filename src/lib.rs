use detour::static_detour;
use std::{
    ffi::{c_char, c_void, CStr},
    iter, mem, thread,
};
use winapi::um::libloaderapi::GetModuleHandleW;

static_detour! {
    static ChatHook: unsafe extern "fastcall" fn(*const c_void, *const c_void, *const c_char);
}

type FnChat = unsafe extern "fastcall" fn(*const c_void, *const c_void, *const c_char);

/// Called when the DLL is attached to the process.
#[ctor::ctor()]
fn ctor() {
    thread::spawn(|| {
        unsafe {
            winapi::um::consoleapi::AllocConsole();
        }

        match trainer() {
            Ok(_) => {
                return;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    });
}

fn game_module_address() -> usize {
    let module = "GameLogic.dll"
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    unsafe {
        let h = GetModuleHandleW(module.as_ptr());
        h as usize
    }
}

fn chat_detour(this: *const c_void, d: *const c_void, msg: *const c_char) {
    let c_str: &CStr = unsafe { CStr::from_ptr(msg) };
    let str_slice: &str = c_str.to_str().unwrap();
    if str_slice.starts_with("#") {
        let mut command_parts: Vec<&str> = str_slice.trim_start_matches("#").split(" ").collect();
        if command_parts.len() == 0 {
            return;
        }

        command_parts.reverse();
        let command = command_parts.pop().unwrap();
        println!("COMMAND: {}", command);
        if command == "STOP" {}
        return;
    }
    unsafe { ChatHook.call(this, d, msg) }
}

unsafe fn setup_chat_detour(game_logic: usize) {
    let chat_offset: usize = 0x551A0;
    let chat_address = game_logic + chat_offset;
    let chat: FnChat = mem::transmute(chat_address);
    println!("Hooking chat at address {:#x}...", chat_address);
    ChatHook
        .initialize(chat, chat_detour)
        .unwrap()
        .enable()
        .unwrap();
    println!("Chat is hooked");
}

fn trainer() -> color_eyre::Result<()> {
    println!("Searching for game logic address....");
    let game_logic = game_module_address();
    println!("Found address {:#x}", game_logic);
    unsafe {
        setup_chat_detour(game_logic);
    }
    Ok(())
}

#[no_mangle]
unsafe extern "system" fn unhook() {
    println!("Unhooking...");
    ChatHook.disable().unwrap();
}
