use crate::config::FFmpegConfig;
use hyper::body::Bytes;
use hyper::body::Sender;
use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::Arc;

// ArgBuilder中存放着str的引用
struct ArgBuilder<'a> {
    args: Vec<&'a str>,
}

impl<'a> ArgBuilder<'a> {
    // 为ArgBuilder实现with()方法
    // 作用是防止不应在里面的参数
    fn with(mut self, name: &str, val: &'a str) -> ArgBuilder<'a> {
        // search的作用是进行匹配，如果search能够和args进行匹配，那么传入的val将会作为真实的参数传纳入%xx执行
        let search = format!("%{}", name);
        if let Some(arg) = self.args.iter_mut().find(|a| **a == search) {
            *arg = val;
        } else {
            println!("。", name);
        }
        self
    }

    //build()返回ArgBuilder的所有参数
    fn build(self) -> Vec<&'a str> {
        self.args
    }
}

pub struct FFmpeg {
    // FFmpeg的唯一属性config是FFmpegConfig，这个在moviebay.toml中被读取
    config: Arc<FFmpegConfig>,
}

impl FFmpeg {
    pub fn new(config: Arc<FFmpegConfig>) -> FFmpeg {
        FFmpeg { config }
    }

// 这里是整个转码库的核心，在上层应用被调用
    pub async fn transcode(&self, file: &str, mut sender: Sender) {
        let args = self
            .build_args()
            .with("ss", "0")
            .with("i", file)
            .with("f", "mp4")
            .with("vcodec", "copy")
            .with("acodec", "copy")
            .build();

        // 使用command构造 执行命令
        // Constructs a new Command for launching the program at path program, with the following default configuration:
        // Inherit the current process’s environment
        // Inherit the current process’s working directory
        // Inherit stdin/stdout/stderr for spawn or status, but create pipes for output
        // Builder methods are provided to change these defaults and otherwise configure the process.

        // If program is not an absolute path, the PATH will be searched in an OS-defined way.

        //转码后的文件经过pipe传到了stdout中
        let mut cmd = Command::new(&self.config.bin)
        // Adds multiple arguments to pass to the program.
        // 请注意，参数不是通过 shell 传递的，而是按字面意义提供给程序的。 这意味着 shell 语法，如引号、转义字符、分词、glob 模式、替换等无效。
            .args(&args)
        // 子进程的标准输出 (stdout) 句柄的配置。与 spawn 或 status 一起使用时默认为继承，与输出一起使用时默认为管道。
            .stdout(Stdio::piped())
            // Executes the command as a child process, returning a handle to it.
            // By default, stdin, stdout and stderr are inherited from the parent.
            .spawn()
            .unwrap();

        // 指定了buf 65556 个元素都是0
        let mut buf: [u8; 65536] = [0; 65536];
        // 将stdout进行了转换
        let mut stdout = BufReader::new(cmd.stdout.as_mut().unwrap());


        // Read the exact number of bytes required to fill buf.

        // This function reads as many bytes as necessary to completely fill the specified buffer buf.

        // No guarantees are provided about the contents of buf when this function is called, so implementations cannot rely on any property of the contents of buf being true. It is recommended that implementations only write data to buf instead of reading its contents. The documentation on read has a more detailed explanation on this subject.
        while let Ok(()) = stdout.read_exact(&mut buf) {
            // copy_from_slice将数据从buf复制到b中
            let b = Bytes::copy_from_slice(&buf);
            // sender发送chunk（包含b）给接收者
            sender.send_data(b).await.unwrap();
            // 重新让buf填充为0
            buf = [0; 65536];
        }

        let status = cmd.wait();
        println!("Exited with status {:?}", status);
    }

    // 根据初始的参数构造config
    fn build_args(&self) -> ArgBuilder<'_> {
        let args = self
            .config
            .codecs
            .get("*")
            .unwrap()
            .args
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<&str>>();
        ArgBuilder { args }
    }
}
