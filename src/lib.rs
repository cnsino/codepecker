use clap::Parser;
use error::CodepeckerError;

use crate::project::{Project, Source};

mod args;
pub mod error;
mod pecker;
mod project;
pub async fn builder() -> Result<(), CodepeckerError> {
    let args = args::Codepecker::parse();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Off)
        .filter_module("codepecker", args.log_level)
        .init();

    let pecker = pecker::Pecker::new(args.url.unwrap(), args.proxy, args.key.unwrap()).await?;
    log::debug!("{pecker:?}");

    if let (Some(task), Some(output)) = (&args.task, &args.output) {
        log::info!("外部传入扫描id{:?}", task);
        pecker.get_task_result(task, output).await?;
        return Ok(());
    }

    if let (Some(project_name), Some(lang), Some(template), Some(output)) = (
        args.project,
        args.lang,
        args.template,
        &args.output,
    ) {
        let rule = args.rule;
        let group = args.group;
        let project = Project {
            name: project_name,
            lang,
            template, 
            rule, 
            group,
        };
        log::debug!("输入的参数：项目：{:?}", project);
        let mut taskid = String::new();
        if let Some(code_file) = &args.file {
            taskid = pecker.post_source_code(&project, code_file).await?;
        } else {
            let mut source: Option<Source<reqwest::Url>> = None;
            let branch = args.branch;
            if let Some(git) = args.git {
                if let (Some(user), Some(password)) = (args.user, args.password) {
                    source = Some(Source {
                        remote: 2.to_string(),
                        url: git,
                        user,
                        password,
                        branch,
                    });
                }
            } else if let Some(svn) = args.svn {
                if let (Some(user), Some(password)) = (args.user, args.password) {
                    source = Some(Source {
                        remote: 1.to_string(),
                        url: svn,
                        user,
                        password,
                        branch,
                    });
                }
            }
            if let Some(ref s) = source {
                taskid = pecker.post_source_code_by_svn_or_git(&project, s).await?;
            }
        }
        if !taskid.is_empty() {
            log::info!("代码扫描任务: {:?}下发完成", taskid);
            let scan_status = pecker.query_task_status(taskid.as_str()).await?;
            if scan_status {
                log::info!("代码扫描任务: {:?}扫描完成", taskid);
                pecker.get_task_result(taskid.as_str(), output).await?;
            }
        }
    } else {
        log::error!("{}", CodepeckerError::ParamMissing);
        return Err(CodepeckerError::ParamMissing);
    }
    Ok(())
}
