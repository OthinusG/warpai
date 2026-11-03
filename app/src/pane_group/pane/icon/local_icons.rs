use std::{
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
use warpui::assets::asset_cache::{AssetSource, AsyncAssetId, AsyncAssetType};

const MAX_ICON_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;

/// Icon files compiled into the app; a same-named file in `directory()` overrides them.
const BUNDLED_ICONS: &[(&str, &[u8])] = &[
    (
        "btop.svg",
        include_bytes!("../../../../assets/bundled/pane-icons/btop.svg"),
    ),
    (
        "herdr.svg",
        include_bytes!("../../../../assets/bundled/pane-icons/herdr.svg"),
    ),
    (
        "spotify.svg",
        include_bytes!("../../../../assets/bundled/pane-icons/spotify.svg"),
    ),
    (
        "ssh.svg",
        include_bytes!("../../../../assets/bundled/pane-icons/ssh.svg"),
    ),
    (
        "wsl.svg",
        include_bytes!("../../../../assets/bundled/pane-icons/wsl.svg"),
    ),
];

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct LocalPaneIcon {
    pub name: String,
    #[serde(skip)]
    pub source: Option<AssetSource>,
    /// Whether the asset is a red-channel mask that follows the theme foreground color.
    #[serde(skip)]
    pub tinted: bool,
}

impl From<LocalPaneIcon> for String {
    fn from(icon: LocalPaneIcon) -> Self {
        icon.name
    }
}

impl TryFrom<String> for LocalPaneIcon {
    type Error = &'static str;
    fn try_from(name: String) -> Result<Self, Self::Error> {
        let extension = Path::new(&name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase);
        if name.is_empty()
            || name.contains(['/', '\\', ':'])
            || name.chars().any(char::is_control)
            || !matches!(extension.as_deref(), Some("svg" | "png"))
        {
            return Err("Local pane icons require an SVG/PNG basename");
        }
        Ok(Self {
            name,
            source: None,
            tinted: false,
        })
    }
}

impl PartialEq for LocalPaneIcon {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for LocalPaneIcon {}

struct LocalIconAsset;
impl AsyncAssetType for LocalIconAsset {}

pub(crate) fn directory() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config/warpai/icon"))
}

/// Bundled icons plus validated files from `root`; files override bundles by name.
///
/// Run off the UI thread when a directory is present.
pub(crate) fn discover_all(root: Option<&Path>) -> Vec<LocalPaneIcon> {
    let mut icons = bundled_icons();
    if let Some(root) = root {
        for icon in discover(root) {
            match icons.iter_mut().find(|existing| existing.name == icon.name) {
                Some(existing) => *existing = icon,
                None => icons.push(icon),
            }
        }
    }
    icons.sort_by(|left, right| left.name.cmp(&right.name));
    icons
}

fn bundled_icons() -> Vec<LocalPaneIcon> {
    BUNDLED_ICONS
        .iter()
        .filter_map(|(name, data)| {
            let name = LocalPaneIcon::try_from((*name).to_owned()).ok()?.name;
            let (data, tinted) = icon_from_bytes(&name, *data)?;
            Some(pane_icon(name, data, tinted))
        })
        .collect()
}

/// Run off the UI thread. Only direct regular files become in-memory assets.
pub(crate) fn discover(root: &Path) -> Vec<LocalPaneIcon> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return vec![];
    };
    let mut paths: Vec<_> = entries
        .take(1024)
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    let mut total = 0;
    paths
        .into_iter()
        .filter_map(|path| {
            let name = LocalPaneIcon::try_from(path.file_name()?.to_str()?.to_owned())
                .ok()?
                .name;
            let mut data = Vec::new();
            File::open(&path)
                .ok()?
                .take(MAX_ICON_BYTES + 1)
                .read_to_end(&mut data)
                .ok()?;
            let (data, tinted) = icon_from_bytes(&name, &data)?;
            if total + data.len() > MAX_TOTAL_BYTES {
                return None;
            }
            total += data.len();
            Some(pane_icon(name, data, tinted))
        })
        .take(128)
        .collect()
}

/// Validate and normalize an SVG or PNG icon, returning the bytes to cache and whether
/// the icon follows the theme foreground color.
fn icon_from_bytes(name: &str, data: &[u8]) -> Option<(Vec<u8>, bool)> {
    if data.len() as u64 > MAX_ICON_BYTES {
        return None;
    }
    let extension = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
    if extension == "svg" {
        let source = std::str::from_utf8(data)
            .ok()?
            .trim_start_matches('\u{feff}')
            .trim_start();
        if !source.starts_with('<') {
            return None;
        }
        let sanitized = warpui::image_cache::sanitize_local_icon_svg(source.as_bytes()).ok()?;
        Some((sanitized.data, sanitized.tinted))
    } else if extension == "png" {
        let reader =
            image::ImageReader::with_format(std::io::Cursor::new(data), image::ImageFormat::Png);
        let (width, height) = reader.into_dimensions().ok()?;
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return None;
        }
        let image = image::load_from_memory_with_format(data, image::ImageFormat::Png).ok()?;
        // Tab previews never need a full-size bitmap resident in the UI cache.
        let mut output = std::io::Cursor::new(Vec::new());
        image
            .thumbnail(256, 256)
            .write_to(&mut output, image::ImageFormat::Png)
            .ok()?;
        Some((output.into_inner(), false))
    } else {
        None
    }
}

fn pane_icon(name: String, data: Vec<u8>, tinted: bool) -> LocalPaneIcon {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let id = AsyncAssetId::new::<LocalIconAsset>(format!("{name}:{:x}", hasher.finish()));
    let bytes = Arc::new(data);
    let source = AssetSource::Async {
        id,
        fetch: Arc::new(move || {
            let bytes = bytes.clone();
            Box::pin(async move { Ok(bytes.as_ref().clone().into()) })
        }),
    };
    LocalPaneIcon {
        name,
        source: Some(source),
        tinted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pane_group::pane::icon::PaneIcon;

    const SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><circle cx="12" cy="12" r="10" fill="gray"/></svg>"#;

    #[test]
    fn terminal_ssh_session_local_icon_discovery_validates_and_refreshes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("icon");
        assert!(discover(&root).is_empty());
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("A.SVG"), format!("\u{feff}  {SVG}")).unwrap();
        image::RgbaImage::from_pixel(2, 2, image::Rgba([20, 30, 40, 0]))
            .save(root.join("b.PNG"))
            .unwrap();
        std::fs::write(root.join("broken.svg"), "not an SVG").unwrap();
        std::fs::write(root.join("broken.png"), "not a PNG").unwrap();
        std::fs::write(root.join("ignored.jpg"), SVG).unwrap();
        #[cfg(unix)]
        std::fs::write(root.join("invalid:name.svg"), SVG).unwrap();
        std::fs::create_dir(root.join("nested")).unwrap();
        std::fs::write(root.join("nested/ignored.svg"), SVG).unwrap();
        File::create(root.join("large.svg"))
            .unwrap()
            .set_len(MAX_ICON_BYTES + 1)
            .unwrap();
        image::RgbaImage::new(4097, 1)
            .save(root.join("wide.png"))
            .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("A.SVG"), root.join("link.svg")).unwrap();

        let icons = discover(&root);
        assert_eq!(
            icons
                .iter()
                .map(|icon| icon.name.as_str())
                .collect::<Vec<_>>(),
            ["A.SVG", "b.PNG"]
        );
        assert!(icons.iter().all(|icon| icon.source.is_some()));
        let selection = PaneIcon::Local(icons[0].clone());
        let serialized = serde_json::to_string(&selection).unwrap();
        assert!(!serialized.contains("source"));
        assert!(!serialized.contains(&temp.path().to_string_lossy().to_string()));
        let restored: PaneIcon = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored, selection);
        assert!(
            !restored.is_available(),
            "restart waits for validated local discovery"
        );

        std::fs::write(root.join("A.SVG"), SVG.replace("gray", "black")).unwrap();
        let replaced = discover(&root);
        assert_ne!(
            icons[0].source, replaced[0].source,
            "new content invalidates the image cache"
        );
        std::fs::remove_file(root.join("A.SVG")).unwrap();
        assert!(discover(&root).iter().all(|icon| icon.name != "A.SVG"));
        let invalid = PaneIcon::Local(LocalPaneIcon {
            name: "../outside.svg".into(),
            source: None,
            tinted: false,
        });
        assert!(!invalid.is_available());
    }

    #[test]
    fn terminal_ssh_session_bundled_icons_merge_and_yield_to_user_files() {
        use warpui::AssetProvider;

        for path in [
            crate::ui_components::icon_with_status::SSH_TAB_ICON_PATH,
            crate::ui_components::icon_with_status::WSL_TAB_ICON_PATH,
        ] {
            assert!(
                crate::ASSETS.get(path).is_ok(),
                "missing bundled asset {path}"
            );
        }

        let bundled = discover_all(None);
        assert_eq!(
            bundled
                .iter()
                .map(|icon| icon.name.as_str())
                .collect::<Vec<_>>(),
            ["btop.svg", "herdr.svg", "spotify.svg", "ssh.svg", "wsl.svg"]
        );
        assert!(bundled.iter().all(|icon| icon.source.is_some()));

        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("ssh.svg"), SVG.replace("gray", "black")).unwrap();
        std::fs::write(temp.path().join("custom.svg"), SVG).unwrap();
        let merged = discover_all(Some(temp.path()));
        assert_eq!(merged.len(), 6);
        let user_ssh = merged.iter().find(|icon| icon.name == "ssh.svg").unwrap();
        let bundled_ssh = bundled.iter().find(|icon| icon.name == "ssh.svg").unwrap();
        assert_ne!(
            user_ssh.source, bundled_ssh.source,
            "a user file overrides the bundled icon"
        );
        assert!(merged.iter().any(|icon| icon.name == "custom.svg"));
    }

    #[test]
    fn terminal_ssh_session_local_svg_autocolor_follows_theme() {
        let temp = tempfile::tempdir().unwrap();
        let themed = r#"<svg xmlns="http://www.w3.org/2000/svg" autocolor="true" width="24" height="24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#;
        let plain = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#;
        let fixed = r#"<svg xmlns="http://www.w3.org/2000/svg" autocolor="true" width="24" height="24"><circle cx="12" cy="12" r="10" fill="gray"/></svg>"#;
        std::fs::write(temp.path().join("themed.svg"), themed).unwrap();
        std::fs::write(temp.path().join("plain.svg"), plain).unwrap();
        std::fs::write(temp.path().join("fixed.svg"), fixed).unwrap();

        let icons = discover(temp.path());
        assert_eq!(icons.len(), 3);
        let find = |name: &str| icons.iter().find(|icon| icon.name == name).unwrap();
        let normalized = |icon: &LocalPaneIcon| {
            let Some(AssetSource::Async { fetch, .. }) = &icon.source else {
                panic!("Expected validated in-memory image");
            };
            let bytes = futures::executor::block_on(fetch()).unwrap();
            std::str::from_utf8(&bytes).unwrap().to_owned()
        };

        assert!(find("themed.svg").tinted);
        assert!(!find("plain.svg").tinted);
        assert!(
            !find("fixed.svg").tinted,
            "autocolor without currentColor keeps original colors"
        );

        let themed = normalized(find("themed.svg"));
        assert!(themed.contains("#ff0000"));
        assert!(!themed.contains("currentColor"));
        assert!(normalized(find("plain.svg")).contains("#000000"));
    }

    #[test]
    fn terminal_ssh_session_local_svg_icon_cannot_resolve_external_images() {
        let temp = tempfile::tempdir().unwrap();
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><rect width="24" height="24" fill="gray"/><image href="/outside-icon-directory.png" width="24" height="24"/></svg>"#;
        std::fs::write(temp.path().join("external.svg"), svg).unwrap();
        let icons = discover(temp.path());
        assert_eq!(icons.len(), 1);
        let Some(AssetSource::Async { fetch, .. }) = &icons[0].source else {
            panic!("Expected validated in-memory image");
        };
        let bytes = futures::executor::block_on(fetch()).unwrap();
        let normalized = std::str::from_utf8(&bytes).unwrap();
        assert!(!normalized.contains("outside-icon-directory.png"));
        assert!(normalized.contains("<svg"));
    }
}
