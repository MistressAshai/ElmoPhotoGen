use md5;
use nanoserde::SerJson;
use std::{fs::{self}, io::{self, Read}, path::Path, time::UNIX_EPOCH};

#[derive(SerJson)]
#[allow(non_snake_case)]
struct LpHeadInfo {
    id: u64,
    index: u32,
    photoHeadDatas: Vec<PhotoHeadData>,
}

#[derive(SerJson)]
#[allow(non_snake_case)]
struct PhotoHeadData {
    id: u32,
    time: u64,
    fileName: String,
    hash: Vec<u8>,
}

fn pause() {
    println!("\nPress Enter to exit...");
    let _ = io::stdin().read(&mut [0u8]).unwrap();
    std::process::exit(1);
}

fn get_uuid() -> String {
    let path = std::env::current_dir().unwrap();
    let folder_name = path
        .file_name()
        .unwrap()
        .to_string_lossy();

    if !folder_name.contains("_lobby_photo") {
        println!(r#"Invalid folder: must run from "GIRLS' FRONTLINE 2 EXILIUM\GF2_Exilium_Data\LocalCache\Data\<uid>_lobby_photo""#);
        pause();
    }

    let uuid = folder_name.split_once('_')
        .map(|(left, _)| left)
        .unwrap_or(&folder_name);

    uuid.to_string()
}

fn get_all_photos() -> Vec<PhotoHeadData> {
    let dir = std::env::current_dir().expect("no cwd");

    let mut photos = Vec::new();

    for entry in fs::read_dir(dir).expect("failed to read dir") {
        let entry = entry.expect("bad entry");
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // filter image types
            let valid_types = ["png", "jpg", "jpeg"];
            if valid_types.contains(&ext.as_str()) {
                photos.push(create_photoheaddata(&path));
            }
        }
    }

    photos
}

fn create_photoheaddata(path: &Path) -> PhotoHeadData {
    let data = fs::read(&path).expect("failed to read file");

    let digest = md5::compute(&data);
    let hash = digest.0.to_vec();

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let metadata = fs::metadata(&path).expect("failed to read metadata");
    let time = metadata
        .created()
        .or_else(|_| metadata.modified())
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    PhotoHeadData {
        id: 0,
        time,
        fileName: file_name,
        hash,
    }
}

fn create_header(uuid: &str) -> LpHeadInfo {
    let mut photos = get_all_photos();
    let photo_count = photos.len() as u32;

    println!("Found {photo_count} photos...");

    for (i, photo) in photos.iter_mut().enumerate() {
        photo.id = i as u32 + 1;

        println!("      {}", photo.fileName);
    }

    LpHeadInfo {
        id: uuid.parse().unwrap(),
        index: photo_count,
        photoHeadDatas: photos,
    }
}

#[cfg(windows)]
fn set_title() {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::System::Console::SetConsoleTitleW;

    let title = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let wide: Vec<u16> = OsStr::new(&title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { SetConsoleTitleW(wide.as_ptr()); }
}

fn main() {
    set_title();

    let uuid = get_uuid();
    let header = create_header(&uuid);
    let json = nanoserde::SerJson::serialize_json(&header);

    let path = std::env::current_dir().expect("no cwd");
    let parent_folder = path
        .parent()
        .expect("no parent dir")
        .join("lpheadinfo");

    let output = parent_folder.join(&uuid);
    fs::write(&output, json).expect("failed to write json");

    println!("Successfully sent photos to the Elmo!");
    println!("Remember, photos must be 16:9, otherwise stretching will occur!");
    pause();
}