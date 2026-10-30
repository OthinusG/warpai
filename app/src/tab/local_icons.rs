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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LocalTabIcon {
    pub name: String,
    #[serde(skip)]
    pub source: Option<AssetSource>,
}

impl PartialEq for LocalTabIcon {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for LocalTabIcon {}

struct LocalIconAsset;
impl AsyncAssetType for LocalIconAsset {}

pub(crate) fn directory() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config/warpai/icon"))
}

/// Run off the UI thread. Only direct regular files become in-memory assets.
pub(crate) fn discover(root: &Path) -> Vec<LocalTabIcon> {
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
            let name = path.file_name()?.to_str()?.to_owned();
            if name.chars().any(char::is_control) {
                return None;
            }
            let extension = path.extension()?.to_str()?.to_ascii_lowercase();
            if !matches!(extension.as_str(), "png" | "svg") {
                return None;
            }
            let mut data = Vec::new();
            File::open(&path)
                .ok()?
                .take(MAX_ICON_BYTES + 1)
                .read_to_end(&mut data)
                .ok()?;
            if data.len() as u64 > MAX_ICON_BYTES {
                return None;
            }
            let data = if extension == "svg" {
                let source = std::str::from_utf8(&data)
                    .ok()?
                    .trim_start_matches('\u{feff}')
                    .trim_start();
                if !source.starts_with('<') {
                    return None;
                }
                warpui::image_cache::sanitize_local_icon_svg(source.as_bytes()).ok()?
            } else {
                let reader = image::ImageReader::with_format(
                    std::io::Cursor::new(&data),
                    image::ImageFormat::Png,
                );
                let (width, height) = reader.into_dimensions().ok()?;
                if width == 0 || height == 0 || width > 4096 || height > 4096 {
                    return None;
                }
                let image = image::load_from_memory_with_format(&data, image::ImageFormat::Png).ok()?;
                // Tab previews never need a full-size bitmap resident in the UI cache.
                let mut output = std::io::Cursor::new(Vec::new());
                image.thumbnail(256, 256).write_to(&mut output, image::ImageFormat::Png).ok()?;
                output.into_inner()
            };
            if data.len() > MAX_ICON_BYTES as usize || total + data.len() > MAX_TOTAL_BYTES {
                return None;
            }
            total += data.len();
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
            Some(LocalTabIcon {
                name,
                source: Some(source),
            })
        })
        .take(128)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab::TabIcon;

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
        let selection = TabIcon::Local(icons[0].clone());
        let serialized = serde_json::to_string(&selection).unwrap();
        assert!(!serialized.contains("source"));
        assert!(!serialized.contains(&temp.path().to_string_lossy().to_string()));
        let restored: TabIcon = serde_json::from_str(&serialized).unwrap();
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
        let invalid = TabIcon::Local(LocalTabIcon {
            name: "../outside.svg".into(),
            source: None,
        });
        assert!(!invalid.is_available());
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
