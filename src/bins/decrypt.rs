use std::env;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::sync::mpsc::channel;
use std::thread;

use cbv::{spin, Machine};
use crypto::{digest::Digest, sha1::Sha1};

static CODEX_URL: &str = "http://www.boundvariable.org/codex.umz";
// As found at http://www.boundvariable.org/task.shtml#materials
static CODEX_SHA1_HASH: &str = "088ac79d311db02d9823def598e48f2f8723e98a";
static CODEX_DECRYPTION_KEY: &str = r#"(\b.bb)(\v.vv)06FHPVboundvarHRAk"#;

const SIG_EXPORT: &[u8; 25] = b"UM program follows colon:";

fn codex() -> Vec<u8> {
    let mut buf = Vec::new();
    reqwest::get(CODEX_URL)
        .map(|mut r| {
            r.copy_to(&mut buf)
                .expect("Could not write codex to buffer")
        })
        .expect("Could not access codex URL");
    let mut hash = Sha1::new();
    hash.input(buf.as_slice());
    assert_eq!(hash.result_str(), CODEX_SHA1_HASH);
    buf
}

fn main() {
    let file_name = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("umix_os.um"));

    if let Some(p) = Path::new(&file_name).parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    
    let mut export_file = File::create(&file_name).unwrap();
    let (client_sender, client_receiver) = channel();
    let (machine_sender, machine_receiver) = channel();
    let mut machine = Machine::new(machine_receiver, client_sender, &mut codex().as_slice());
    for byte in CODEX_DECRYPTION_KEY
        .bytes()
        .chain(vec![10u8, 112u8, 10u8].into_iter())
    {
        machine_sender.send(u32::from(byte)).unwrap();
    }
    let handle = thread::spawn(move || {
        let mut buf = Vec::new();
        let mut export = false;
        while let Ok(i) = client_receiver.recv() {
            buf.push(i as u8);
            if export {
                continue;
            }
            if buf.len() > SIG_EXPORT.len() {
                buf.remove(0);
            }
            print!("{}", i as u8 as char);
            if buf == SIG_EXPORT {
                buf.truncate(0);
                export = true;
                println!("\nExporting remaining output to file: '{}'", file_name);
            };
        }
        export_file.write_all(buf.as_slice()).unwrap();
    });

    while let Some(m) = spin(machine) {
        machine = m
    }
    handle.join().unwrap();
}