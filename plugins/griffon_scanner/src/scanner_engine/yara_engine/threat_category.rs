use clap::ValueEnum;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
pub enum ThreatCategory {
    Ransomware,
    Trojan,
    Backdoor,
    Cryptominer,
    Webshell,
    Rootkit,
    Spyware,
    Apt,
    Other,
}

impl ThreatCategory {
    #[allow(dead_code)]
    pub fn all() -> &'static [ThreatCategory] {
        &[
            ThreatCategory::Ransomware,
            ThreatCategory::Trojan,
            ThreatCategory::Backdoor,
            ThreatCategory::Cryptominer,
            ThreatCategory::Webshell,
            ThreatCategory::Rootkit,
            ThreatCategory::Spyware,
            ThreatCategory::Apt,
            ThreatCategory::Other,
        ]
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ransomware => "ransomware",
            Self::Trojan => "trojans",
            Self::Backdoor => "backdoors",
            Self::Cryptominer => "cryptominers",
            Self::Webshell => "webshells",
            Self::Rootkit => "rootkits",
            Self::Spyware => "spyware",
            Self::Apt => "apt",
            Self::Other => "other",
        }
    }

    #[allow(dead_code)]
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ransomware" => Some(Self::Ransomware),
            "trojans" => Some(Self::Trojan),
            "backdoors" => Some(Self::Backdoor),
            "cryptominers" => Some(Self::Cryptominer),
            "webshells" => Some(Self::Webshell),
            "rootkits" => Some(Self::Rootkit),
            "spyware" => Some(Self::Spyware),
            "apt" => Some(Self::Apt),
            _ => None,
        }
    }
}

impl std::str::FromStr for ThreatCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ransomware" => Ok(Self::Ransomware),
            "trojans" => Ok(Self::Trojan),
            "backdoors" => Ok(Self::Backdoor),
            "cryptominers" => Ok(Self::Cryptominer),
            "webshells" => Ok(Self::Webshell),
            "rootkits" => Ok(Self::Rootkit),
            "spyware" => Ok(Self::Spyware),
            "apt" => Ok(Self::Apt),
            other => Err(format!("Unknown category: '{}'", other)),
        }
    }
}
