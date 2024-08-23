#![windows_subsystem = "windows"]

use std::{
    fs,
    path::{Path, PathBuf},
};
use windows::{
    core::{w, PCWSTR},
    Win32::UI::{
        HiDpi,
        WindowsAndMessaging::{
            MessageBoxW, IDNO, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK,
            MB_YESNO, MB_YESNOCANCEL, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
        },
    },
};

const DLL_PATH: &str = "C:/Windows/System32/XInput1_4.dll";
const DEFAULT_PATH: &str = "C:/Program Files (x86)/Steam/steamapps/common/Cuphead";
const BYTECODE: [[u8; 25]; 2] = [
    [
        0x8B, 0xC3, 0x48, 0x8B, 0x5C, 0x24, 0x50, 0x48, 0x83, 0xC4, 0x30, 0x41, 0x5E, 0x5F, 0x5E,
        0xC3, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC,
    ],
    [
        0xB8, 0xFF, 0x00, 0x00, 0x00, 0x66, 0x31, 0x47, 0x07, 0x8B, 0xC3, 0x48, 0x8B, 0x5C, 0x24,
        0x50, 0x48, 0x83, 0xC4, 0x30, 0x41, 0x5E, 0x5F, 0x5E, 0xC3,
    ],
];

fn find_all<T: PartialEq>(data: &[T], req: &[T]) -> Vec<usize> {
    (0..data.len() - req.len() + 1)
        .filter(|&i| data[i..i + req.len()] == req[..])
        .collect()
}

fn substitute<T: Clone>(data: &mut [T], req: &[T], i: usize) {
    data[i..i + req.len()]
        .iter_mut()
        .zip(req.iter())
        .for_each(|(d, b)| *d = b.clone());
}

fn patch(dst_path: &Path) -> std::io::Result<usize> {
    let mut data = std::fs::read(DLL_PATH)?;
    let indexes = find_all(&data, &BYTECODE[0]);
    if indexes.len() == 1 {
        substitute(&mut data, &BYTECODE[1], indexes[0]);
        std::fs::write(dst_path, data)?;
    }
    Ok(indexes.len())
}

fn msg_box(lptext: PCWSTR, utype: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    unsafe { MessageBoxW(None, lptext, w!("Autofire"), utype) }
}

fn to_pcwstr(data: &str) -> (PCWSTR, Vec<u16>) {
    let vec = data.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    (PCWSTR(vec.as_ptr()), vec)
}

fn main() {
    unsafe {
        let _ = HiDpi::SetProcessDpiAwarenessContext(HiDpi::DPI_AWARENESS_CONTEXT_SYSTEM_AWARE);
    }

    let mut path: PathBuf = DEFAULT_PATH.into();

    let skip_pick = if path.exists() {
        match msg_box(
            to_pcwstr(&format!("Использовать путь по умолчанию?\n{DEFAULT_PATH}")).0,
            MB_YESNOCANCEL | MB_ICONQUESTION,
        ) {
            IDYES => true,
            IDNO => false,
            _ => return,
        }
    } else {
        false
    };

    if !skip_pick {
        path = rfd::FileDialog::new()
            .set_title("Select Cuphead folder")
            .pick_folder()
            .unwrap();
    }

    path.push("XInput1_4.dll");
    let res = if path.exists() {
        if msg_box(
            w!("Патч уже установлен. Удалить?"),
            MB_YESNO | MB_ICONQUESTION,
        ) != IDYES
        {
            return;
        }
        match fs::remove_file(path) {
            Ok(()) => Ok("Патч удален".to_owned()),
            Err(err) => Err(err.to_string()),
        }
    } else {
        match patch(path.as_path()) {
            Ok(1) => Ok("Патч установлен".to_owned()),
            Ok(_) => Err("Неподдерживаемая библиотека".to_owned()),
            Err(err) => Err(err.to_string()),
        }
    };

    let utype = if res.is_ok() {
        MB_ICONINFORMATION
    } else {
        MB_ICONERROR
    } | MB_OK;
    let lptext = to_pcwstr(&match res {
        Ok(text) => text,
        Err(text) => text,
    });
    msg_box(lptext.0, utype);
}
