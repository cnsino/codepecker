use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::Read,
    path::Path,
    time::Duration,
};

use crate::{error::CodepeckerError, project::Project, project::Source};
use reqwest::{multipart, Client, IntoUrl};
use serde_json::Value;
#[derive(Debug, Clone)]
pub(crate) struct Pecker<T> {
    url: T,
    client: Client,
    key: String,
}

impl<T> Pecker<T>
where
    T: IntoUrl + Display,
{
    pub(crate) async fn new<U>(
        url: T,
        proxy: Option<U>,
        key: String,
    ) -> Result<Self, CodepeckerError>
    where
        U: IntoUrl + Display,
    {
        let mut builder = Client::builder();
        if let Some(proxy) = proxy {
            log::debug!("使用的代理：{proxy}");
            let proxy = reqwest::Proxy::all(proxy).map_err(|_| CodepeckerError::ProxyBuildError)?;
            builder = builder.proxy(proxy);
        }
        let client = builder
            .http1_title_case_headers()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|_| CodepeckerError::ClientBuildError)?;

        let pecker = Self { url, client, key };
        Ok(pecker)
    }

    // 上传源码并检测
    pub(crate) async fn post_source_code(
        &self,
        project: &Project,
        zip_file: &str,
    ) -> Result<String, CodepeckerError> {
        let upload_url = format!("{}cp4/webInterface/postSourceCode.action", self.url);
        log::debug!("upload_url{:?}", upload_url);
        let file_path = Path::new(zip_file);

        let mut file = fs::File::open(file_path)?;
        let metadata = fs::metadata(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // 获取文件的 MIME 类型
        let mime_type = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .as_ref()
            .to_string();
        log::debug!("mime_type：{:?}", mime_type);

        let template = project.template.to_string();

        let mut form = multipart::Form::new()
            .text("auth", self.key.to_string())
            .text("projectId", project.name.to_string())
            .text("langType", project.lang.to_string())
            .text("projectLevel", template.to_string());
        if let Some(group) = &project.group {
            form = form.text("projectGroupId", group.to_string());
        }
        if template == "user_defined" {
            form = form.text("ruleId", project.rule.as_ref().unwrap().to_string());
        }

        form = form.part(
            "uploadFile",
            multipart::Part::stream_with_length(buffer, metadata.len())
                .file_name(
                    file_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                )
                .mime_str(&mime_type)
                .map_err(|_| CodepeckerError::FileUploadError)?, // 这里设置你的 content-type
        );

        log::debug!("{:?}", form.boundary());
        let response = self
            .client
            .post(&upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(|_| CodepeckerError::UnableToConnect(upload_url))?;
        if response.status().is_success() {
            log::info!("下发任务请求完成!");
            let results = response
                .json::<Value>()
                .await
                .map_err(|_| CodepeckerError::UnableToParseJson)?;
            if let Some(0) = results["status"].as_u64() {
                if let Some(task) = results["taskId"].as_str() {
                    log::info!("从服务端获取任务id完成!");
                    return Ok(task.to_string());
                }
            } else if let Some(error_msg) = results["errorMsg"].as_str() {
                log::error!("下发任务失败: {}!", error_msg.to_string());
                return Err(CodepeckerError::CustomInvalidInfo(error_msg.to_string()));
            }
        } else {
            log::debug!("{}", response.status());
            log::debug!(
                "{:?}",
                response
                    .text()
                    .await
                    .map_err(|_| CodepeckerError::UnableToGetText)?
            );
            log::error!("上传源代码文件失败,请检查URL地址及key值!");
        }
        Err(CodepeckerError::CustomInvalidInfo(
            "上传源代码文件失败,请检查URL地址及key值".to_owned(),
        ))
    }

    // 通过SVN/GIT下载源码并检测
    pub(crate) async fn post_source_code_by_svn_or_git<U>(
        &self,
        project: &Project,
        source: &Source<U>,
    ) -> Result<String, CodepeckerError>
    where
        U: IntoUrl + Display,
    {
        let upload_url = format!("{}cp4/webInterface/postSourceCodeBySvnGit.action", self.url);
        log::debug!("upload_url{:?}", upload_url);

        let template = project.template.to_string();
        let mut params = HashMap::new();
        params.insert("auth", self.key.to_string());
        params.insert("projectId", project.name.to_string());
        if let Some(group) = &project.group {
            params.insert("projectGroupId", group.to_string());
        }
        params.insert("langType", project.lang.to_string());
        params.insert("projectLevel", template.to_string());

        if template == "user_defined" {
            params.insert("ruleId", project.rule.as_ref().unwrap().to_string());
        }
        params.insert("downloadType", source.remote.to_string());
        params.insert("svngitUrl", source.url.to_string());
        params.insert("svngitUserName", source.user.to_string());
        params.insert("svngitPassword", source.password.to_string());
        if let Some(branch) = &source.branch {
            params.insert("gitBranchName", branch.to_string());
        }

        let response = self
            .client
            .post(&upload_url)
            .form(&params)
            .send()
            .await
            .map_err(|_| CodepeckerError::UnableToConnect(upload_url))?;
        if response.status().is_success() {
            log::info!("下发任务请求完成!");
            let results = response
                .json::<Value>()
                .await
                .map_err(|_| CodepeckerError::UnableToParseJson)?;
            if let Some(0) = results["status"].as_u64() {
                if let Some(task) = results["taskId"].as_str() {
                    log::info!("从服务端获取任务id完成!");
                    return Ok(task.to_string());
                }
            } else if let Some(error_msg) = results["errorMsg"].as_str() {
                log::error!("下发任务失败: {}!", error_msg.to_string());
                return Err(CodepeckerError::CustomInvalidInfo(error_msg.to_string()));
            }
        } else {
            log::debug!("{}", response.status());
            log::debug!(
                "{:?}",
                response
                    .text()
                    .await
                    .map_err(|_| CodepeckerError::UnableToGetText)?
            );
            log::error!("部署GIT/SVN源代码扫描失败,请检查URL地址及key值!");
        }
        Err(CodepeckerError::CustomInvalidInfo(
            "部署GIT/SVN源代码扫描失败,请检查URL地址及key值".to_owned(),
        ))
    }

    // 查询检测项目的状态
    pub(crate) async fn query_task_status(&self, task: &str) -> Result<bool, CodepeckerError> {
        let status_url = format!("{}cp4/webInterface/queryTaskStatus.action", self.url);
        let mut params = HashMap::new();
        params.insert("taskId", task);
        params.insert("auth", &self.key);
        loop {
            log::debug!("status_url{:?}", status_url);
            let response = self
                .client
                .post(&status_url)
                .form(&params)
                .send()
                .await
                .map_err(|_| CodepeckerError::UnableToConnect(status_url.to_string()))?
                .json::<Value>()
                .await
                .map_err(|_| CodepeckerError::UnableToParseJson)?;
            if let Some(0) = response["status"].as_u64() {
                match response["taskStatus"].as_str() {
                    Some("0") => log::info!("代码上传成功"),
                    Some("1") => log::info!("已解压待检测"),
                    Some("2") => log::info!("检查中，请等待"),
                    Some("3") => {
                        log::info!("检测完成");
                        return Ok(true);
                    }
                    Some("4") => {
                        log::error!("检测异常,请查看codepecker运行状态");
                        return Err(CodepeckerError::CustomInvalidInfo(
                            "检测异常,请查看codepecker运行状态".to_owned(),
                        ));
                    }
                    Some("99") => log::warn!("排队中，请等待"),
                    _ => {
                        log::error!("检测异常,其他未知性错误,不该执行到的地方");
                        return Err(CodepeckerError::CustomInvalidInfo(
                            "检测异常,其他未知性错误".to_owned(),
                        ));
                    }
                }
            } else if let Some(error_msg) = response["errorMsg"].as_str() {
                log::error!("下发任务失败: {}!", error_msg.to_string());
                return Err(CodepeckerError::CustomInvalidInfo(error_msg.to_string()));
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    // 获取检测结果统计信息
    async fn query_statistics(&self, task: &str) -> Result<Value, CodepeckerError> {
        let statistics_url = format!("{}cp4/webInterface/queryStatistics.action", self.url);
        log::debug!("statistics_url{:?}", statistics_url);
        let mut params = HashMap::new();
        params.insert("taskId", task);
        params.insert("auth", &self.key);

        let response = self
            .client
            .post(&statistics_url)
            .form(&params)
            .send()
            .await
            .map_err(|_| CodepeckerError::UnableToConnect(statistics_url.to_string()))?;

        if response.status().is_success() {
            log::info!("获取扫描结果请求完成!");
            let results = response
                .json::<Value>()
                .await
                .map_err(|_| CodepeckerError::UnableToParseJson)?;
            Ok(results)
        } else {
            log::error!("无法从服务端获取扫描结果,请检查URL地址及key值.");
            Err(CodepeckerError::CustomInvalidInfo(
                "无法从服务端获取扫描结果,请检查URL地址及key值".to_owned(),
            ))
        }
    }

    fn filter_by_severity(&self, severity: &str, all_defects: Vec<Value>) -> Vec<Value> {
        match severity {
            "info" => all_defects
                .into_iter()
                .filter(|v| v["severityLevel"].as_i64().unwrap_or(0) <= 5)
                .collect(),
            "low" => all_defects
                .into_iter()
                .filter(|v| v["severityLevel"].as_i64().unwrap_or(0) <= 4)
                .collect(),
            "medium" => all_defects
                .into_iter()
                .filter(|v| v["severityLevel"].as_i64().unwrap_or(0) <= 3)
                .collect(),
            "high" => all_defects
                .into_iter()
                .filter(|v| v["severityLevel"].as_i64().unwrap_or(0) <= 2)
                .collect(),
            "critical" => all_defects
                .into_iter()
                .filter(|v| v["severityLevel"].as_i64().unwrap_or(0) == 1)
                .collect(),
            _ => vec![],
        }
    }

    // 获取检测结果
    pub(crate) async fn get_task_result(
        &self,
        task: &str,
        severity: &str,
        output: &str,
    ) -> Result<(), CodepeckerError> {
        let result_url = format!("{}cp4/webInterface/getTaskResult.action", self.url);
        log::debug!("result_url{:?}", result_url);
        let mut all_defects = Vec::new();
        let mut request_num = 1;
        let info = self.query_statistics(task).await?;
        loop {
            let mut params = HashMap::new();
            params.insert("taskId", task);
            params.insert("auth", &self.key);
            let request_num_str = request_num.to_string();
            params.insert("requestNum", &request_num_str);

            let response = self
                .client
                .post(&result_url)
                .form(&params)
                .send()
                .await
                .map_err(|_| CodepeckerError::UnableToConnect(result_url.to_string()))?;

            if response.status().is_success() {
                log::info!("获取第{request_num_str}页扫描结果请求完成!");
                let results = response
                    .json::<Value>()
                    .await
                    .map_err(|_| CodepeckerError::UnableToParseJson)?;
                // 解析响应体为 JSON
                if let Some(defects) = results.get("problem").and_then(|v| v.as_array()) {
                    if defects.is_empty() {
                        break;
                    } else {
                        all_defects.extend_from_slice(defects);
                        request_num += 1;
                    }
                } else {
                    break;
                }
            } else {
                log::error!("无法从服务端获取扫描结果,请检查URL地址及key值.");
                return Err(CodepeckerError::CustomInvalidInfo(
                    "无法从服务端获取扫描结果,请检查URL地址及key值".to_owned(),
                ));
            }
        }
        let filter_problems = self.filter_by_severity(severity, all_defects);
        let problem_count = filter_problems.len();
        log::info!("筛选{severity}及级别以上的缺陷或漏洞,数量为{problem_count}个");
        let result_json = serde_json::json!({
            "task_id": task,
            "severity": severity,
            "problem_count": problem_count,
            "info": info,
            "problems": filter_problems
        });
        // 将 JSON 写入文件
        let file = File::create(output)?;
        log::debug!("{:?}", file.metadata());
        serde_json::to_writer_pretty(file, &result_json)?;
        log::info!("将扫描结果写入文件{:?}完成!", output);
        Ok(())
    }

    // 获取开源组件检测结果统计信息
    // pub(crate) async fn query_task_jars_detection_result() {
    //     todo!()
    // }
}
