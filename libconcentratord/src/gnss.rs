use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Device {
    #[default]
    None,
    TtyPath(String),
    Gpsd(String),
}

impl Device {
    pub fn new(path: &str) -> Device {
        if path.is_empty() {
            return Device::None;
        }

        if let Some(host) = path.strip_prefix("gpsd://") {
            return Device::Gpsd(host.to_string());
        }

        Device::TtyPath(path.to_string())
    }
}

impl<'de> Deserialize<'de> for Device {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Device::new(&s))
    }
}

impl Serialize for Device {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&match self {
            Device::None => "".to_string(),
            Device::TtyPath(v) => v.to_string(),
            Device::Gpsd(v) => format!("gpsd://{}", v),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_device_none() {
        assert_eq!(Device::None, Device::new(""));
    }

    #[test]
    fn test_device_tty_path() {
        assert_eq!(
            Device::TtyPath("/dev/ttyAMA0".to_string()),
            Device::new("/dev/ttyAMA0")
        );
    }

    #[test]
    fn test_device_gpsd() {
        assert_eq!(
            Device::Gpsd("localhost:2947".to_string()),
            Device::new("gpsd://localhost:2947")
        );
    }
}
