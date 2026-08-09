use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;

use futures_util::StreamExt;
use livekit::track::RemoteVideoTrack;
use livekit::webrtc::video_frame::native::VideoFrameBufferExt;
use livekit::webrtc::video_frame::VideoFormatType;
use livekit::webrtc::video_stream::native::NativeVideoStream;
use tauri::{AppHandle, Emitter};

use super::client::is_current;
use super::events::{VideoPayload, VIDEO_EVENT};
use super::frames;

static IS_STREAMING: AtomicBool = AtomicBool::new(false);

/// Bumped whenever the displayed track changes, so a frame loop for a track we have moved
/// off stops without having to be cancelled directly.
static STREAM_EPOCH: AtomicU64 = AtomicU64::new(0);
static CURRENT_PRIORITY: AtomicU8 = AtomicU8::new(0);
static CURRENT_SID: Mutex<Option<String>> = Mutex::new(None);

/// A shared screen always wins over a camera, so plugging in a presentation takes the
/// display back from whoever was pointing a phone at the room.
pub const PRIORITY_CAMERA: u8 = 1;
pub const PRIORITY_SCREEN: u8 = 2;

/// Consumes decoded frames from the subscribed screen share.
///
/// Frames are converted to BGRA, which on a little endian machine is the same layout as
/// cairo's ARgb32, and handed to the native drawing surface sitting under the webview.
pub fn start(app: AppHandle, generation: u64, track: RemoteVideoTrack, priority: u8, sid: String) {
    // A camera must not displace a screen share that is already on the display.
    if IS_STREAMING.load(Ordering::SeqCst) && CURRENT_PRIORITY.load(Ordering::SeqCst) > priority {
        return;
    }

    let epoch = STREAM_EPOCH.fetch_add(1, Ordering::SeqCst) + 1;
    CURRENT_PRIORITY.store(priority, Ordering::SeqCst);
    IS_STREAMING.store(true, Ordering::SeqCst);

    if let Ok(mut guard) = CURRENT_SID.lock() {
        *guard = Some(sid);
    }

    tauri::async_runtime::spawn(async move {
        let mut stream = NativeVideoStream::new(track.rtc_track());
        let mut last_size = (0u32, 0u32);

        // Reused across frames so a sixty frame per second stream does not allocate
        // several megabytes per frame.
        let mut rgba = Vec::new();
        let mut rgb = Vec::new();
        let mut encoded = Vec::new();

        while let Some(frame) = stream.next().await {
            if STREAM_EPOCH.load(Ordering::SeqCst) != epoch || !is_current(generation) {
                return;
            }

            let width = frame.buffer.width();
            let height = frame.buffer.height();
            if width == 0 || height == 0 {
                continue;
            }

            if (width, height) != last_size {
                last_size = (width, height);
                let _ = app.emit(
                    VIDEO_EVENT,
                    VideoPayload {
                        active: true,
                        width,
                        height,
                    },
                );
            }

            let Some((target_width, target_height)) = frames::next_target(width, height) else {
                continue;
            };

            // to_argb converts, it does not resample, so the buffer has to be scaled
            // first. Doing it in I420 is cheaper than scaling four channels of RGBA.
            let mut i420 = frame.buffer.to_i420();
            let source = if (target_width, target_height) == (width, height) {
                i420
            } else {
                i420.scale(target_width as i32, target_height as i32)
            };

            let stride = target_width * 4;
            rgba.resize((stride * target_height) as usize, 0);

            // libyuv names formats after the 32 bit word, not the byte order. On little
            // endian its "RGBA" lands in memory as A,B,G,R, so taking the first three
            // bytes as red, green and blue picked up alpha as red and tinted everything.
            // "ABGR" is the one that lays out as R,G,B,A in memory.
            source.to_argb(
                VideoFormatType::ABGR,
                &mut rgba,
                stride,
                target_width as i32,
                target_height as i32,
            );

            frames::send(&rgba, &mut rgb, &mut encoded, target_width, target_height);
        }

        // The stream ended on its own rather than being replaced.
        if STREAM_EPOCH.load(Ordering::SeqCst) == epoch && is_current(generation) {
            stop(&app);
        }
    });
}

/// Clears the display only if the track that went away is the one being shown. A camera
/// leaving must not blank a screen share that took over from it.
pub fn stop_track(app: &AppHandle, sid: &str) {
    let is_current_track = CURRENT_SID
        .lock()
        .map(|guard| guard.as_deref() == Some(sid))
        .unwrap_or(false);

    if is_current_track {
        stop(app);
    }
}

pub fn stop(app: &AppHandle) {
    STREAM_EPOCH.fetch_add(1, Ordering::SeqCst);
    CURRENT_PRIORITY.store(0, Ordering::SeqCst);

    if let Ok(mut guard) = CURRENT_SID.lock() {
        *guard = None;
    }

    if !IS_STREAMING.swap(false, Ordering::SeqCst) {
        return;
    }

    let _ = app.emit(
        VIDEO_EVENT,
        VideoPayload {
            active: false,
            width: 0,
            height: 0,
        },
    );
}
