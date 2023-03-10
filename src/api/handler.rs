use crate::config::Config;
use crate::ffmpeg::FFmpeg;
use crate::model::{MovieTable, Table};
use crate::sqlite::SharedDb;
use hyper::{header, Body, Response, StatusCode};
use std::sync::Arc;

macro_rules! json {
    ($x:expr) => {
        match serde_json::to_string($x) {
            Ok(json) => Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(Body::from(json))
                .unwrap(),
            Err(_) => Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("INTERNAL_SERVER_ERROR".into())
                .unwrap(),
        }
    };
}

pub async fn get_movies(db: SharedDb) -> Result<Response<Body>, hyper::Error> {
    let table = MovieTable::new(db);
    let movies = table.all().await.unwrap();
    Ok(json!(&movies))
}

pub async fn get_movie(db: SharedDb, id: i32) -> Result<Response<Body>, hyper::Error> {
    let table = MovieTable::new(db);
    let movie = table.by_id(id).await.unwrap();
    Ok(json!(&movie))
}

// 第二步：执行get_stream
// get_stream的具体实现
pub async fn get_stream(
    db: SharedDb,
    // 用来在多个线程中复用的指针
    config: Arc<Config>,
    id: i32,
) -> Result<Response<Body>, hyper::Error> {
    let table = MovieTable::new(db);
    // let _movie = table.by_id(id).await.unwrap();

    let config = Arc::new(config.ffmpeg.clone());

    // 创建ffmpeg，使用相应的config进行初始化
    let ffmpeg = FFmpeg::new(config);

    // Runtime::channel()创建一个带有关联的发送者一半的主体流。当想要从另一个线程流式传输chunk时很有用。
    let (tx, body) = Body::channel();

    // 这里使用了多线程，开启新的线程调用了ffmpeg下的transcode方法
    tokio::spawn(async move {
        ffmpeg.transcode("h264.mkv", tx).await;
    });

    // 这里构建给前端的返回值
    let resp = Response::builder()
        .header("Content-Type", "video/mp4")
        .header("Content-Disposition", "inline")
        .header("Content-Transfer-Enconding", "binary")
        .body(body)
        .unwrap();

    Ok(resp)
}
