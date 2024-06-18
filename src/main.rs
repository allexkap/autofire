use std::io::Write;

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

fn pause() {
    std::process::Command::new("cmd.exe")
        .arg("/c")
        .arg("pause")
        .status()
        .unwrap();
}

fn patch() -> std::io::Result<usize> {
    let mut data = std::fs::read("C:/Windows/System32/XInput1_4.dll")?;
    let mut path = String::new();

    println!("default path {DEFAULT_PATH}");
    print!("custom path or nothing: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut path)?;
    path = path.lines().next().unwrap().into();
    if path.len() == 0 {
        path = DEFAULT_PATH.into();
    }
    path.push_str("/XInput1_4.dll");

    let indexes = find_all(&data, &BYTECODE[0]);
    if indexes.len() == 1 {
        substitute(&mut data, &BYTECODE[1], indexes[0]);
        std::fs::write(path, data)?;
    }
    Ok(indexes.len())
}

fn main() {
    match patch() {
        Err(err) => println!("{}", err),
        Ok(0) => println!("pattern not found"),
        Ok(1) => println!("successfully"),
        Ok(_) => println!("multiple matches"),
    }
    pause();
}
