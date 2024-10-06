use std::sync::{Condvar, Mutex};

use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::il2cpp::{hook::umamusume::{StoryTimelineController, StoryTimelineData}, symbols::{IList, Thread}};

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

static STORY_GOTO_BLOCK_ID: Mutex<i32> = Mutex::new(0);
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
        Command::StoryGotoBlock { block_id } => {
            if block_id < -1 {
                return Ok(CommandResponse::error("Block ID cannot be smaller than -1".to_owned()));
            }

            let mut current_block_id = STORY_GOTO_BLOCK_ID.lock().unwrap();
            *current_block_id = block_id;

            Thread::main_thread().schedule(|| {
                let mut block_id = STORY_GOTO_BLOCK_ID.lock().unwrap();

                fn exec(block_id: i32) -> i32 {
                    let mut handle_guard = StoryTimelineController::CURRENT.lock().unwrap();
                    let Some(handle) = &*handle_guard else {
                        error!("No current StoryTimelineController");
                        return -3;
                    };

                    let controller = handle.target();
                    if controller.is_null() || StoryTimelineController::get_IsFinished(controller) {
                        *handle_guard = None;
                        error!("No current StoryTimelineController");
                        return -3;
                    }

                    let timeline_data = StoryTimelineController::get_TimelineData(controller);
                    if timeline_data.is_null() {
                        error!("TimelineData is NULL");
                        return -3;
                    }

                    let Some(block_list) = <IList>::new(StoryTimelineData::get_BlockList(timeline_data)) else {
                        return -3;
                    };

                    let count = block_list.count();
                    if block_id >= count {
                        error!("Block ID out of range (max: {})", count - 1);
                        return -3;
                    }

                    StoryTimelineController::GotoBlock_orig(controller, block_id, false, false, false);
                    -2
                }

                // Notify that it has finished
                *block_id = exec(*block_id);
                STORY_GOTO_BLOCK_CVAR.notify_one();
            });

            // Block until thread finishes
            while *current_block_id > -2 {
                current_block_id = STORY_GOTO_BLOCK_CVAR.wait(current_block_id).unwrap();
            }

            if *current_block_id == -3 {
                return Ok(CommandResponse::error(None));
            }
        },

        Command::ReloadLocalizedData => {
            Hachimi::instance().load_localized_data();
            if let Some(mutex) = Gui::instance() {
                mutex.lock().unwrap().show_notification("Localized data reloaded.");
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
        block_id: i32
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