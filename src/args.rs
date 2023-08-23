use std::str::FromStr;

use clap::builder::TypedValueParser as _;
use clap::Parser;
use reqwest::Url;
/// Codepecker 的命令行程序
#[derive(Parser, Debug, Clone)]
#[command(author,version, about, long_about = None)]
pub(crate) struct Codepecker {
    /// 设置 Codepecker 的访问地址. eg. http://pecker.abc.local:8081.
    #[arg(
        short,
        long,
        value_name = "Url",
        default_value = "http://127.0.0.1:8081"
    )]
    pub(crate) url: Option<Url>,
    /// 设置 Codepecker 的apikey. eg. adfadfe343g.
    #[arg(
        short,
        long,
        value_name = "Apikey",
        default_value = "Oh9LHLfrLgk77e67DEZtiitOWZwvFVXI"
    )]
    pub(crate) key: Option<String>,

    /// 设置 Codepecker 的连接代理. eg. http://127.0.0.1:8080.
    #[arg(long, value_name = "Proxy")]
    pub(crate) proxy: Option<Url>,

    /// 设置 Codepecker 的项目名称.
    #[arg(short, long, value_name = "Project Name", default_value = "test")]
    pub(crate) project: Option<String>,
    /// 设置 Codepecker 的项目组id.
    #[arg(long, value_name = "Project Group ID")]
    pub(crate) group: Option<String>,
    /// 设置 Codepecker 的项目语言.
    #[arg(short, long, value_name = "Project Language", default_value = "java")]
    pub(crate) lang: Option<String>,
    /// 设置 Codepecker 的缺陷模板类型.
    #[arg(short, long, value_name = "Scan Template",default_value = "default",value_parser = clap::builder::PossibleValuesParser::new(["default", "high", "user_defined"]))]
    pub(crate) template: Option<String>,
    /// 设置 Codepecker 的缺陷模板规则,当缺陷模板(template)类型为 user_defined 时生效.
    #[arg(short, long, value_name = "Scan Rule")]
    pub(crate) rule: Option<String>,

    /// 设置 Codepecker 的源码文件.
    #[arg(short, long, value_name = "Zip File")]
    pub(crate) file: Option<String>,

    /// 设置 Codepecker 的SVN地址.
    #[arg(short, long, value_name = "SVN")]
    pub(crate) svn: Option<Url>,
    /// 设置 Codepecker 的GIT地址.
    #[arg(short, long, value_name = "GIT")]
    pub(crate) git: Option<Url>,

    /// 设置 Codepecker SVN/GIT的用户名.
    #[arg(long, value_name = "SVN/GIT UserName")]
    pub(crate) user: Option<String>,
    /// 设置 Codepecker SVN/GIT的密码.
    #[arg(long, value_name = "SVN/GIT Password")]
    pub(crate) password: Option<String>,
    /// 设置 Codepecker SVN/GIT的分支.
    #[arg(long, value_name = "SVN/GIT Password")]
    pub(crate) branch: Option<String>,

    /// 设置 Codepecker 的taskid.
    #[arg(long, value_name = "Task ID")]
    pub(crate) task: Option<String>,
    /// 设置 Codepecker 的扫描结果存储位置.
    #[arg(
        short,
        long,
        value_name = "Scan Result",
        default_value = "results.json"
    )]
    pub(crate) output: Option<String>,

    /// 设置输出日志的级别(选择off不输出日志)
    #[arg(
        long,
        default_value = "debug",
        value_parser = clap::builder::PossibleValuesParser::new(["off", "debug", "info", "warn", "error"])
            .map(|s| log::LevelFilter::from_str(&s).unwrap()),
    )]
    pub(crate) log_level: log::LevelFilter,
}
