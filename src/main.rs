use eframe::egui;

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

struct AutofireApp {
    path: String,
    msg: String,
}

impl Default for AutofireApp {
    fn default() -> Self {
        AutofireApp {
            path: DEFAULT_PATH.into(),
            msg: "".into(),
        }
    }
}

impl eframe::App for AutofireApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.path);
                if ui.button("open folder").clicked() {
                    println!("clicked");
                }
            });
            if ui.button("patch").clicked() {
                self.msg = match patch(DLL_PATH, &self.path) {
                    Ok(_) => "successfully".to_owned(),
                    Err(err) => err.to_string(),
                };
            }
            if !self.msg.is_empty() {
                ui.label(&self.msg);
            }
        });
    }
}

fn patch(dll_path: &str, game_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut data = std::fs::read(dll_path)?;
    let mut game_dll_path = game_path.to_owned();
    game_dll_path.push_str("/XInput1_4.dll");

    let indexes = find_all(&data, &BYTECODE[0]);
    match indexes.len() {
        1 => {
            substitute(&mut data, &BYTECODE[1], indexes[0]);
            std::fs::write(game_dll_path, data)?;
            Ok(())
        }
        0 => Err("pattern not found".into()),
        _ => Err("multiple matches".into()),
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Autofire",
        options,
        Box::new(|_cc| Box::<AutofireApp>::default()),
    )
    .unwrap();
}
