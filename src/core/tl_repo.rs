use std::{fs, io::{Read, Write}, path::{Path, PathBuf}, sync::{atomic::{self, AtomicUsize}, Arc, Mutex}};

use arc_swap::ArcSwap;
use fnv::FnvHashMap;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use size::Size;
use threadpool::ThreadPool;

use super::{gui::SimpleYesNoDialog, hachimi::LocalizedData, http::{self, AsyncRequest}, utils, Error, Gui, Hachimi};

#[derive(Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub index: String
}

pub fn new_meta_index_request() -> AsyncRequest<Vec<RepoInfo>> {
    let meta_index_url = &Hachimi::instance().config.load().meta_index_url;
    AsyncRequest::with_json_response(ureq::get(meta_index_url))
}

#[derive(Deserialize)]
struct RepoIndex {
    base_url: String,
    zip_url: String,
    zip_dir: String,
    files: Vec<RepoFile>
}

#[derive(Deserialize, Clone)]
struct RepoFile {
    path: String,
    hash: String,
    size: usize
}

impl RepoFile {
    fn get_fs_path(&self, root_dir: &Path) -> PathBuf {
        // Modern Windows versions support forward slashes anyways but it doesn't hurt to do something so trivial
        #[cfg(target_os = "windows")]
        return root_dir.join(&self.path.replace("/", "\\"));

        #[cfg(not(target_os = "windows"))]
        return root_dir.join(&self.path);
    }
}

#[derive(Clone)]
struct UpdateInfo {
    base_url: String,
    zip_url: String,
    zip_dir: String,
    files: Vec<RepoFile>, // only contains files needed for update
    is_new_repo: bool,
    cached_files: FnvHashMap<String, String>, // from repo cache
    size: usize
}

#[derive(Default, Clone)]
pub struct UpdateProgress {
    pub current: usize,
    pub total: usize
}

impl UpdateProgress {
    pub fn new(current: usize, total: usize) -> UpdateProgress {
        UpdateProgress {
            current,
            total
        }
    }
}

const REPO_CACHE_FILENAME: &str = ".tl_repo_cache";
#[derive(Serialize, Deserialize, Default)]
struct RepoCache {
    base_url: String,
    files: FnvHashMap<String, String> // path: hash
}

#[derive(Default)]
pub struct Updater {
    update_check_mutex: Mutex<()>,
    new_update: ArcSwap<Option<UpdateInfo>>,
    progress: ArcSwap<Option<UpdateProgress>>
}

const LOCALIZED_DATA_DIR: &str = "localized_data";
const CHUNK_SIZE: usize = 8192; // 8KiB
const NUM_THREADS: usize = 8;
const INCREMENTAL_UPDATE_LIMIT: usize = 200;

struct DownloadJob {
    agent: ureq::Agent,
    hasher: blake3::Hasher,
    buffer: [u8; CHUNK_SIZE]
}

impl DownloadJob {
    fn new() -> DownloadJob {
        DownloadJob {
            agent: ureq::Agent::new(),
            hasher: blake3::Hasher::new(),
            buffer: [0u8; CHUNK_SIZE]
        }
    }

    fn execute(&mut self, file_path: &Path, url: &str, file_hash: &str, add_bytes: impl Fn(usize)) -> Result<String, Error> {
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(file_path)?;

        let res = self.agent.get(url).call()?;
        http::download_file_buffered(res, &mut file, &mut self.buffer, |bytes| {
            self.hasher.update(bytes);
            add_bytes(bytes.len());
        })?;
        file.sync_data()?;

        // Hash the file
        let hash = self.hasher.finalize().to_hex().to_string();
        if hash != file_hash {
            return Err(Error::FileHashMismatch(file_path.to_str().unwrap_or("").to_string()));
        }

        self.hasher.reset();

        Ok(hash)
    }
}

impl Updater {
    pub fn check_for_updates(self: Arc<Self>) {
        std::thread::spawn(move || {
            if let Err(e) = self.check_for_updates_internal() {
                error!("{}", e);
            }
        });
    }

    fn check_for_updates_internal(&self) -> Result<(), Error> {
        // Prevent multiple update checks running at the same time
        let Ok(_guard) = self.update_check_mutex.try_lock() else {
            return Ok(());
        };

        let hachimi = Hachimi::instance();
        let config = hachimi.config.load();
        let Some(index_url) = &config.translation_repo_index else {
            return Ok(());
        };
        let ld_dir_path = config.localized_data_dir.as_ref().map(|p| hachimi.get_data_path(p));

        if let Some(mutex) = Gui::instance() {
            mutex.lock().unwrap().show_notification(&t!("notification.checking_for_tl_updates"));
        }

        let index: RepoIndex = http::get_json(index_url)?;

        let cache_path = hachimi.get_data_path(REPO_CACHE_FILENAME);
        let repo_cache = if fs::metadata(&cache_path).is_ok() {
            let json = fs::read_to_string(&cache_path)?;
            serde_json::from_str(&json)?
        }
        else {
            RepoCache::default()
        };

        let is_new_repo = index.base_url != repo_cache.base_url;
        let mut update_files: Vec<RepoFile> = Vec::new();
        let mut update_size: usize = 0;
        let mut total_size: usize = 0;
        for file in index.files.iter() {
            if file.path.contains("..") || Path::new(&file.path).has_root() {
                warn!("File path '{}' sanitized", file.path);
                continue;
            }

            let updated = if is_new_repo {
                // redownload every single file because the directory will be deleted
                true
            }
            else if let Some(hash) = repo_cache.files.get(&file.path) {
                if hash == &file.hash {
                    // download if the file doesn't actually exist on disk
                    ld_dir_path.as_ref().map(|p| !p.join(&file.path).is_file()).unwrap_or(true)
                }
                else {
                    true
                }
            }
            else {
                // file doesnt exist yet, download it
                true
            };

            if updated {
                update_files.push(file.clone());
                update_size += file.size;
            }
            total_size += file.size;
        }

        let is_zip_download = update_files.len() > INCREMENTAL_UPDATE_LIMIT;
        if !update_files.is_empty() {
            self.new_update.store(Arc::new(Some(UpdateInfo {
                is_new_repo,
                base_url: index.base_url,
                zip_url: index.zip_url,
                zip_dir: index.zip_dir,
                files: update_files,
                cached_files: repo_cache.files,
                size: if is_zip_download { total_size } else { update_size }
            })));
            if let Some(mutex) = Gui::instance() {
                mutex.lock().unwrap().show_window(Box::new(SimpleYesNoDialog::new(
                    &t!("tl_update_dialog.title"),
                    &t!("tl_update_dialog.content", size = Size::from_bytes(update_size)),
                    |ok| {
                        if !ok { return; }
                        Hachimi::instance().tl_updater.clone().run();
                    }
                )));
            }
        }
        else if let Some(mutex) = Gui::instance() {
            mutex.lock().unwrap().show_notification(&t!("notification.no_tl_updates"));
        }
        
        Ok(())
    }

    pub fn run(self: Arc<Self>) {
        std::thread::spawn(move || {
            if let Err(e) = self.clone().run_internal() {
                error!("{}", e);
                self.progress.store(Arc::new(None));
                if let Some(mutex) = Gui::instance() {
                    mutex.lock().unwrap().show_notification(&t!("notification.update_failed", reason = e.to_string()));
                }
            }
        });
    }

    fn run_internal(self: Arc<Self>) -> Result<(), Error> {
        let Some(update_info) = (**self.new_update.load()).clone() else {
            return Ok(());
        };
        self.new_update.store(Arc::new(None));

        self.progress.store(Arc::new(Some(UpdateProgress::new(0, update_info.size))));
        if let Some(mutex) = Gui::instance() {
            mutex.lock().unwrap().update_progress_visible = true;
        }

        // Empty the localized data so files couldnt be accessed while update is in progress
        let hachimi = Hachimi::instance();
        hachimi.localized_data.store(Arc::new(LocalizedData::default()));

        // Clear the localized data if downloading from a new repo
        let localized_data_dir = hachimi.get_data_path(LOCALIZED_DATA_DIR);
        if update_info.is_new_repo {
            // rm -rf
            if let Ok(meta) = fs::metadata(&localized_data_dir) {
                if meta.is_dir() {
                    fs::remove_dir_all(&localized_data_dir)?;
                }
            }
        }

        fs::create_dir_all(&localized_data_dir)?;

        // Download the files
        let cached_files = Arc::new(Mutex::new(update_info.cached_files.clone()));
        // There are errors that can be ignored, let the downloader count how many non-fatal errors there are
        let error_count = if update_info.files.len() > INCREMENTAL_UPDATE_LIMIT {
            // It would be too slow to do a large amount of HTTP requests, so just download a zip file and extract it
            self.clone().download_zip(&update_info, &localized_data_dir, cached_files.clone())
        }
        else {
            self.clone().download_incremental(&update_info, &localized_data_dir, cached_files.clone())
        }?; // <-- looga this question mark
        
        // Modify the config if needed
        if hachimi.config.load().localized_data_dir.is_none() {
            let mut config = (**hachimi.config.load()).clone();
            config.localized_data_dir = Some(LOCALIZED_DATA_DIR.to_owned());
            hachimi.save_and_reload_config(config)?;
        }

        // Drop the download state
        self.progress.store(Arc::new(None));

        // Reload the localized data
        hachimi.load_localized_data();

        // Save the repo cache (done last so if any of the previous fails, the entire update would be voided)
        let repo_cache = RepoCache {
            base_url: update_info.base_url.clone(),
            files: cached_files.lock().unwrap().clone()
        };
        let cache_path = hachimi.get_data_path(REPO_CACHE_FILENAME);
        utils::write_json_file(&repo_cache, &cache_path)?;

        if let Some(mutex) = Gui::instance() {
            let mut gui = mutex.lock().unwrap();
            gui.show_notification(&t!("notification.update_completed"));
            if error_count > 0 {
                gui.show_notification(&t!("notification.errors_during_update", count = error_count));
            }
        }
        Ok(())
    }

    fn download_incremental(
        self: Arc<Self>,
        update_info: &UpdateInfo, localized_data_dir: &Path, cached_files: Arc<Mutex<FnvHashMap<String, String>>>
    ) -> Result<usize, Error> {
        let mut jobs_vec = Vec::with_capacity(NUM_THREADS);
        for _ in 0..NUM_THREADS {
            jobs_vec.push(DownloadJob::new());
        }
        let jobs = Arc::new(Mutex::new(jobs_vec));
        let pool = ThreadPool::new(NUM_THREADS);
        let current_size = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        let total_size = update_info.size;
        for repo_file in update_info.files.iter() {
            let repo_file_path = repo_file.path.clone();
            let file_path = repo_file.get_fs_path(localized_data_dir);
            let url = utils::concat_unix_path(&update_info.base_url, &repo_file.path);

            // Clone the Arcs for the closure
            let jobs = jobs.clone();
            let file_hash = repo_file.hash.clone();
            let updater = self.clone();
            let current_size = current_size.clone();
            let cached_files = cached_files.clone();
            let error_count = error_count.clone();

            pool.execute(move || {
                let mut job = { jobs.lock().unwrap().pop().expect("vacant job in job pool") };
                
                let res = job.execute(&file_path, &url, &file_hash, |read_bytes| {
                    let prev_size = current_size.fetch_add(read_bytes, atomic::Ordering::SeqCst);
                    updater.progress.store(Arc::new(Some(UpdateProgress::new(prev_size + read_bytes, total_size))));
                });

                match res {
                    Ok(hash) => { cached_files.lock().unwrap().insert(repo_file_path, hash); },
                    Err(e) => {
                        error!("{}", e);
                        error_count.fetch_add(1, atomic::Ordering::SeqCst);
                    }
                }

                // Return the job back to the pool
                jobs.lock().unwrap().push(job);
            });
        }

        // Wait for the thread pool to finish
        pool.join();

        Ok(error_count.load(atomic::Ordering::Relaxed))
    }

    fn download_zip(
        self: Arc<Self>,
        update_info: &UpdateInfo, localized_data_dir: &Path, cached_files: Arc<Mutex<FnvHashMap<String, String>>>
    ) -> Result<usize, Error> {
        let mut cached_files = cached_files.lock().unwrap();
        let mut error_count = 0;
        let zip_path = localized_data_dir.join(".tmp.zip");
        let sync_pool = ThreadPool::new(NUM_THREADS);

        { // block that drops the file objects so we can delete the temp file later
            let mut zip_file = fs::File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&zip_path)?;

            let res = ureq::get(&update_info.zip_url).call()?;
            let content_length = res.header("Content-Length")
                .map(|s| s.parse::<usize>().ok())
                .unwrap_or_default();
            let mut buffer = [0u8; CHUNK_SIZE];
            let mut downloaded = 0;
            http::download_file_buffered(res, &mut zip_file, &mut buffer, |bytes| {
                let progress = if let Some(len) = &content_length {
                    downloaded += bytes.len();
                    UpdateProgress::new(downloaded, *len)
                }
                else {
                    // fake progress
                    downloaded += 1;
                    UpdateProgress::new(downloaded, 100000)
                };
                self.progress.store(Arc::new(Some(progress)));
            })?;
            zip_file.sync_data()?;

            let mut zip_archive = zip::ZipArchive::new(zip_file)?;
            let mut hasher = blake3::Hasher::new();
            let mut current_bytes = 0;
            for repo_file in update_info.files.iter() {
                let archive_path = utils::concat_unix_path(&update_info.zip_dir, &repo_file.path);
                let mut archive_file = match zip_archive.by_name(&archive_path) {
                    Ok(v) => v,
                    Err(_) => {
                        error!("File not found in zip: {}", archive_path);
                        continue;
                    }
                };

                let path = repo_file.get_fs_path(localized_data_dir);
                if let Some(parent) = Path::new(&path).parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file = fs::File::create(&path)?;

                let mut buffer_pos = 0usize;
                loop {
                    let read_bytes = archive_file.read(&mut buffer[buffer_pos..])?;
            
                    let prev_buffer_pos = buffer_pos;
                    buffer_pos += read_bytes;
                    hasher.update(&buffer[prev_buffer_pos..buffer_pos]);

                    current_bytes += read_bytes;
                    self.progress.store(Arc::new(Some(UpdateProgress::new(current_bytes, update_info.size))));
            
                    if buffer_pos == buffer.len() {
                        buffer_pos = 0;
                        let written = file.write(&buffer)?;
                        if written != buffer.len() {
                            return Err(Error::OutOfDiskSpace);
                        }
                    }
            
                    if read_bytes == 0 {
                        break;
                    }
                }
            
                // Extract finished, flush the buffer
                if buffer_pos != 0 {
                    let written = file.write(&buffer[..buffer_pos])?;
                    if written != buffer_pos {
                        return Err(Error::OutOfDiskSpace);
                    }
                }
                sync_pool.execute(move || {
                    if let Err(e) = file.sync_data() {
                        error!("Failed to sync file: {}", e)
                    }
                });

                // Hash the file
                let hash = hasher.finalize().to_hex().to_string();
                if hash != repo_file.hash {
                    return Err(Error::FileHashMismatch(path.to_str().unwrap_or("").to_string()));
                }
                cached_files.insert(repo_file.path.clone(), hash);

                hasher.reset();
            }
        }

        // Wait for the sync pool to finish
        sync_pool.join();

        if let Err(e) = fs::remove_file(&zip_path) {
            error!("Failed to remove '{}': {}", zip_path.display(), e);
            error_count += 1;
        }

        Ok(error_count)
    }

    pub fn progress(&self) -> Option<UpdateProgress> {
        (**self.progress.load()).clone()
    }
}