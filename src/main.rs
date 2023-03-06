mod api;
mod config;
mod context;
mod ffmpeg;
mod model;
mod scan;
mod sqlite;
mod tmdb;

use crate::api::MakeApiSvc;
use crate::config::Config;
use crate::context::Context;
use crate::model::{MovieTable, Table};
use crate::scan::Scanner;
use crate::sqlite::{params, Connection};
use hyper::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::from_config(Config::new());
    //config 的sqlite和config 来源于ctx
    let sqlite = ctx.db();
    let config = ctx.cfg();

// 这里是tmdb的相关设置，和ffmpeg的相关代码没有关系
    // let tmdb_cfg = tmdb::fetch_configuration(config.tmdb.clone())
    //     .await
    //     .unwrap();
    // let movies = tmdb::search_movie(config.tmdb.clone(), "Collateral", 2004)
    //     .await
    //     .unwrap();

    // // println!("{:?}", tmdb_cfg);
    // // println!("{:?}", movies);
    
    // let table = MovieTable::new(sqlite.clone());
    // table.create_table().await?;

    // let mut scanner = Scanner::new(config.clone());
    // let movies = scanner.run()?;

    // for movie in movies {
    //     println!("{:?}", movie);

    //     let m = movie.clone();
    //     sqlite
    //         .spawn(Box::new(move |conn: &Connection| {
    //             conn.execute(
    //                 "INSERT INTO movies (tmdb_id, title, overview, release_year, file_path, poster_path,  backdrop_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    //                 params![0, m.title, "", m.release_year, m.path.to_str(), m.poster_path, m.backdrop_path],
    //             )
    //         }))
    //         .await?;
    // }

    // sqlite.save().await?;


    // 前端的第一个link，访问了http://localhost:3000/stream/1这个地址
    let addr = ([127, 0, 0, 1], 3000).into();

    // 这里初始化了MakeApiSvc 的new方法
    // 使用了config和sqlite进行了配置的初始化，并且绑定到server
    let server = Server::bind(&addr).serve(MakeApiSvc::new(config, sqlite));
    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}
