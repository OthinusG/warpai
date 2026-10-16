use super::*;

#[test]
fn language_restart_selects_original_image_only_for_its_mount() {
    let root = tempfile::tempdir().unwrap();
    let mount = root.path().join(".mount_Warpai");
    std::fs::create_dir(&mount).unwrap();
    let executable = mount.join("opt/warpai/warpai");
    let image = root.path().join("Warpai ' $HOME 中文.AppImage");
    std::fs::write(&image, b"image fixture").unwrap();
    let image_env = Some(image.clone().into_os_string());
    let mount_env = Some(mount.into_os_string());
    assert_eq!(language_restart_target(&executable, image_env.clone(), mount_env.clone()).unwrap(), image);
    assert!(language_restart_target(&executable, None, mount_env.clone()).is_err());
    assert!(language_restart_target(&executable, Some("relative.AppImage".into()), mount_env.clone()).is_err());
    let native = root.path().join("native/warpai");
    assert_eq!(language_restart_target(&native, image_env, mount_env).unwrap(), native);
    std::fs::remove_file(&image).unwrap();
    assert!(language_restart_target(&executable, Some(image.into_os_string()), Some(root.path().join(".mount_Warpai").into_os_string())).is_err());
}

#[test]
fn language_restart_waits_for_previous_process_before_executing_literal_path() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("Warpai ' $HOME 中文");
    std::fs::write(&executable, "#!/bin/sh\ntouch -- \"$(dirname -- \"$0\")/restarted\"\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut previous = command::blocking::Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let mut restart = super::super::language_restart_command(&executable, previous.id()).spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(250));
    assert!(!root.path().join("restarted").exists());
    previous.kill().unwrap();
    previous.wait().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = restart.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = restart.kill();
            let _ = restart.wait();
            panic!("Language restart did not resume after the previous process exited");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(root.path().join("restarted").is_file());
}

#[test]
fn test_repo_name() {
    assert_eq!(repo_name(Channel::Dev), "warpdotdev-dev");
    assert_eq!(repo_name(Channel::Stable), "warpdotdev");
}
