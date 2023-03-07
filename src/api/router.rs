use super::path::Path;
use hyper::{Body, Error, Method, Response};
use std::future::Future;
use std::pin::Pin;

pub type Handler =
    Pin<Box<dyn Future<Output = Result<Response<Body>, Error>> + Send + Sync + 'static>>;

pub struct RouteBuilder {
    route: Route,
}

impl RouteBuilder {
    pub fn new(route: Route) -> RouteBuilder {
        RouteBuilder { route }
    }

    pub fn name(mut self, name: &str) -> Route {
        self.route.name = name.to_owned();
        self.route
    }
}

/// Holds route information
#[derive(Clone)]
pub struct Route {
    /// HTTP method to match
    pub method: Method,

    /// Path to match
    pub path: Path,

    /// Name of the route
    pub name: String,

    /// Extraced parts of the path
    pub params: Vec<String>,
}

impl Route {
    pub fn get(path: &str) -> RouteBuilder {
        // Method::GET 是 hyper 库中的一个枚举类型，用于表示 HTTP GET 方法。
        Route::from(Method::GET, path)
    }

    fn from(method: Method, path: &str) -> RouteBuilder {
        RouteBuilder::new(Route {
            method,
            path: Path::new(path),
            // name为空字符串转换成String
            name: "".to_owned(),
            // 这里的params没有对url做提取
            // 后续做了相关的处理，采用了regex进行子串的拼接
            params: Vec::new(),
        })
    }
}

pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Router {
        Router { routes: Vec::new() }
    }

    pub fn is_match(&mut self, path: &str) -> Option<Route> {
        // 遍历routes获得可变引用
        for route in self.routes.iter_mut() {
            // path是自定义结构体，path包含的matcher是regex类型
            // regex::Regex::is_match 是 Rust 中的一个方法，用于检查一个字符串是否与正则表达式匹配。
            // 这个方法返回一个布尔值，表示字符串是否与正则表达式匹配。通常，可以将这个方法用于解析文本或验证用户输入。
            if route.path.matcher.is_match(path) {
                // regex::Regex::captures 是 Rust 中的一个方法，用于从一个字符串中提取匹配的子串。
                // 这个方法返回一个 Option 类型，表示字符串是否与正则表达式匹配。
                // 如果匹配成功，则返回一个 Captures 对象，它包含所有捕获的子串。通常，可以将这个方法用于解析文本或提取特定的信息。
                // 在这里就是提取出url的后面的子串
                let caps = route.path.matcher.captures(path).unwrap();
                //这里只会进行一次切分，也就是最多只有一个子串
                if caps.len() > 1 {
                    route.params.push(caps[1].to_owned());
                }
                return Some(route.clone());
            }
        }
        None
    }

    pub fn add(&mut self, route: Route) {
        self.routes.push(route);
    }
}
