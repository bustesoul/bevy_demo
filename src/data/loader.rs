use bevy::asset::{io::Reader, ron, AssetLoader, LoadContext};
use bevy::prelude::*;
use std::future::Future;
use thiserror::Error;

use super::schema::ItemList;

#[derive(Default)]
pub struct RonItemLoader;

#[derive(Debug, Error)]
pub enum RonItemLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse RON: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("Could not interpret bytes as UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

impl AssetLoader for RonItemLoader {
    type Asset = ItemList;
    type Settings = ();
    type Error = RonItemLoaderError;

    // --- 正确的签名 ---
    // 1. 移除 fn load<'a> 中的 <'a>
    // 2. 移除所有参数前的 'a
    // 3. 移除返回值 impl Future... 末尾的 + 'a
    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl Future<Output = Result<Self::Asset, Self::Error>> + Send {
        async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let s = std::str::from_utf8(&bytes)?;
            let list: ItemList = ron::de::from_str(s)?;

            Ok(list)
        }
    }
}