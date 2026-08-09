# Spike: video performance on the Pi Zero 2W

The original question was whether WebKitGTK could reach hardware H.264 decode. That turned
out to be moot: WebKitGTK ships without WebRTC at all, so the kiosk cannot receive media in
the webview.

The kiosk now joins the room from Rust with the native LiveKit SDK. Decoding happens there,
each frame is scaled and JPEG encoded, and the result is pushed to the webview to be drawn
on a canvas. The question this spike answers has therefore changed: can a Zero 2W decode the
incoming stream **and** re-encode it fast enough to look live.

The decode half is still worth measuring with the commands below, because libwebrtc uses the
same V4L2 path. The encode half is measured from the app itself, by lowering
`PISTATION_VIDEO_MAX_WIDTH`, `PISTATION_VIDEO_FPS` and `PISTATION_VIDEO_QUALITY` until the
picture keeps up.

## Baseline

The Zero 2W has a VideoCore IV class block exposed through V4L2 stateful decode at
`/dev/video10`, reached through GStreamer's `v4l2h264dec`.

## Steps

1. Confirm the kernel exposes the decoder.

   ```
   ls -l /dev/video10 /dev/video11 /dev/video12
   v4l2-ctl -d /dev/video10 --all | head -40
   ```

   `/dev/video10` must report `Codec Decoder` capabilities.

2. Confirm GStreamer picks the hardware element over the software one.

   ```
   gst-inspect-1.0 v4l2h264dec
   gst-launch-1.0 videotestsrc num-buffers=600 ! video/x-raw,width=1280,height=720 \
     ! x264enc tune=zerolatency ! h264parse ! v4l2h264dec ! fpsdisplaysink text-overlay=false
   ```

   Record the reported average FPS. Under 25 means the decode path is not viable.

3. Run the same clip through the software decoder for comparison.

   ```
   gst-launch-1.0 videotestsrc num-buffers=600 ! video/x-raw,width=1280,height=720 \
     ! x264enc tune=zerolatency ! h264parse ! avdec_h264 ! fpsdisplaysink text-overlay=false
   ```

   If step 2 and step 3 report similar numbers, WebKit is almost certainly using
   software decode and the hardware element is not being selected.

4. Measure the real path. Start the kiosk, join from a laptop, share a 1280x720
   screen, and sample CPU while the stream is live.

   ```
   systemctl start pistation-kiosk
   top -b -n 30 -d 1 | grep -E "WebKitWebProcess|pistation-kiosk"
   ```

   Sustained WebKitWebProcess CPU above roughly 85 percent of one core means the
   decode is running in software.

5. Force the issue if step 4 is inconclusive.

   ```
   GST_DEBUG=v4l2*:5 systemctl start pistation-kiosk
   journalctl -u pistation-kiosk | grep -i v4l2h264dec
   ```

## Recording the result

| Measurement | Value | Pass threshold |
| --- | --- | --- |
| `v4l2h264dec` 720p30 FPS | | 25 or higher |
| WebKitWebProcess CPU, 720p live share | | under 85 percent of one core |
| Frames dropped over 5 minutes | | under 1 percent |
| Annotation input to on-screen latency | | under 250 ms |

## If the spike fails

1. Try 960x540 at 20 fps by capping the publisher in `apps/web-client` before
   declaring failure. The Zero 2W often passes at that resolution.
2. If it still fails, build `services/compositor-bot`. GStreamer composites the
   screen share and the annotation canvas server side and sends the Pi one stream,
   which drops the Pi's job to plain video playback.
