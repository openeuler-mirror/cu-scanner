//! CSAF(Common Security Advisory Framework)数据结构定义
//!
//! 该模块定义了CSAF(Common Security Advisory Framework)标准的数据结构，
//! 用于描述安全漏洞信息、产品信息以及相关的修复建议。

use log::{error, info};
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use utils::Result;

/// CSAF整体结构体包含三个部分：
/// - document: 文档
/// - product_tree: 产品相关信息
/// - vulnerabilities: 漏洞相关描述
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CSAF {
    pub document: Document,
    pub product_tree: ProductTree,
    pub vulnerabilities: Vec<Vulnerabilitie>,
}

/// Document: 文档相关的描述
///
/// 包含以下字段：
/// - aggregate_severity: 严重性信息
/// - category: 类别
/// - distribution: 分发类型
/// - lang: 语言
/// - notes: 文档描述信息
/// - publisher: 发布者信息
/// - references: 引用信息，比如CVE、NVD链接信息
/// - title: 标题
/// - tracking: 漏洞跟踪信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Document {
    pub aggregate_severity: AggregateSeverity,
    pub category: String,
    pub csaf_version: String,
    pub distribution: Distribution,
    pub lang: String,
    pub notes: Vec<Note>,
    pub publisher: Publisher,
    pub references: Vec<Reference>,
    pub title: String,
    pub tracking: Tracking,
}

/// CVE严重性信息
///
/// 包含以下字段：
/// - namespace: 漏洞严重程度参考信息
/// - text: 严重程度 Low、Medium、High
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AggregateSeverity {
    pub namespace: String,
    pub text: String,
}

/// 分发信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Distribution {
    pub tlp: Tlp,
}

/// Tlp: 交通灯协议，此协议用于信息共享和传递的安全协议
///
/// 包含以下字段：
/// - label: 颜色标签:
///     - WHITE: 信息可公开共享，几乎没有限制
///     - GREEN: 信息可以在更广泛的范围内共享，需要保持一定的保密性，不可
///              公开发布
///     - AMBER: 可以共享给指定的受信任方，不能公开传播
///     - RED: 此信息仅限于接收者个人使用，不能共享或者传播给其他任何人。
/// - url: 链接，一般是交通灯协议的网址
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tlp {
    pub label: String,
    pub url: String,
}

/// 文档信息
///
/// 包含以下字段：
/// - text: 漏洞信息描述
/// - category: 分类信息
/// - title: 标题
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Note {
    pub text: String,
    pub category: String,
    pub title: String,
}

/// 发布者信息
///
/// 包含以下字段：
/// - issuing_authority: 发布者签发团队
/// - name: 发布者团体
/// - namespace: 发布者的官网
/// - contact_details: 发布者联系方式
/// - category: 类别
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Publisher {
    pub issuing_authority: String,
    pub name: String,
    pub namespace: String,
    pub contact_details: String,
    pub category: String,
}

/// 链接信息
///
/// 包含以下字段：
/// - summary: 简要说明
/// - category: 类别
/// - url: 参考链接
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Reference {
    pub summary: String,
    pub category: String,
    pub url: String,
}

/// 跟踪信息
///
/// 包含以下字段：
/// - initial_release_date: 初始的发布时间
/// - revision_history: 历史信息
/// - generator: 生成报告的信息
/// - current_release_date: 当前发布日期
/// - id: CVE ID
/// - version: 版本
/// - status: 状态
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tracking {
    pub initial_release_date: String,
    pub revision_history: Vec<History>,
    pub generator: Generator,
    pub current_release_date: String,
    pub id: String,
    pub version: String,
    pub status: String,
}

/// 历史信息
///
/// 包含以下字段：
/// - date: 时间
/// - summary: 简要说明
/// - number: 版本号信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct History {
    pub date: String,
    pub summary: String,
    pub number: String,
}

/// 生成信息
///
/// 包含以下字段：
/// - date: 时间
/// - engine: 生成的引擎
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Generator {
    pub date: String,
    pub engine: Engine,
}

/// 文档生成引擎信息
///
/// 包含以下字段：
/// - name: 引擎名字
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Engine {
    pub name: String,
}

/// 产品信息
///
/// 包含以下字段：
/// - branches: 分支信息
/// - relationships: 关系信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProductTree {
    pub branches: Vec<Branches>,
    pub relationships: Vec<RelationShip>,
}

/// 分支信息，此分支信息包含嵌套的分支
///
/// 包含以下字段：
/// - name: 产品名字
/// - category: 分类
/// - branches: 子分支
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Branches {
    pub name: String,
    pub category: String,
    pub branches: Vec<Branch>,
}

/// 分支信息
///
/// 包含以下字段：
/// - name: 产品名字
/// - category: 分类
/// - branches: 子分支信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Branch {
    pub name: String,
    pub category: String,
    pub branches: Vec<SubBranch>,
}

/// 子分支信息
///
/// 包含以下字段：
/// - product: 产品信息
/// - name: 子产品名字
/// - category: 分类信息，包含产品分支、软件包分支
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubBranch {
    pub product: Product,
    pub name: String,
    pub category: String,
}

/// 产品信息
///
/// 包含以下字段：
/// - product_identification_helper: 产品分类区分信息
/// - product_id: 最终系统、软件包名字
/// - name: 最终系统、软件包名字
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Product {
    pub product_identification_helper: ProductIdentificationHelper,
    pub product_id: String,
    pub name: String,
}

/// 产品分类区分信息
///
/// 包含以下字段：
/// - cpe: cpe信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProductIdentificationHelper {
    pub cpe: String,
}

/// 关系信息
///
/// 包含以下字段：
/// - relates_to_product_reference: 关系到产品信息
/// - product_reference: 产品，软件、系统信息
/// - full_product_name: 产品信息
/// - category: 分类信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelationShip {
    pub relates_to_product_reference: String,
    pub product_reference: String,
    pub full_product_name: FullProductName,
    pub category: String,
}

/// 产品信息
///
/// 包含以下字段：
/// - product_id: 产品信息，系统名字信息
/// - name: 产品名字，软件包名字
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullProductName {
    pub product_id: String,
    pub name: String,
}

/// 漏洞信息
///
/// 包含以下字段：
/// - cve: CVE ID
/// - notes: 漏洞描述
/// - product_status: 产品状态
/// - remediations: 整改信息
/// - scores: cve分数信息
/// - threats: 威胁信息
/// - title: 标题
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vulnerabilitie {
    pub cve: String,
    pub notes: Vec<VulNote>,
    pub product_status: ProductStatus,
    pub remediations: Vec<Remediation>,
    pub scores: Vec<Score>,
    pub threats: Vec<VulThreat>,
    pub title: String,
}

/// 漏洞描述
///
/// 包含以下字段：
/// - text: 详细描述
/// - category: note的类别
/// - title: 标题
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VulNote {
    pub text: String,
    pub category: String,
    pub title: String,
}

/// 产品信息
///
/// 包含以下字段：
/// - fixed: 修正的软件包版本全部信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProductStatus {
    pub fixed: Vec<String>,
}

/// 整改信息
///
/// 包含以下字段：
/// - product_ids: 产品信息，包含系统版本、涉及软件包信息
/// - details: 本产品描述
/// - category: 类别
/// - url: 参考链接
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Remediation {
    pub product_ids: Vec<String>,
    pub details: String,
    pub category: String,
    pub url: String,
}

/// CVE得分信息
///
/// 包含以下字段：
/// - cvss_v3: CVE的CVSS评分信息
/// - products: 产品信息，系统、软件包信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Score {
    pub cvss_v3: CvssV3,
    pub products: Vec<String>,
}

/// Cvss相关信息
///
/// 包含以下字段：
/// - baseSeverity: 严重性信息
/// - baseScore: 分数信息
/// - vectorString: 向量信息
/// - version: 版本信息
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CvssV3 {
    #[serde(rename = "baseSeverity")]
    pub base_severity: String,

    #[serde(rename = "baseScore")]
    pub base_score: f32,

    #[serde(rename = "vectorString")]
    pub vector_string: String,
    pub version: String,
}

/// 漏洞威胁信息
///
/// 包含以下字段：
/// - details: 详细信息，严重程度
/// - category: 分类，是否受影响等
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VulThreat {
    pub details: String,
    pub category: String,
}

impl CSAF {
    /// 创建新的空CSAF实例
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            product_tree: ProductTree::new(),
            vulnerabilities: Vec::new(),
        }
    }

    /// 获取CSAF文档的ID
    pub fn get_id(&self) -> &str {
        &self.document.tracking.id
    }

    /// 获取CSAF文档的版本
    pub fn get_version(&self) -> &str {
        &self.document.tracking.version
    }

    /// 获取CSAF文档的标题
    pub fn get_title(&self) -> &str {
        &self.document.title
    }

    /// 获取漏洞数量
    pub fn get_vulnerability_count(&self) -> usize {
        self.vulnerabilities.len()
    }

    /// 获取所有CVE ID列表
    pub fn get_cve_ids(&self) -> Vec<&str> {
        self.vulnerabilities
            .iter()
            .map(|v| v.cve.as_str())
            .collect()
    }

    /// 获取文档状态
    pub fn get_status(&self) -> &str {
        &self.document.tracking.status
    }

    /// 获取发布日期
    pub fn get_release_date(&self) -> &str {
        &self.document.tracking.current_release_date
    }

    /// 获取初始发布日期
    pub fn get_initial_release_date(&self) -> &str {
        &self.document.tracking.initial_release_date
    }

    /// 检查是否包含指定的CVE
    pub fn contains_cve(&self, cve_id: &str) -> bool {
        self.vulnerabilities.iter().any(|v| v.cve == cve_id)
    }

    /// 从文件路径加载CSAF数据
    ///
    /// # 参数
    ///
    /// * `path` - CSAF文件路径
    ///
    /// # 返回值
    ///
    /// 返回Result<CSAF>，成功时包含解析的CSAF对象，失败时包含错误信息
    pub fn from_file(path: &str) -> Result<Self> {
        info!("从文件加载CSAF数据: {}", path);
        let data = std::fs::read_to_string(path)?;
        let csaf: Self = serde_json::from_str::<Self>(&data)?;
        info!("成功加载CSAF数据，漏洞数量: {}", csaf.vulnerabilities.len());
        Ok(csaf)
    }

    /// 将CSAF数据保存到文件
    ///
    /// # 参数
    ///
    /// * `self` - CSAF对象
    /// * `path` - 保存文件路径
    ///
    /// # 返回值
    ///
    /// 返回Result<()>，成功时返回()，失败时包含错误信息
    pub fn to_file(self, path: &str) -> Result<()> {
        info!("将CSAF数据保存到文件: {}", path);
        let data = serde_json::to_string_pretty(&self)?;
        let res = std::fs::write(path, &data);
        match res {
            Ok(_) => {
                info!("成功保存CSAF数据到文件: {}", path);
                Ok(())
            }
            Err(e) => {
                error!("保存CSAF数据到文件失败: {}", e);
                Err(e.into())
            }
        }
    }

    /// 从URL加载CSAF数据
    ///
    /// # 参数
    ///
    /// * `url` - CSAF数据的URL地址
    ///
    /// # 返回值
    ///
    /// 返回Result<CSAF>，成功时包含解析的CSAF对象，失败时包含错误信息
    pub fn from_url(url: &str) -> Result<Self> {
        info!("从URL加载CSAF数据: {}", url);
        let response = get(url)?;
        if response.status().is_success() {
            let body = response.text()?;
            let csaf: Self = serde_json::from_str::<Self>(&body)?;
            info!(
                "成功从URL加载CSAF数据，漏洞数量: {}",
                csaf.vulnerabilities.len()
            );
            Ok(csaf)
        } else {
            let error_data = format!("Get csaf data file from {} error", url);
            error!("从URL加载CSAF数据失败: {}", error_data);
            Err(error_data.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 从配置文件读取测试文件路径
    fn get_test_file_path(filename: &str) -> String {
        // 在测试环境中，直接使用相对路径
        format!("../test/csaf/{}", filename)
    }

    #[test]
    fn conver_test() {
        let test_file = get_test_file_path("csaf-openeuler-sa-2025-1004.json");
        match CSAF::from_file(&test_file) {
            Ok(csaf) => println!("{:#?}", csaf),
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn write_test() {
        let input_file = get_test_file_path("csaf-openeuler-sa-2025-1004.json");
        let output_file = get_test_file_path("csaf_test_001.json");

        let csaf = match CSAF::from_file(&input_file) {
            Ok(csaf) => csaf,
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
                return;
            }
        };
        match csaf.to_file(&output_file) {
            Ok(_) => assert!(true),
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
            }
        }
    }
}

// 为其他结构体添加实现

impl Document {
    /// 创建新的空Document实例
    pub fn new() -> Self {
        Self {
            aggregate_severity: AggregateSeverity::new(),
            category: String::new(),
            csaf_version: "2.0".to_string(),
            distribution: Distribution::new(),
            lang: "en".to_string(),
            notes: Vec::new(),
            publisher: Publisher::new(),
            references: Vec::new(),
            title: String::new(),
            tracking: Tracking::new(),
        }
    }

    /// 获取文档类别
    pub fn get_category(&self) -> &str {
        &self.category
    }

    /// 获取文档语言
    pub fn get_lang(&self) -> &str {
        &self.lang
    }

    /// 获取发布者名称
    pub fn get_publisher_name(&self) -> &str {
        &self.publisher.name
    }
}

impl AggregateSeverity {
    /// 创建新的空AggregateSeverity实例
    pub fn new() -> Self {
        Self {
            namespace: String::new(),
            text: String::new(),
        }
    }

    /// 获取严重性级别
    pub fn get_severity(&self) -> &str {
        &self.text
    }

    /// 检查是否为严重级别
    pub fn is_critical(&self) -> bool {
        self.text.to_lowercase() == "critical"
    }

    /// 检查是否为高危级别
    pub fn is_high(&self) -> bool {
        self.text.to_lowercase() == "high"
    }
}

impl Distribution {
    /// 创建新的Distribution实例
    pub fn new() -> Self {
        Self { tlp: Tlp::new() }
    }
}

impl Tlp {
    /// 创建新的Tlp实例，默认为WHITE
    pub fn new() -> Self {
        Self {
            label: "WHITE".to_string(),
            url: "https://www.first.org/tlp/".to_string(),
        }
    }

    /// 检查是否可以公开共享
    pub fn is_public(&self) -> bool {
        self.label.to_uppercase() == "WHITE"
    }
}

impl Publisher {
    /// 创建新的Publisher实例
    pub fn new() -> Self {
        Self {
            issuing_authority: String::new(),
            name: String::new(),
            namespace: String::new(),
            contact_details: String::new(),
            category: String::new(),
        }
    }
}

impl Tracking {
    /// 创建新的Tracking实例
    pub fn new() -> Self {
        Self {
            initial_release_date: String::new(),
            revision_history: Vec::new(),
            generator: Generator::new(),
            current_release_date: String::new(),
            id: String::new(),
            version: String::new(),
            status: String::new(),
        }
    }

    /// 获取修订历史数量
    pub fn get_revision_count(&self) -> usize {
        self.revision_history.len()
    }

    /// 获取最新修订信息
    pub fn get_latest_revision(&self) -> Option<&History> {
        self.revision_history.last()
    }
}

impl Generator {
    /// 创建新的Generator实例
    pub fn new() -> Self {
        Self {
            date: String::new(),
            engine: Engine::new(),
        }
    }
}

impl Engine {
    /// 创建新的Engine实例
    pub fn new() -> Self {
        Self {
            name: String::new(),
        }
    }
}

impl ProductTree {
    /// 创建新的ProductTree实例
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            relationships: Vec::new(),
        }
    }

    /// 获取所有产品ID列表
    pub fn get_all_product_ids(&self) -> Vec<String> {
        let mut product_ids = Vec::new();

        // 从branches中收集产品ID
        for branch in &self.branches {
            for sub_branch in &branch.branches {
                for leaf in &sub_branch.branches {
                    product_ids.push(leaf.product.product_id.clone());
                }
            }
        }

        // 从relationships中收集产品ID
        for rel in &self.relationships {
            product_ids.push(rel.full_product_name.product_id.clone());
        }

        product_ids
    }

    /// 获取产品数量
    pub fn get_product_count(&self) -> usize {
        self.get_all_product_ids().len()
    }
}

impl Vulnerabilitie {
    /// 创建新的Vulnerabilitie实例
    pub fn new() -> Self {
        Self {
            cve: String::new(),
            notes: Vec::new(),
            product_status: ProductStatus::new(),
            remediations: Vec::new(),
            scores: Vec::new(),
            threats: Vec::new(),
            title: String::new(),
        }
    }

    /// 获取CVE ID
    pub fn get_cve_id(&self) -> &str {
        &self.cve
    }

    /// 获取标题
    pub fn get_title(&self) -> &str {
        &self.title
    }

    /// 获取受影响的产品数量
    pub fn get_affected_product_count(&self) -> usize {
        self.product_status.fixed.len()
    }

    /// 获取CVSS分数（如果有）
    pub fn get_cvss_score(&self) -> Option<f32> {
        self.scores.first().map(|s| s.cvss_v3.base_score)
    }

    /// 获取严重性级别（如果有）
    pub fn get_severity(&self) -> Option<&str> {
        self.scores
            .first()
            .map(|s| s.cvss_v3.base_severity.as_str())
    }

    /// 检查是否为严重漏洞
    pub fn is_critical(&self) -> bool {
        self.get_severity()
            .map(|s| s.to_lowercase() == "critical")
            .unwrap_or(false)
    }

    /// 检查是否为高危漏洞
    pub fn is_high(&self) -> bool {
        self.get_severity()
            .map(|s| s.to_lowercase() == "high")
            .unwrap_or(false)
    }
}

impl ProductStatus {
    /// 创建新的ProductStatus实例
    pub fn new() -> Self {
        Self { fixed: Vec::new() }
    }

    /// 获取已修复的产品列表
    pub fn get_fixed_products(&self) -> &[String] {
        &self.fixed
    }

    /// 检查产品是否已修复
    pub fn is_product_fixed(&self, product_id: &str) -> bool {
        self.fixed.iter().any(|p| p == product_id)
    }
}

impl Score {
    /// 获取CVSS基础分数
    pub fn get_base_score(&self) -> f32 {
        self.cvss_v3.base_score
    }

    /// 获取严重性级别
    pub fn get_severity(&self) -> &str {
        &self.cvss_v3.base_severity
    }

    /// 获取向量字符串
    pub fn get_vector_string(&self) -> &str {
        &self.cvss_v3.vector_string
    }
}

impl CvssV3 {
    /// 创建新的CvssV3实例
    pub fn new() -> Self {
        Self {
            base_severity: String::new(),
            base_score: 0.0,
            vector_string: String::new(),
            version: "3.1".to_string(),
        }
    }

    /// 检查是否为严重级别
    pub fn is_critical(&self) -> bool {
        self.base_severity.to_lowercase() == "critical" || self.base_score >= 9.0
    }

    /// 检查是否为高危级别
    pub fn is_high(&self) -> bool {
        self.base_severity.to_lowercase() == "high"
            || (self.base_score >= 7.0 && self.base_score < 9.0)
    }

    /// 检查是否为中危级别
    pub fn is_medium(&self) -> bool {
        self.base_severity.to_lowercase() == "medium"
            || (self.base_score >= 4.0 && self.base_score < 7.0)
    }

    /// 检查是否为低危级别
    pub fn is_low(&self) -> bool {
        self.base_severity.to_lowercase() == "low"
            || (self.base_score > 0.0 && self.base_score < 4.0)
    }
}
