use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use livekit::track::{RemoteTrack, TrackSource};
use livekit::{Room, RoomEvent, RoomOptions};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use super::events::{
    DataPayload, ParticipantView, ParticipantsPayload, StatusPayload, DATA_EVENT,
    PARTICIPANTS_EVENT, STATUS_EVENT,
};
use super::video;

/// Bumped on every connect attempt. Tasks belonging to a superseded connection compare
/// against this before emitting anything, so a room we deliberately replaced cannot
/// report itself as disconnected and trigger a pointless reconnect.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn is_current(generation: u64) -> bool {
    GENERATION.load(Ordering::SeqCst) == generation
}

#[derive(Default)]
pub struct RoomHandle {
    room: Mutex<Option<Arc<Room>>>,
}

impl RoomHandle {
    pub async fn connect(&self, app: AppHandle, url: String, token: String) -> Result<(), String> {
        let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

        self.close_current().await;
        video::stop(&app);

        emit_status(&app, generation, StatusPayload::new("connecting"));

        let (room, mut events) = Room::connect(&url, &token, RoomOptions::default())
            .await
            .map_err(|error| error.to_string())?;

        if !is_current(generation) {
            let _ = room.close().await;
            return Ok(());
        }

        *self.room.lock().await = Some(Arc::new(room));
        emit_status(&app, generation, StatusPayload::new("connected"));

        let room_for_task = self.room.lock().await.clone();

        tauri::async_runtime::spawn(async move {
            if let Some(room) = room_for_task.as_ref() {
                emit_participants(&app, generation, room);
            }

            while let Some(event) = events.recv().await {
                if !is_current(generation) {
                    return;
                }

                if matches!(
                    event,
                    RoomEvent::ParticipantConnected(_) | RoomEvent::ParticipantDisconnected(_)
                ) {
                    if let Some(room) = room_for_task.as_ref() {
                        emit_participants(&app, generation, room);
                    }
                }

                handle_event(&app, generation, event);
            }

            if is_current(generation) {
                video::stop(&app);
                emit_status(&app, generation, StatusPayload::new("disconnected"));
            }
        });

        Ok(())
    }

    pub async fn disconnect(&self) {
        GENERATION.fetch_add(1, Ordering::SeqCst);
        self.close_current().await;
    }

    async fn close_current(&self) {
        let existing = self.room.lock().await.take();
        if let Some(room) = existing {
            let _ = room.close().await;
        }
    }
}

fn handle_event(app: &AppHandle, generation: u64, event: RoomEvent) {
    match event {
        RoomEvent::Connected { .. } => emit_status(app, generation, StatusPayload::new("connected")),

        RoomEvent::Reconnecting => {
            emit_status(app, generation, StatusPayload::new("reconnecting"))
        }

        RoomEvent::Reconnected => emit_status(app, generation, StatusPayload::new("connected")),

        RoomEvent::Disconnected { reason } => {
            video::stop(app);
            emit_status(
                app,
                generation,
                StatusPayload::with_detail("disconnected", format!("{reason:?}")),
            );
        }

        RoomEvent::DataReceived { payload, .. } => {
            if let Ok(envelope) = String::from_utf8(payload.to_vec()) {
                let _ = app.emit(DATA_EVENT, DataPayload { envelope });
            }
        }

        RoomEvent::TrackSubscribed {
            track, publication, ..
        } => {
            let priority = match publication.source() {
                TrackSource::Screenshare => video::PRIORITY_SCREEN,
                TrackSource::Camera => video::PRIORITY_CAMERA,
                _ => return,
            };

            if let RemoteTrack::Video(video_track) = track {
                video::start(
                    app.clone(),
                    generation,
                    video_track,
                    priority,
                    publication.sid().to_string(),
                );
            }
        }

        RoomEvent::TrackUnsubscribed { publication, .. } => {
            video::stop_track(app, &publication.sid().to_string());
        }

        _ => {}
    }
}

fn emit_participants(app: &AppHandle, generation: u64, room: &Room) {
    if !is_current(generation) {
        return;
    }

    let participants = room
        .remote_participants()
        .values()
        .map(|participant| {
            let identity = participant.identity().to_string();
            let name = participant.name();

            ParticipantView {
                display_name: if name.is_empty() {
                    identity.clone()
                } else {
                    name
                },
                identity,
            }
        })
        .collect();

    let _ = app.emit(PARTICIPANTS_EVENT, ParticipantsPayload { participants });
}

fn emit_status(app: &AppHandle, generation: u64, payload: StatusPayload) {
    if !is_current(generation) {
        return;
    }
    let _ = app.emit(STATUS_EVENT, payload);
}
