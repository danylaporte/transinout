use notify::Watcher;
use parking_lot::Mutex;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicI8, Ordering::Relaxed},
        Arc, Once,
    },
    thread,
    time::Duration,
};

pub(crate) struct Shared {
    id: i32,
    transpose: AtomicI8,
}

impl Shared {
    pub fn get_or_init(id: i32) -> Arc<Shared> {
        INIT.call_once(spawn);
        find_or_add(&mut SHARED.lock(), id)
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn amount(&self) -> i8 {
        self.transpose.load(Relaxed)
    }
}

static INIT: Once = Once::new();
static SHARED: Mutex<Vec<Arc<Shared>>> = Mutex::new(Vec::new());

fn spawn() {
    std::thread::spawn(|| {
        if let Some(d) =
            directories::UserDirs::new().and_then(|d| d.audio_dir().map(|p| p.to_path_buf()))
        {
            let p = d.with_file_name("transinout.toml");
            let p2 = p.clone();

            if let Ok(mut watcher) = notify::recommended_watcher(move |r| {
                if let Ok(_) = r {
                    thread::sleep(Duration::from_millis(100));

                    if let Some(map) = load_transpose_file(&p2) {
                        update_transpose(map);
                    }
                }
            }) {
                let _ = watcher.watch(&p, notify::RecursiveMode::NonRecursive);
            }
        }
    });
}

fn load_transpose_file(path: &Path) -> Option<Vec<i8>> {
    let text = std::fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

fn find_or_add(vec: &mut Vec<Arc<Shared>>, id: i32) -> Arc<Shared> {
    if let Some(v) = vec.get(id as usize) {
        return v.clone();
    }

    let shared = Arc::new(Shared {
        id,
        transpose: AtomicI8::new(0),
    });

    vec.push(shared.clone());

    shared
}

fn update_transpose(map: Vec<i8>) {
    let mut vec = SHARED.lock();
    let mut iter = map.into_iter().enumerate().take(i32::MAX as usize);

    for item in vec.iter_mut() {
        item.transpose
            .store(iter.next().unwrap_or_default().1, Relaxed);
    }

    vec.extend(iter.map(|(id, transpose)| {
        Arc::new(Shared {
            id: id as i32,
            transpose: AtomicI8::new(transpose),
        })
    }));
}
