include!("../build.rs");

#[cfg(test)]
mod tests {
  use super::*;
  use std::{fs, process::Command};

  #[test]
  fn preparation_preserves_native_branding_across_debug_and_release() {
    const WORKER: &str = "TAURI_ELECTRON_BRANDING_TEST";
    if env::var_os(WORKER).is_some() {
      prepare().unwrap();
      return;
    }
    let root = env::temp_dir().join(format!("tauri-electron-branding-{}", std::process::id()));
    let app = root.join("src-tauri");
    let target = app.join("target");
    let distribution = target.join("electron/linux-x64");
    fs::create_dir_all(&distribution).unwrap();
    fs::write(app.join("Cargo.toml"), "[package]\nname = 'branding-fixture'\nversion = '0.1.0'\nedition = '2021'\n[workspace]\n[[bin]]\nname = 'fixture'\npath = 'main.rs'\n").unwrap();
    fs::write(app.join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(app.join("tauri.conf.json"), r#"{"productName":"WaterPlusAI","identifier":"com.shuiwujia.waterbuddy"}"#).unwrap();
    let mut executable = vec![0u8; 64];
    executable[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    executable[16..18].copy_from_slice(&2u16.to_le_bytes());
    executable[18..20].copy_from_slice(&62u16.to_le_bytes());
    fs::write(distribution.join("electron"), &executable).unwrap();
    fs::write(distribution.join("version"), electron_rs::DEFAULT_VERSION).unwrap();

    for profile in ["debug", "release", "debug"] {
      let out = target.join(profile).join("build/fixture/out");
      fs::create_dir_all(&out).unwrap();
      let output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "tests::preparation_preserves_native_branding_across_debug_and_release", "--nocapture"])
        .env(WORKER, "1").env("PROFILE", profile).env("TARGET", "x86_64-unknown-linux-gnu")
        .env("OUT_DIR", &out).env("INIT_CWD", &root).env("PWD", &root)
        .env_remove("TAURI_CONFIG").env_remove("CARGO_TARGET_DIR").current_dir(&root)
        .output().unwrap();
      assert!(output.status.success(), "{profile}: {}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
      assert!(distribution.join("WaterPlusAI").is_file(), "{profile} must prepare the WaterPlusAI executable");
      assert_eq!(fs::read(distribution.join("WaterPlusAI")).unwrap(), executable, "{profile} must preserve the branded executable");
      assert!(!distribution.join("electron").exists(), "{profile} must not restore the stock Electron name");
      let marker: serde_json::Value = serde_json::from_slice(&fs::read(distribution.join("electron-rs.json")).unwrap()).unwrap();
      assert_eq!(marker["pe"], true, "{profile} must apply native branding");
    }
    fs::remove_dir_all(root).unwrap();
  }
}
