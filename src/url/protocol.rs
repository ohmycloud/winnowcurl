#[derive(Debug)]
pub enum Protocol {
    HTTPS,
    HTTP,
    FTP,
    SFTP,
    TFTP,
    TELNET,
    LDAP,
    WS,
    WSS,
    UNKNOWN,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::HTTPS
    }
}

impl From<&str> for Protocol {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "https" => Protocol::HTTPS,
            "http" => Protocol::HTTP,
            "ftp" => Protocol::FTP,
            "sftp" => Protocol::SFTP,
            "tftp" => Protocol::TFTP,
            "telnet" => Protocol::TELNET,
            "ldap" => Protocol::LDAP,
            "ws" => Protocol::WS,
            "wss" => Protocol::WSS,
            _ => Protocol::UNKNOWN,
        }
    }
}
