use std::sync::{Condvar, Mutex};

use rust_i18n::t;
use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::{core::utils::notify_error, il2cpp::{hook::umamusume::{StoryTimelineController, StoryTimelineData}, symbols::{IList, Thread}}};

use super::{Error, Gui, Hachimi};

pub fn start_http(listen_all: bool) {
    std::thread::spawn(move || http_thread(listen_all));
}

fn http_thread(listen_all: bool) {
    let address = if listen_all {
        "0.0.0.0:50433"
    }
    else {
        "127.0.0.1:50433"
    };

    let server = match Server::http(address) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to start HTTP server: {}", e);
            return;
        }
    };

    info!("IPC server listening on {}", address);

    for mut request in server.incoming_requests() {
        let command_response = match on_http_request(&mut request) {
            Ok(v) => v,
            Err(e) => {
                error!("Error occurred while processing command: {}", e);
                CommandResponse::error(e.to_string())
            },
        };

        let response_data = serde_json::to_string(&command_response).unwrap_or_else(|_|
            serde_json::to_string(&CommandResponse::error(
                "Failed to encode response".to_owned()
            )).unwrap()
        );

        if let Err(e) = request.respond(
            Response::from_string(response_data)
                .with_header(Header::from_bytes("content-type", "application/json").unwrap())
                .with_status_code(
                    match command_response {
                        CommandResponse::Error { .. } => 400,
                        _ => 200
                    }
                )
        ) {
            error!("Failed to send HTTP response: {}", e);
        }
    }
}

static STORY_GOTO_BLOCK_PARAMS: Mutex<(i32, bool)> = Mutex::new((0, false));
static STORY_GOTO_BLOCK_CVAR: Condvar = Condvar::new();

fn on_http_request(request: &mut Request) -> Result<CommandResponse, Error> {
    let method = request.method();
    if *method == Method::Get {
        return Ok(CommandResponse::HelloWorld { message: "Hachimi's IPC server is working!" });
    }
    else if *method != Method::Post {
        return Ok(CommandResponse::error("Invalid request method".to_owned()));
    }

    let headers = Headers { headers: request.headers() };
    if !headers.get("content-type").map(|t| t.eq_ignore_ascii_case("application/json")).unwrap_or(false) {
        return Ok(CommandResponse::error("Invalid content type".to_owned()));
    }

    let command: Command = serde_json::from_reader(request.as_reader())?;
    match command {
        Command::StoryGotoBlock { block_id, incremental } => {
            if block_id < -1 {
                return Ok(CommandResponse::error("Block ID cannot be smaller than -1".to_owned()));
            }

            let mut params = STORY_GOTO_BLOCK_PARAMS.lock().unwrap();
            *params = (block_id, incremental);

            Thread::main_thread().schedule(|| {
                let (ref mut block_id, incremental) = *STORY_GOTO_BLOCK_PARAMS.lock().unwrap();

                fn exec(block_id: i32, incremental: bool) -> i32 {
                    let mut handle_guard = StoryTimelineController::CURRENT.lock().unwrap();
                    let Some(controller) = (*handle_guard).as_ref()
                        .map(|h| h.target())
                        .filter(|c| !c.is_null() && !StoryTimelineController::get_IsFinished(*c))
                    else {
                        *handle_guard = None;
                        notify_error("No current StoryTimelineController");
                        return -3;
                    };
                    drop(handle_guard);

                    let timeline_data = StoryTimelineController::get_TimelineData(controller);
                    if timeline_data.is_null() {
                        notify_error("TimelineData is NULL");
                        return -3;
                    }

                    let Some(block_list) = <IList>::new(StoryTimelineData::get_BlockList(timeline_data)) else {
                        return -3;
                    };

                    let count = block_list.count();
                    if block_id >= count {
                        notify_error(format!("Block ID out of range (max: {})", count - 1));
                        return -3;
                    }

                    if incremental && block_id != -1 {
                        let last_block_id = StoryTimelineController::last_block_id();
                        let start = if last_block_id > block_id { 0 } else { last_block_id + 1 };
                        for i in start..=block_id {
                            StoryTimelineController::GotoBlock(controller, i, false, false, false);
                        }
                    }
                    else {
                        StoryTimelineController::GotoBlock(controller, block_id, false, false, false);
                    }
                    -2
                }

                // Notify that it has finished
                *block_id = exec(*block_id, incremental);
                STORY_GOTO_BLOCK_CVAR.notify_one();
            });

            // Block until thread finishes
            while (*params).0 > -2 {
                params = STORY_GOTO_BLOCK_CVAR.wait(params).unwrap();
            }

            if (*params).0 == -3 {
                return Ok(CommandResponse::error(None));
            }
        },

        Command::ReloadLocalizedData => {
            Hachimi::instance().load_localized_data();
            if let Some(mutex) = Gui::instance() {
                mutex.lock().unwrap().show_notification(&t!("notification.localized_data_reloaded"));
            }
        }
    }

    Ok(CommandResponse::Ok)
}

struct Headers<'a> {
    headers: &'a [Header]
}

impl<'a> Headers<'a> {
    fn get(&self, name: &'static str) -> Option<&'a str> {
        for header in self.headers {
            if header.field.equiv(name) {
                return Some(header.value.as_str());
            }
        }
    
        None
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Command {
    StoryGotoBlock {
        block_id: i32,
        #[serde(default)]
        incremental: bool
    },

    ReloadLocalizedData
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum CommandResponse {
    Ok,

    Error {
        message: Option<String>
    },

    HelloWorld {
        message: &'static str
    }
}

impl CommandResponse {
    fn error(message: impl Into<Option<String>>) -> Self {
        Self::Error { message: message.into() }
    }
}