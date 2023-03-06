use hyper::service::Service;
use hyper::{Body, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::{
    handler,
    router::{Handler, Route, Router},
};
use crate::config::SharedCfg;
use crate::sqlite::SharedDb;
use serde::Serialize;

type FuturePin<T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>;

#[derive(Debug, Serialize)]
struct Movie {
    id: i32,
    title: String,
    release_year: i32,
    file_path: String,
    poster_path: String,
    backdrop_path: String,
}

pub struct ApiService {
    config: SharedCfg,
    db: SharedDb,
    router: Router,
}

impl ApiService {
    // new函数为ApiService添加了相应的路径
    fn new(db: SharedDb, config: SharedCfg) -> ApiService {
        let mut router = Router::new();
        router.add(Route::get(r"/movies/(\d+)").name("get_movie"));
        router.add(Route::get("/movies/").name("get_movies"));
        router.add(Route::get(r"/stream/(\d+)").name("get_stream"));
        ApiService { config, db, router }
    }
}

impl Service<Request<Body>> for ApiService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = FuturePin<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        if let Some(route) = self.router.is_match(req.uri().path()) {
            let res: Handler = match route.name.as_ref() {
                "get_movies" => Box::pin(handler::get_movies(self.db.clone())),
                "get_movie" => {
                    let id = route.params[0].parse().unwrap();
                    Box::pin(handler::get_movie(self.db.clone(), id))
                }
                // 如果是get_stream 就匹配到handler中的service方法
                "get_stream" => {
                    let id = route.params[0].parse().unwrap();
                    Box::pin(handler::get_stream(
                        self.db.clone(),
                        self.config.clone(),
                        id,
                    ))
                }
                _ => unimplemented!(),
            };
            return res;
        }

        Box::pin(async { Ok(Response::builder().body(Body::from("Not Found")).unwrap()) })
    }
}

// 定义结构体MakeApiSvc，其中包含两个属性config和db
pub struct MakeApiSvc {
    config: SharedCfg,
    db: SharedDb,
}

impl MakeApiSvc {
    pub fn new(config: SharedCfg, db: SharedDb) -> MakeApiSvc {
        MakeApiSvc { config, db }
    }
}

// 为MakeApiSvc实现了Service接口
impl<T> Service<T> for MakeApiSvc {
    type Response = ApiService;
    type Error = hyper::Error;
    type Future = FuturePin<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: T) -> Self::Future {
        // 获取MakeApiSvc的config和db，这里直接使用clone方法
        let config = self.config.clone();
        let db = self.db.clone();

        // routes

        //  在这里通过一个service初始化了另外一个 service ，也就是ApiService
        let fut = async move { Ok(ApiService::new(db, config)) };
        Box::pin(fut)
    }
}
