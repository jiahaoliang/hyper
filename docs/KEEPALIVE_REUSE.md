# Keep-alive reuse timeout (fork extension)

This branch starts at hyper v1.8.1 (`166c6cacc74b215674937e782b3ab2cbd8b69883`).
The `upstream/v1.8.1` branch preserves that exact upstream snapshot. The h2
dependency is pinned to 0.4.13; this feature does not change the h2 protocol engine.

`client::conn::http2::Builder::keep_alive_reuse_timeout(Some(duration))` adds an
optional notification deadline to the existing keep-alive PING timeout phase.
Install a per-connection `rt::KeepAliveObserver` using `keep_alive_observer`.
The callback should permanently remove the connection from the caller's pool.
Hyper does not change the readiness or behavior of direct `SendRequest` handles.

The soft and hard deadlines start together, when the existing keep-alive state
machine enters its PING timeout phase. This is submission to h2, not proof of
transmission on the wire. Adaptive-window PINGs alone do not start a soft timer.
The PING interval is based on read inactivity, not a fixed wall-clock cadence.

- Default `None` preserves existing behavior.
- With keep-alive disabled the reuse timeout is inactive.
- An active reuse timeout must satisfy `0 < soft < hard`. Handshake checks the
  final builder configuration so setter order does not matter.
- A ready ACK takes priority over the timers. Receiving it cancels the soft wait.
- If hard and soft deadlines are both expired when polled, hard timeout wins.
- Soft expiry takes and invokes the observer once, after releasing the PING lock.
- It does not send GOAWAY/RST_STREAM, close IO, or cancel any stream. The original
  hard timer remains registered and retains its original deadline.
- A late ACK allows existing streams to progress but does not undo retirement.

For the hyper-util legacy pool, configure interval 10 seconds, reuse timeout
5 seconds, hard timeout 60 seconds, and keep-alive while idle. The hard timeout
is still configured by the application; hyper's existing 20-second default has
not changed. Requests already checked out can race with retirement. There is no
proactive reconnect, automatic stream migration, or new replay policy.

Validation lives in `src/proto/h2/ping/tests.rs` and in the companion hyper-util
fork's `tests/keepalive_reuse.rs`. The latter pauses both directions of a real H2
connection over in-memory IO without EOF/reset, permits replacement connections,
and restores the original connection before its hard deadline.
