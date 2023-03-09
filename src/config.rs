use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

pub type SharedCfg = Arc<Config>;

// CodecConfig封装了许多String，用来存储所有的字符串
/// Codec settings used with FFmpeg
#[derive(Debug, Clone, Deserialize)]
pub struct CodecConfig {
    pub args: Vec<String>,
}

/// Settings for Database
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
}


// FFmpegConfig struct

/// Settings for FFmpeg
#[derive(Debug, Clone, Deserialize)]
pub struct FFmpegConfig {
    pub bin: String,
    pub codecs: HashMap<String, CodecConfig>,
}

/// Settings for the library
#[derive(Debug, Clone, Deserialize)]
pub struct LibraryConfig {
    pub movies: String,
}

/// Settings for the The Movie Database
#[derive(Debug, Clone, Deserialize)]
pub struct TmdbConfig {
    pub api_key: String,
}


/// The base `Config` for moviebay
// 所有的类型，若想用 std::fmt 的格式化打印，都要求实现至少一个可打印的 traits。
// 仅有一些类型提供了自动实现，比如 std 库中的类型。所有其他类型都必须手动实现。
// fmt::Debug 这个 trait 使这项工作变得相当简单。
// 所有类型都能推导（derive，即自动创建）fmt::Debug 的实现。但是 fmt::Display 需要手动实现。

// Serde 提供了一个 derive 宏来为你的 crate 中定义的数据结构生成序列化和反序列化特征的实现，
// 允许它们以所有 Serde 的数据格式方便地表示。
/// The base `Config` for moviebay
#[derive(Debug, Deserialize)]
pub struct Config {
    pub ffmpeg: FFmpegConfig,
    pub library: LibraryConfig,
    pub database: DatabaseConfig,
    pub tmdb: TmdbConfig,
}

impl Config {
    // 这里调用了config的new方法，返回了默认设置
    pub fn new() -> SharedCfg {
        Config::default().into_shared()
    }

    // 从一个配置文件直接生成config
    pub fn from_file<P: AsRef<Path>>(file: P) -> io::Result<Config> {
        let mut contents = String::new();
        let mut file = File::open(file)?;
        // 将file的内容读取到contents中
        file.read_to_string(&mut contents)?;
        // 通过toml:: from_str把moviebay.toml中的字符串反序列化成类型
        let config: Config = toml::from_str(&contents)?;
        println!("{:?}", config);

        Ok(config)
    }

    // 作用是将一个数据封装成Arc数据，用来在多线程中共享
    pub fn into_shared(self) -> SharedCfg {
        Arc::new(self)
    }
}

// default trait 可以用于定义类型的默认值。
// 实现这个方法能够直接返回类的实例
impl Default for Config {
    fn default() -> Config {
        Config::from_file("moviebay.toml").unwrap()
    }
}
