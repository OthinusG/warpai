pub(crate) mod local_icons;

use crate::ui_components::icons::Icon;
use serde::{Deserialize, Serialize};

/// Stable selections persisted with a pane; local sources are resolved separately.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneIcon {
    Folder,
    Code,
    Globe,
    Cloud,
    Laptop,
    Tool,
    Lightbulb,
    Notebook,
    Local(local_icons::LocalPaneIcon),
}

impl PaneIcon {
    pub(crate) const ALL: [Self; 8] = [
        Self::Folder,
        Self::Code,
        Self::Globe,
        Self::Cloud,
        Self::Laptop,
        Self::Tool,
        Self::Lightbulb,
        Self::Notebook,
    ];

    pub(crate) fn icon(&self) -> Icon {
        match self {
            Self::Folder => Icon::Folder,
            Self::Code => Icon::Code2,
            Self::Globe => Icon::Globe,
            Self::Cloud => Icon::Cloud,
            Self::Laptop => Icon::Laptop,
            Self::Tool => Icon::Tool,
            Self::Lightbulb => Icon::Lightbulb,
            Self::Notebook => Icon::Notebook,
            Self::Local(_) => Icon::Terminal,
        }
    }

    pub(crate) fn label(&self) -> String {
        match self {
            Self::Folder => warpui::localization::text("Folder"),
            Self::Code => warpui::localization::text("Code"),
            Self::Globe => warpui::localization::text("Globe"),
            Self::Cloud => warpui::localization::text("Cloud"),
            Self::Laptop => warpui::localization::text("Laptop"),
            Self::Tool => warpui::localization::text("Tool"),
            Self::Lightbulb => warpui::localization::text("Lightbulb"),
            Self::Notebook => warpui::localization::text("Notebook"),
            Self::Local(icon) => &icon.name,
        }
        .to_string()
    }

    pub(crate) fn is_available(&self) -> bool {
        !matches!(self, Self::Local(icon) if icon.source.is_none())
    }

    pub(crate) fn local_source(&self) -> Option<&warpui::assets::asset_cache::AssetSource> {
        match self {
            Self::Local(icon) => icon.source.as_ref(),
            _ => None,
        }
    }
}
