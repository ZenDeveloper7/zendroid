use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidDevice {
    pub serial: String,
    pub state: String,
    pub description: String,
}

pub fn discover_devices() -> Result<Vec<AndroidDevice>, String> {
    let adb = resolve_adb().ok_or_else(|| "adb not found on PATH or in Android SDK".to_string())?;
    let output = Command::new(&adb)
        .arg("devices")
        .arg("-l")
        .output()
        .map_err(|err| format!("failed to run {} devices -l: {err}", adb.display()))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(parse_devices(&String::from_utf8_lossy(&output.stdout)))
}

pub fn resolve_adb() -> Option<PathBuf> {
    if let Ok(path) = env::var("PATH") {
        for entry in env::split_paths(&path) {
            let candidate = entry.join("adb");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    for key in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(path) = env::var(key) {
            let candidate = PathBuf::from(path).join("platform-tools").join("adb");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    dirs::home_dir()
        .map(|home| {
            home.join("Android")
                .join("Sdk")
                .join("platform-tools")
                .join("adb")
        })
        .filter(|candidate| candidate.is_file())
}

pub fn parse_devices(raw: &str) -> Vec<AndroidDevice> {
    raw.lines()
        .skip(1)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let mut parts = trimmed.split_whitespace();
            let serial = parts.next()?.to_string();
            let state = parts.next()?.to_string();
            let description = parts.collect::<Vec<_>>().join(" ");
            Some(AndroidDevice {
                serial,
                state,
                description,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adb_device_output() {
        let raw = "\
List of devices attached
emulator-5554 device product:sdk_gphone model:Pixel_8 device:emu64x transport_id:1
abc123 offline usb:1-2 product:foo model:Phone device:bar
";

        let devices = parse_devices(raw);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].serial, "emulator-5554");
        assert_eq!(devices[0].state, "device");
        assert!(devices[0].description.contains("Pixel_8"));
    }
}
