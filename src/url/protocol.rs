#[derive(Debug)]
pub enum Schema {
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

impl Default for Schema {
    fn default() -> Self {
        Self::HTTPS
    }
}

impl From<&str> for Schema {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "https" => Schema::HTTPS,
            "http" => Schema::HTTP,
            "ftp" => Schema::FTP,
            "sftp" => Schema::SFTP,
            "tftp" => Schema::TFTP,
            "telnet" => Schema::TELNET,
            "ldap" => Schema::LDAP,
            "ws" => Schema::WS,
            "wss" => Schema::WSS,
            _ => Schema::UNKNOWN,
        }
    }
}
