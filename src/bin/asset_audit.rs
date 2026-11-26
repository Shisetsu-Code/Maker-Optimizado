#![forbid(unsafe_code)]
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs, io::{self, Read}, path::Path};

const MAX_FILE: u64 = 25 * 1024 * 1024;
const FREE_FILE_LIMIT: usize = 20_000;

#[derive(Default)]
struct Audit {
    files: usize,
    bytes: u64,
    unique_bytes: u64,
    digests: BTreeMap<[u8;32], (u64, String)>,
    duplicates: Vec<(String,String)>,
    oversized: Vec<String>,
}

fn walk(root: &Path, directory: &Path, audit: &mut Audit) -> io::Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    let mut buf = [0u8; 64*1024];
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type()?;
        let name = path.strip_prefix(root).map_err(io::Error::other)?.to_string_lossy().replace('\\', "/");
        if kind.is_symlink() {
            return Err(io::Error::new(io::ErrorKind::InvalidData,format!("Symlink not permitted: {name}")));
        }
        if kind.is_dir() { walk(root,&path,audit)?; continue; }
        if !kind.is_file() { return Err(io::Error::new(io::ErrorKind::InvalidData,format!("Not a file: {name}"))); }
        if matches!(name.as_str(),"_headers"|"_redirects"|".assetsignore") { continue; }
        let mut file = fs::File::open(&path)?;
        let mut hash = Sha256::new();
        let mut size = 0u64;
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 { break; }
            hash.update(&buf[..n]);
            size += n as u64;
        }
        audit.files += 1;
        audit.bytes += size;
        if size > MAX_FILE { audit.oversized.push(name.clone()); }
        let digest: [u8;32] = hash.finalize().into();
        if let Some((_, first)) = audit.digests.get(&digest) {
            audit.duplicates.push((name,first.clone()));
        } else {
            audit.unique_bytes += size;
            audit.digests.insert(digest,(size,name));
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::args_os().nth(1).unwrap_or_else(|| "public".into());
    let root = Path::new(&root);
    let mut audit = Audit::default();
    walk(root,root,&mut audit)?;
    let report = serde_json::json!({"files":audit.files,"bytes":audit.bytes,"unique_files":audit.digests.len(),
        "unique_bytes":audit.unique_bytes,"duplicate_bytes":audit.bytes-audit.unique_bytes,
        "duplicate_paths":audit.duplicates.len(),"oversized":audit.oversized,
        "within_free_file_limit":audit.files<=FREE_FILE_LIMIT});
    println!("{}",serde_json::to_string_pretty(&report)?);
    fs::create_dir_all("work")?;
    fs::write("work/asset-audit.json",serde_json::to_vec_pretty(&report)?)?;
    fs::write("work/asset-duplicates.json",serde_json::to_vec_pretty(&audit.duplicates)?)?;
    if audit.files > FREE_FILE_LIMIT || !audit.oversized.is_empty() {
        return Err("Static assets exceed configured free-plan limits".into());
    }
    Ok(())
}
