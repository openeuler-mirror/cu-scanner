//! CSAF文件获取库
//!
//! 该模块提供了通过HTTP协议获取CSAF文件的功能，支持同步和异步两种方式。

use csaf::CSAF;
use log::{debug, error, info, warn};
use reqwest;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use thiserror::Error;
use url::Url;

/// CSAF获取器错误类型
#[derive(Error, Debug)]
pub enum FetchError {
    /// HTTP请求错误
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// URL解析错误
    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    /// JSON解析错误
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP状态码错误
    #[error("HTTP status error: {status}, body: {body}")]
    StatusError { status: u16, body: String },

    /// IO错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}

/// 结果类型
pub type Result<T> = std::result::Result<T, FetchError>;

/// 异步检查回调函数类型
/// 参数：文件路径
/// 返回：true 表示文件已存在（跳过下载），false 表示不存在（需要下载）
pub type AsyncCheckCallback = Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>;

/// 同步检查回调函数类型
/// 参数：文件路径
/// 返回：true 表示文件已存在（跳过下载），false 表示不存在（需要下载）
pub type CheckCallback = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// CSAF获取器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetcherConfig {
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
    /// 用户代理字符串
    pub user_agent: String,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            user_agent: "CSAF-Fetcher/0.1.0".to_string(),
        }
    }
}

/// CSAF同步获取器
pub struct CsafFetcher {
    client: reqwest::blocking::Client,
    config: FetcherConfig,
}

impl CsafFetcher {
    /// 创建新的CSAF获取器
    ///
    /// # 参数
    ///
    /// * `config` - 配置选项
    ///
    /// # 返回值
    ///
    /// 返回Result<CsafFetcher>
    pub fn new(config: FetcherConfig) -> Result<Self> {
        info!("创建CSAF同步获取器，超时: {}秒", config.timeout_secs);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .build()?;

        Ok(Self { client, config })
    }

    /// 使用默认配置创建获取器
    pub fn with_defaults() -> Result<Self> {
        Self::new(FetcherConfig::default())
    }

    /// 从URL获取CSAF文件
    ///
    /// # 参数
    ///
    /// * `url` - CSAF文件的URL地址
    ///
    /// # 返回值
    ///
    /// 返回Result<CSAF>
    pub fn fetch(&self, url: &str) -> Result<CSAF> {
        info!("开始获取CSAF文件: {}", url);

        // 验证URL
        let parsed_url = Url::parse(url)?;
        debug!("URL解析成功: {}", parsed_url);

        let mut last_error = None;

        // 重试逻辑
        for attempt in 1..=self.config.max_retries {
            debug!("尝试第 {} 次获取", attempt);

            match self.fetch_once(url) {
                Ok(csaf) => {
                    info!("成功获取CSAF文件，漏洞数量: {}", csaf.vulnerabilities.len());
                    return Ok(csaf);
                }
                Err(e) => {
                    warn!("第 {} 次获取失败: {}", attempt, e);
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        std::thread::sleep(Duration::from_millis(self.config.retry_delay_ms));
                    }
                }
            }
        }

        error!("所有重试均失败");
        Err(last_error.unwrap())
    }

    /// 单次获取（不重试）
    fn fetch_once(&self, url: &str) -> Result<CSAF> {
        let response = self.client.get(url).send()?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .unwrap_or_else(|_| String::from("无法读取响应体"));
            return Err(FetchError::StatusError {
                status: status.as_u16(),
                body,
            });
        }

        let text = response.text()?;
        debug!("接收到响应，长度: {} 字节", text.len());

        let csaf: CSAF = serde_json::from_str(&text)?;
        Ok(csaf)
    }

    /// 从URL获取CSAF并保存到文件
    ///
    /// # 参数
    ///
    /// * `url` - CSAF文件的URL地址
    /// * `output_path` - 输出文件路径
    ///
    /// # 返回值
    ///
    /// 返回Result<CSAF>
    pub fn fetch_and_save(&self, url: &str, output_path: &str) -> Result<CSAF> {
        info!("获取CSAF文件并保存到: {}", output_path);

        let csaf = self.fetch(url)?;

        // 保存到文件
        let json_str = serde_json::to_string_pretty(&csaf)?;
        std::fs::write(output_path, json_str)?;

        info!("成功保存CSAF文件到: {}", output_path);
        Ok(csaf)
    }

    /// 批量获取CSAF文件
    ///
    /// # 参数
    ///
    /// * `urls` - CSAF文件URL列表
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表
    pub fn fetch_batch(&self, urls: &[String]) -> Vec<(String, Result<CSAF>)> {
        info!("批量获取 {} 个CSAF文件", urls.len());

        urls.iter()
            .map(|url| {
                let result = self.fetch(url);
                (url.clone(), result)
            })
            .collect()
    }

    /// 从index.txt文件获取CSAF文件路径列表
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址，例如: "http://csaf-website/index.txt"
    ///
    /// # 返回值
    ///
    /// 返回CSAF文件相对路径列表
    pub fn fetch_index(&self, index_url: &str) -> Result<Vec<String>> {
        info!("获取CSAF索引文件: {}", index_url);

        let response = self.client.get(index_url).send()?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .unwrap_or_else(|_| String::from("无法读取响应体"));
            return Err(FetchError::StatusError {
                status: status.as_u16(),
                body,
            });
        }

        let text = response.text()?;
        debug!("接收到索引文件，长度: {} 字节", text.len());

        // 解析index.txt文件，每行一个文件路径
        let paths: Vec<String> = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.ends_with(".json"))
            .map(|line| line.to_string())
            .collect();

        info!("从索引文件中解析出 {} 个CSAF文件路径", paths.len());
        Ok(paths)
    }

    /// 从index.txt文件批量获取所有CSAF文件
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址
    /// * `base_url` - CSAF文件的基础URL，例如: "http://csaf-website"
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表，每个元素包含(文件路径, 获取结果)
    pub fn fetch_from_index(
        &self,
        index_url: &str,
        base_url: &str,
    ) -> Result<Vec<(String, Result<CSAF>)>> {
        info!("从索引文件批量获取CSAF文件");
        info!("  索引URL: {}", index_url);
        info!("  基础URL: {}", base_url);

        // 获取文件路径列表
        let paths = self.fetch_index(index_url)?;

        // 确保base_url末尾没有斜杠
        let base_url = base_url.trim_end_matches('/');

        // 构建完整URL并获取文件
        let results: Vec<(String, Result<CSAF>)> = paths
            .iter()
            .map(|path| {
                let full_url = format!("{}/{}", base_url, path);
                debug!("获取CSAF文件: {}", full_url);
                let result = self.fetch(&full_url);
                (path.clone(), result)
            })
            .collect();

        // 统计结果
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let fail_count = results.len() - success_count;
        info!(
            "批量获取完成: 成功 {} 个, 失败 {} 个",
            success_count, fail_count
        );

        Ok(results)
    }

    /// 从index.txt文件批量获取并保存所有CSAF文件
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址
    /// * `base_url` - CSAF文件的基础URL
    /// * `output_dir` - 输出目录路径
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表
    pub fn fetch_from_index_and_save(
        &self,
        index_url: &str,
        base_url: &str,
        output_dir: &str,
    ) -> Result<Vec<(String, Result<()>)>> {
        info!("从索引文件批量获取并保存CSAF文件到: {}", output_dir);

        // 确保输出目录存在
        std::fs::create_dir_all(output_dir)?;

        // 获取所有CSAF文件
        let results = self.fetch_from_index(index_url, base_url)?;

        // 保存文件
        let save_results: Vec<(String, Result<()>)> = results
            .into_iter()
            .map(|(path, csaf_result)| {
                let save_result = match csaf_result {
                    Ok(csaf) => {
                        // 构建输出文件路径
                        let filename = path.replace('/', "_");
                        let output_path = format!("{}/{}", output_dir, filename);

                        // 保存文件
                        match serde_json::to_string_pretty(&csaf) {
                            Ok(json_str) => match std::fs::write(&output_path, json_str) {
                                Ok(_) => {
                                    info!("成功保存: {} -> {}", path, output_path);
                                    Ok(())
                                }
                                Err(e) => Err(FetchError::IoError(e)),
                            },
                            Err(e) => Err(FetchError::JsonError(e)),
                        }
                    }
                    Err(e) => Err(e),
                };
                (path, save_result)
            })
            .collect();

        // 统计结果
        let success_count = save_results.iter().filter(|(_, r)| r.is_ok()).count();
        let fail_count = save_results.len() - success_count;
        info!(
            "批量保存完成: 成功 {} 个, 失败 {} 个",
            success_count, fail_count
        );

        Ok(save_results)
    }
}

/// CSAF异步获取器
pub struct AsyncCsafFetcher {
    client: reqwest::Client,
    config: FetcherConfig,
}

impl AsyncCsafFetcher {
    /// 创建新的异步CSAF获取器
    ///
    /// # 参数
    ///
    /// * `config` - 配置选项
    ///
    /// # 返回值
    ///
    /// 返回Result<AsyncCsafFetcher>
    pub fn new(config: FetcherConfig) -> Result<Self> {
        info!("创建CSAF异步获取器，超时: {}秒", config.timeout_secs);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .build()?;

        Ok(Self { client, config })
    }

    /// 使用默认配置创建获取器
    pub fn with_defaults() -> Result<Self> {
        Self::new(FetcherConfig::default())
    }

    /// 从URL异步获取CSAF文件
    ///
    /// # 参数
    ///
    /// * `url` - CSAF文件的URL地址
    ///
    /// # 返回值
    ///
    /// 返回Result<CSAF>
    pub async fn fetch(&self, url: &str) -> Result<CSAF> {
        info!("异步获取CSAF文件: {}", url);

        // 验证URL
        let parsed_url = Url::parse(url)?;
        debug!("URL解析成功: {}", parsed_url);

        let mut last_error = None;

        // 重试逻辑
        for attempt in 1..=self.config.max_retries {
            debug!("异步尝试第 {} 次获取", attempt);

            match self.fetch_once(url).await {
                Ok(csaf) => {
                    info!(
                        "成功异步获取CSAF文件，漏洞数量: {}",
                        csaf.vulnerabilities.len()
                    );
                    return Ok(csaf);
                }
                Err(e) => {
                    warn!("异步第 {} 次获取失败: {}", attempt, e);
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }

        error!("所有异步重试均失败");
        Err(last_error.unwrap())
    }

    /// 单次异步获取（不重试）
    async fn fetch_once(&self, url: &str) -> Result<CSAF> {
        let response = self.client.get(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("无法读取响应体"));
            return Err(FetchError::StatusError {
                status: status.as_u16(),
                body,
            });
        }

        let text = response.text().await?;
        debug!("异步接收到响应，长度: {} 字节", text.len());

        let csaf: CSAF = serde_json::from_str(&text)?;
        Ok(csaf)
    }

    /// 从URL异步获取CSAF并保存到文件
    ///
    /// # 参数
    ///
    /// * `url` - CSAF文件的URL地址
    /// * `output_path` - 输出文件路径
    ///
    /// # 返回值
    ///
    /// 返回Result<CSAF>
    pub async fn fetch_and_save(&self, url: &str, output_path: &str) -> Result<CSAF> {
        info!("异步获取CSAF文件并保存到: {}", output_path);

        let csaf = self.fetch(url).await?;

        // 保存到文件
        let json_str = serde_json::to_string_pretty(&csaf)?;
        tokio::fs::write(output_path, json_str).await?;

        info!("成功异步保存CSAF文件到: {}", output_path);
        Ok(csaf)
    }

    /// 批量异步获取CSAF文件
    ///
    /// # 参数
    ///
    /// * `urls` - CSAF文件URL列表
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表
    pub async fn fetch_batch(&self, urls: &[String]) -> Vec<(String, Result<CSAF>)> {
        info!("批量异步获取 {} 个CSAF文件", urls.len());

        let mut results = Vec::new();

        for url in urls {
            let result = self.fetch(url).await;
            results.push((url.clone(), result));
        }

        results
    }

    /// 并发批量异步获取CSAF文件
    ///
    /// # 参数
    ///
    /// * `urls` - CSAF文件URL列表
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表
    pub async fn fetch_batch_concurrent(&self, urls: &[String]) -> Vec<(String, Result<CSAF>)> {
        info!("并发批量异步获取 {} 个CSAF文件", urls.len());

        let futures: Vec<_> = urls
            .iter()
            .map(|url| {
                let url_clone = url.clone();
                async move {
                    let result = self.fetch(&url_clone).await;
                    (url_clone, result)
                }
            })
            .collect();

        futures::future::join_all(futures).await
    }

    /// 从index.txt文件异步获取CSAF文件路径列表
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址
    ///
    /// # 返回值
    ///
    /// 返回CSAF文件相对路径列表
    pub async fn fetch_index(&self, index_url: &str) -> Result<Vec<String>> {
        info!("异步获取CSAF索引文件: {}", index_url);

        let response = self.client.get(index_url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("无法读取响应体"));
            return Err(FetchError::StatusError {
                status: status.as_u16(),
                body,
            });
        }

        let text = response.text().await?;
        debug!("接收到索引文件，长度: {} 字节", text.len());

        // 解析index.txt文件，每行一个文件路径
        let paths: Vec<String> = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.ends_with(".json"))
            .map(|line| line.to_string())
            .collect();

        info!("从索引文件中解析出 {} 个CSAF文件路径", paths.len());
        Ok(paths)
    }

    /// 从index.txt文件异步批量获取所有CSAF文件
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址
    /// * `base_url` - CSAF文件的基础URL
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表
    pub async fn fetch_from_index(
        &self,
        index_url: &str,
        base_url: &str,
    ) -> Result<Vec<(String, Result<CSAF>)>> {
        info!("从索引文件异步批量获取CSAF文件");
        info!("  索引URL: {}", index_url);
        info!("  基础URL: {}", base_url);

        // 获取文件路径列表
        let paths = self.fetch_index(index_url).await?;

        // 确保base_url末尾没有斜杠
        let base_url = base_url.trim_end_matches('/');

        // 构建完整URL并获取文件
        let mut results = Vec::new();
        for path in paths {
            let full_url = format!("{}/{}", base_url, path);
            debug!("异步获取CSAF文件: {}", full_url);
            let result = self.fetch(&full_url).await;
            results.push((path, result));
        }

        // 统计结果
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let fail_count = results.len() - success_count;
        info!(
            "异步批量获取完成: 成功 {} 个, 失败 {} 个",
            success_count, fail_count
        );

        Ok(results)
    }

    /// 从index.txt文件并发批量获取所有CSAF文件
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址
    /// * `base_url` - CSAF文件的基础URL
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表
    pub async fn fetch_from_index_concurrent(
        &self,
        index_url: &str,
        base_url: &str,
    ) -> Result<Vec<(String, Result<CSAF>)>> {
        info!("从索引文件并发批量获取CSAF文件");
        info!("  索引URL: {}", index_url);
        info!("  基础URL: {}", base_url);

        // 获取文件路径列表
        let paths = self.fetch_index(index_url).await?;

        // 确保base_url末尾没有斜杠
        let base_url = base_url.trim_end_matches('/');

        // 构建完整URL并并发获取文件
        let futures: Vec<_> = paths
            .into_iter()
            .map(|path| {
                let full_url = format!("{}/{}", base_url, path);
                let path_clone = path.clone();
                async move {
                    debug!("并发获取CSAF文件: {}", full_url);
                    let result = self.fetch(&full_url).await;
                    (path_clone, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        // 统计结果
        let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
        let fail_count = results.len() - success_count;
        info!(
            "并发批量获取完成: 成功 {} 个, 失败 {} 个",
            success_count, fail_count
        );

        Ok(results)
    }

    /// 从index.txt文件异步批量获取CSAF文件（带数据库检查）
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址
    /// * `base_url` - CSAF文件的基础URL
    /// * `check_exists` - 异步检查回调函数，用于检查文件是否已存在于数据库
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表，只包含实际下载的文件
    pub async fn fetch_from_index_with_check(
        &self,
        index_url: &str,
        base_url: &str,
        check_exists: AsyncCheckCallback,
    ) -> Result<Vec<(String, Result<CSAF>)>> {
        info!("从索引文件异步批量获取CSAF文件（带数据库检查）");
        info!("  索引URL: {}", index_url);
        info!("  基础URL: {}", base_url);

        // 获取文件路径列表
        let paths = self.fetch_index(index_url).await?;
        let total_count = paths.len();
        info!("从索引文件中解析出 {} 个文件路径", total_count);

        // 确保base_url末尾没有斜杠
        let base_url = base_url.trim_end_matches('/');

        // 遍历每个文件路径，检查是否存在
        let mut results = Vec::new();
        let mut skipped_count = 0;
        let mut downloaded_count = 0;

        for path in paths {
            // 调用检查回调函数
            let exists = check_exists(path.clone()).await;

            if exists {
                debug!("文件已存在，跳过: {}", path);
                skipped_count += 1;
                continue;
            }

            // 文件不存在，下载
            debug!("文件不存在，开始下载: {}", path);
            let full_url = format!("{}/{}", base_url, path);
            let result = self.fetch(&full_url).await;

            if result.is_ok() {
                downloaded_count += 1;
            }

            results.push((path, result));
        }

        info!(
            "批量获取完成: 总计 {} 个，跳过 {} 个，下载 {} 个，成功 {} 个",
            total_count,
            skipped_count,
            downloaded_count,
            results.iter().filter(|(_, r)| r.is_ok()).count()
        );

        Ok(results)
    }

    /// 从index.txt文件异步批量获取并保存所有CSAF文件
    ///
    /// # 参数
    ///
    /// * `index_url` - index.txt文件的URL地址
    /// * `base_url` - CSAF文件的基础URL
    /// * `output_dir` - 输出目录路径
    ///
    /// # 返回值
    ///
    /// 返回成功和失败的结果列表
    pub async fn fetch_from_index_and_save(
        &self,
        index_url: &str,
        base_url: &str,
        output_dir: &str,
    ) -> Result<Vec<(String, Result<()>)>> {
        info!("从索引文件异步批量获取并保存CSAF文件到: {}", output_dir);

        // 确保输出目录存在
        tokio::fs::create_dir_all(output_dir).await?;

        // 获取所有CSAF文件
        let results = self.fetch_from_index(index_url, base_url).await?;

        // 保存文件
        let mut save_results = Vec::new();
        for (path, csaf_result) in results {
            let save_result = match csaf_result {
                Ok(csaf) => {
                    // 构建输出文件路径
                    let filename = path.replace('/', "_");
                    let output_path = format!("{}/{}", output_dir, filename);

                    // 保存文件
                    match serde_json::to_string_pretty(&csaf) {
                        Ok(json_str) => match tokio::fs::write(&output_path, json_str).await {
                            Ok(_) => {
                                info!("成功异步保存: {} -> {}", path, output_path);
                                Ok(())
                            }
                            Err(e) => Err(FetchError::IoError(e)),
                        },
                        Err(e) => Err(FetchError::JsonError(e)),
                    }
                }
                Err(e) => Err(e),
            };
            save_results.push((path, save_result));
        }

        // 统计结果
        let success_count = save_results.iter().filter(|(_, r)| r.is_ok()).count();
        let fail_count = save_results.len() - success_count;
        info!(
            "异步批量保存完成: 成功 {} 个, 失败 {} 个",
            success_count, fail_count
        );

        Ok(save_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetcher_config_default() {
        let config = FetcherConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.user_agent, "CSAF-Fetcher/0.1.0");
    }

    #[test]
    fn test_fetcher_creation() {
        let config = FetcherConfig::default();
        let fetcher = CsafFetcher::new(config);
        assert!(fetcher.is_ok());
    }

    #[test]
    fn test_fetcher_with_defaults() {
        let fetcher = CsafFetcher::with_defaults();
        assert!(fetcher.is_ok());
    }

    #[test]
    fn test_url_validation() {
        let fetcher = CsafFetcher::with_defaults().unwrap();

        // 测试无效URL
        let result = fetcher.fetch("invalid-url");
        assert!(result.is_err());

        if let Err(FetchError::UrlError(_)) = result {
            // 预期的错误类型
        } else {
            panic!("Expected UrlError");
        }
    }

    #[tokio::test]
    async fn test_async_fetcher_creation() {
        let config = FetcherConfig::default();
        let fetcher = AsyncCsafFetcher::new(config);
        assert!(fetcher.is_ok());
    }

    #[tokio::test]
    async fn test_async_fetcher_with_defaults() {
        let fetcher = AsyncCsafFetcher::with_defaults();
        assert!(fetcher.is_ok());
    }

    #[test]
    fn test_parse_index_content() {
        // 模拟index.txt文件内容
        let index_content = r#"2021/csaf-openeuler-sa-2021-1001.json
2021/csaf-openeuler-sa-2021-1002.json
2021/csaf-openeuler-sa-2021-1003.json

2022/csaf-openeuler-sa-2022-1001.json
"#;

        let paths: Vec<String> = index_content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.ends_with(".json"))
            .map(|line| line.to_string())
            .collect();

        assert_eq!(paths.len(), 4);
        assert_eq!(paths[0], "2021/csaf-openeuler-sa-2021-1001.json");
        assert_eq!(paths[1], "2021/csaf-openeuler-sa-2021-1002.json");
        assert_eq!(paths[2], "2021/csaf-openeuler-sa-2021-1003.json");
        assert_eq!(paths[3], "2022/csaf-openeuler-sa-2022-1001.json");
    }

    #[test]
    fn test_url_construction() {
        let base_url = "http://csaf-website";
        let path = "2021/csaf-openeuler-sa-2021-1001.json";

        // 确保末尾没有斜杠
        let base_url = base_url.trim_end_matches('/');
        let full_url = format!("{}/{}", base_url, path);

        assert_eq!(
            full_url,
            "http://csaf-website/2021/csaf-openeuler-sa-2021-1001.json"
        );
    }

    #[test]
    fn test_url_construction_with_trailing_slash() {
        let base_url = "http://csaf-website/";
        let path = "2021/csaf-openeuler-sa-2021-1001.json";

        // 确保末尾没有斜杠
        let base_url = base_url.trim_end_matches('/');
        let full_url = format!("{}/{}", base_url, path);

        assert_eq!(
            full_url,
            "http://csaf-website/2021/csaf-openeuler-sa-2021-1001.json"
        );
    }

    #[test]
    fn test_filename_conversion() {
        // 测试路径转换为文件名
        let path = "2021/csaf-openeuler-sa-2021-1001.json";
        let filename = path.replace('/', "_");

        assert_eq!(filename, "2021_csaf-openeuler-sa-2021-1001.json");
    }

    #[test]
    fn test_parse_index_with_empty_lines() {
        // 测试包含空行的index.txt
        let index_content = r#"
2021/csaf-openeuler-sa-2021-1001.json

2021/csaf-openeuler-sa-2021-1002.json

2021/csaf-openeuler-sa-2021-1003.json

"#;

        let paths: Vec<String> = index_content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.ends_with(".json"))
            .map(|line| line.to_string())
            .collect();

        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_parse_index_filters_non_json() {
        // 测试过滤非.json文件
        let index_content = r#"2021/csaf-openeuler-sa-2021-1001.json
README.md
2021/csaf-openeuler-sa-2021-1002.json
index.txt
2021/csaf-openeuler-sa-2021-1003.json
"#;

        let paths: Vec<String> = index_content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.ends_with(".json"))
            .map(|line| line.to_string())
            .collect();

        assert_eq!(paths.len(), 3);
        assert!(paths.iter().all(|p| p.ends_with(".json")));
    }

    // 注意：以下测试需要实际的网络连接和有效的CSAF URL
    // 在实际使用时，应该使用mock服务器或测试数据

    /*
    #[test]
    fn test_fetch_real_csaf() {
        let fetcher = CsafFetcher::with_defaults().unwrap();
        let url = "https://example.com/csaf/example.json";
        let result = fetcher.fetch(url);
        // 根据实际情况验证结果
    }

    #[tokio::test]
    async fn test_async_fetch_real_csaf() {
        let fetcher = AsyncCsafFetcher::with_defaults().unwrap();
        let url = "https://example.com/csaf/example.json";
        let result = fetcher.fetch(url).await;
        // 根据实际情况验证结果
    }
    */
}
