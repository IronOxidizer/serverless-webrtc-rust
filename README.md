# serverless-webrtc-rust

Minimal Rust serverless WebRTC demo

Built with:
- rust
- actix-web
- actix-web-static-files
- webrtc-unreliable

Notes:
- WebRTC will not work with Firefox on 127.0.0.1
- Kill with killall -9 webrtc-serverless-rust
- Removes STUN and TURN servers by sending SDP data directly to WebRTC servers via AJAX