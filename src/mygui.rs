use native_windows_derive::NwgUi;
use native_windows_gui::{self as nwg, NativeUi};
use winapi;

const WINDOW_WIDTH: i32 = 600;
const WINDOW_HEIGHT: i32 = 250;
const HEIGHT: i32 = 30;
const OFFSET: i32 = 20;

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(size: (WINDOW_WIDTH, WINDOW_HEIGHT), title: "Autofire", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [nwg::stop_thread_dispatch()])]
    window: nwg::Window,

    #[nwg_control(text: "XInput DLL path", size: (WINDOW_WIDTH-OFFSET*2, HEIGHT), position: (OFFSET, OFFSET))]
    dll_lable: nwg::Label,
    #[nwg_control(text: "C:/Windows/System32/XInput1_4.dll", size: (WINDOW_WIDTH-OFFSET*2, HEIGHT), position: (OFFSET, OFFSET+HEIGHT))]
    dll_path: nwg::TextInput,

    #[nwg_control(text: "Cuphead path", size: (WINDOW_WIDTH-OFFSET*2, HEIGHT), position: (OFFSET, 100))]
    exe_lable: nwg::Label,
    #[nwg_control(text: "C:/Program Files (x86)/Steam/steamapps/common/Cuphead", size: (WINDOW_WIDTH-OFFSET*2, HEIGHT), position: (OFFSET, 100+HEIGHT))]
    exe_path: nwg::TextInput,

    #[nwg_control(text: "Patch", size: (100, HEIGHT), position: (WINDOW_WIDTH-OFFSET-100, 200))]
    patch_button: nwg::Button,

    #[nwg_control(text: "Restore", size: (100, HEIGHT), position: (WINDOW_WIDTH-OFFSET-100-100-OFFSET, 200))]
    restore_button: nwg::Button,
}

pub fn init() {
    unsafe {
        winapi::um::winuser::SetProcessDpiAwarenessContext(
            winapi::shared::windef::DPI_AWARENESS_CONTEXT_SYSTEM_AWARE,
        );
    };

    nwg::init().expect("Failed to init Native Windows GUI");
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .size(25)
        .family("Segoe UI")
        .weight(400)
        .build(&mut font)
        .expect("Failed to set default font");
    nwg::Font::set_global_default(Some(font));
}
