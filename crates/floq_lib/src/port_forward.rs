// port_forward — SSH-gate + token auth + local TcpListener + pipe to tunnel
// (architecture §14.2). read_ssh_client_banner -> SshGateRejection.
pub enum SshGateRejection {}
