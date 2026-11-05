// raw_websocket — named WebSocket connections + `{"type","payload"}` framing
// (architecture §9b). tokio-tungstenite over PrefixedTcpStream + native-tls.
pub struct RawWebSocketManager;
pub enum RawWebSocketCommand {}
