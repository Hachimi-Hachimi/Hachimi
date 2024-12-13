use std::{fs::File, sync::{atomic::{self, AtomicBool}, Arc, Mutex}};

use arc_swap::ArcSwap;
use serde::Deserialize;
use widestring::Utf16Str;
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::MAX_PATH, System::LibraryLoader::GetModuleFileNameW,
        UI::{Shell::ShellExecuteW, WindowsAndMessaging::{PostMessageW, SW_NORMAL, WM_CLOSE}}
    }
};

use crate::core::{gui::{PersistentMessageWindow, SimpleYesNoDialog}, http, Error, Gui, Hachimi};

use super::{gui_impl::render_hook, main::DLL_HMODULE, utils};

const REPO_PATH: &str = "Hachimi-Hachimi/Hachimi";

#[derive(Default)]
pub struct Updater {
    update_check_mutex: Mutex<()>,
    new_update: ArcSwap<Option<ReleaseAsset>>
}

impl Updater {
    pub fn check_for_updates(self: Arc<Self>, callback: fn(bool)) {
        std::thread::spawn(move || {
            match self.check_for_updates_internal() {
                Ok(v) => callback(v),
                Err(e) => error!("{}", e)
            }
        });
    }

    fn check_for_updates_internal(&self) -> Result<bool, Error> {
        // Prevent multiple update checks running at the same time
        let Ok(_guard) = self.update_check_mutex.try_lock() else {
            return Ok(false);
        };

        if let Some(mutex) = Gui::instance() {
            mutex.lock().unwrap().show_notification("Checking for updates...");
        }

        let latest: Release = http::get_json(&format!("https://api.github.com/repos/{}/releases/latest", REPO_PATH))?;
        if latest.is_different_version() {
            let mut installer_asset = None;
            for asset in latest.assets {
                if asset.name == "hachimi_installer.exe" {
                    installer_asset = Some(asset);
                    break;
                }
            }

            if installer_asset.is_some() {
                self.new_update.store(Arc::new(installer_asset));
                if let Some(mutex) = Gui::instance() {
                    mutex.lock().unwrap().show_window(Box::new(SimpleYesNoDialog::new(
                        "New update available",
                        &format!("A new Hachimi update is available ({}). Do you want to install it?\n\
                                  The game will be restarted in order to apply the update.", latest.tag_name),
                        |ok| {
                            if !ok { return; }
                            Hachimi::instance().updater.clone().run();
                        }
                    )));
                }
                return Ok(true);
            }
        }
        else if let Some(mutex) = Gui::instance() {
            mutex.lock().unwrap().show_notification("No updates available.");
        }

        Ok(false)
    }

    pub fn run(self: Arc<Self>) {
        std::thread::spawn(move || {
            let dialog_show = Arc::new(AtomicBool::new(true));
            if let Some(mutex) = Gui::instance() {
                mutex.lock().unwrap().show_window(Box::new(PersistentMessageWindow::new(
                    "Updating",
                    "Downloading update, the game will restart shortly...",
                    dialog_show.clone()
                )));
            }

            if let Err(e) = self.clone().run_internal() {
                error!("{}", e);
                if let Some(mutex) = Gui::instance() {
                    mutex.lock().unwrap().show_notification(&("Update failed: ".to_owned() + &e.to_string()));
                }
            }

            dialog_show.store(false, atomic::Ordering::Relaxed)
        });
    }

    fn run_internal(self: Arc<Self>) -> Result<(), Error> {
        let Some(ref asset) = **self.new_update.load() else {
            return Ok(());
        };
        self.new_update.store(Arc::new(None));

        // Download the installer
        let installer_path = utils::get_tmp_installer_path();

        let res = ureq::get(&asset.browser_download_url).call()?;
        std::io::copy(&mut res.into_reader(), &mut File::create(&installer_path)?)?;

        // Launch the installer
        let mut slice = [0u16; MAX_PATH as usize];
        let length = unsafe { GetModuleFileNameW(DLL_HMODULE, &mut slice) } as usize;
        let hachimi_path_str = unsafe { Utf16Str::from_slice_unchecked(&slice[..length]) };
        unsafe {
            ShellExecuteW(
                None,
                None,
                &HSTRING::from(installer_path.into_os_string()),
                &HSTRING::from(format!(
                    "install --target \"{}\" --sleep 1000 --prompt-for-game-exit --launch-game -- {}",
                    hachimi_path_str, std::env::args().skip(1).collect::<Vec<String>>().join(" ")
                )),
                PCWSTR::from_raw(slice.as_ptr()),
                SW_NORMAL
            );

            // Close the game
            _ = PostMessageW(render_hook::get_swap_chain_hwnd(), WM_CLOSE, None, None);
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Release {
    // STUB
    tag_name: String,
    assets: Vec<ReleaseAsset>
}

impl Release {
    pub fn is_different_version(&self) -> bool {
        self.tag_name != format!("v{}", env!("CARGO_PKG_VERSION"))
    }
}

#[derive(Deserialize)]
pub struct ReleaseAsset {
    // STUB
    name: String,
    browser_download_url: String
}