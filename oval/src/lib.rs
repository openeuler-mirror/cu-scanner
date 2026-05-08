//! OVAL(Open Vulnerability and Assessment Language)数据结构定义
//!
//! 该模块定义了OVAL格式的数据结构，用于描述安全漏洞检查规则。

use const_format::concatcp;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

/// 通用错误类型
pub type Error = Box<dyn std::error::Error + Send + Sync>;
/// 通用结果类型
pub type Result<T> = std::result::Result<T, Error>;

/// XML命名空间
pub const XMLNS: &str = "http://oval.mitre.org/XMLSchema/oval-definitions-5";
/// OVAL命名空间
pub const OVAL: &str = "http://oval.mitre.org/XMLSchema/oval-common-5";
/// UNIX定义命名空间
pub const UNIX_DEF: &str = "http://oval.mitre.org/XMLSchema/oval-definitions-5#unix";
/// RedHat定义命名空间
pub const RED_DEF: &str = "http://oval.mitre.org/XMLSchema/oval-definitions-5#linux";
/// 独立定义命名空间
pub const IND_DEF: &str = "http://oval.mitre.org/XMLSchema/oval-definitions-5#independent";
/// XML Schema实例命名空间
pub const XMLNS_XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";
/// XSI Schema位置
pub const XSI_SCHEMALOCATION: &str = "http://oval.mitre.org/XMLSchema/oval-common-5 oval-common-schema.xsd http://oval.mitre.org/XMLSchema/oval-definitions-5 oval-definitions-schema.xsd http://oval.mitre.org/XMLSchema/oval-definitions-5#unix unix-definitions-schema.xsd http://oval.mitre.org/XMLSchema/oval-definitions-5#linux linux-definitions-schema.xsd";

/// 默认产品名称
pub const DEF_PRODUCT_NAME: &str = "China Unicom Linux";
/// 默认产品版本
pub const DEF_PRODUCT_VERSION: &str = "4";
/// 默认Schema版本
pub const DEF_SCHEMA_VERSION: &str = "5.10";
/// 中国联通Linux参考前缀
pub const CU_LINUX_REF_PREFIX: &str = "oval:cn.chinaunicom.culinux";
// CUBA: BUG相关的更新定义
/// CUBA前缀
pub const CU_LINUX_CUBA_PREFIX: &str = concatcp!(CU_LINUX_REF_PREFIX, ".cuba");
/// BA定义前缀
pub const CU_LINUX_BA_DEF_PREFIX: &str = concatcp!(CU_LINUX_CUBA_PREFIX, ":def:");
/// BA对象前缀
pub const CU_LINUX_BA_OBJ_PREFIX: &str = concatcp!(CU_LINUX_CUBA_PREFIX, ":obj:");
/// BA测试前缀
pub const CU_LINUX_BA_TST_PREFIX: &str = concatcp!(CU_LINUX_CUBA_PREFIX, ":tst:");
/// BA状态前缀
pub const CU_LINUX_BA_STE_PREFIX: &str = concatcp!(CU_LINUX_CUBA_PREFIX, ":ste:");
// CUSA: 安全相关更新定义
/// CUSA前缀
pub const CU_LINUX_CUSA_PREFIX: &str = concatcp!(CU_LINUX_REF_PREFIX, ".cusa");
/// SA定义前缀
pub const CU_LINUX_SA_DEF_PREFIX: &str = concatcp!(CU_LINUX_CUSA_PREFIX, ":def:");
/// SA对象前缀
pub const CU_LINUX_SA_OBJ_PREFIX: &str = concatcp!(CU_LINUX_CUSA_PREFIX, ":obj:");
/// SA测试前缀
pub const CU_LINUX_SA_TST_PREFIX: &str = concatcp!(CU_LINUX_CUSA_PREFIX, ":tst:");
/// SA状态前缀
pub const CU_LINUX_SA_STE_PREFIX: &str = concatcp!(CU_LINUX_CUSA_PREFIX, ":ste:");

// China Unicom CULinux 版权相关的定义
/// 版权信息
pub const CU_LINUX_COPY_RIGHT: &str = "Copyright 2025 China Unicom, Inc.";

/// 建议来源邮箱
pub const ADVISORY_FROM: &str = "security@chinaunicom.cn";

/// OVAL定义根结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "oval_definitions")]
pub struct OvalDefinitions {
    /// Schema位置
    #[serde(rename = "@xsi:schemaLocation")]
    pub schema_location: String,

    /// 生成器信息
    pub generator: Generator,
    /// 定义列表
    pub definitions: Definitions,
    /// 测试列表
    pub tests: Tests,
    /// 对象列表
    pub objects: Objects,
    /// 状态列表
    pub states: States,
}

impl Default for OvalDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

impl OvalDefinitions {
    /// 创建新的OvalDefinitions实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的OvalDefinitions实例，包含默认值
    pub fn new() -> Self {
        info!("创建新的OvalDefinitions实例");
        Self {
            schema_location: XSI_SCHEMALOCATION.to_string(),
            generator: Generator::new(),
            definitions: Definitions::new(),
            tests: Tests::new(),
            objects: Objects::new(),
            states: States::new(),
        }
    }

    /// 添加定义
    pub fn add_definition(&mut self, definition: Definition) {
        debug!("添加定义: {}", definition.id);
        self.definitions.items.push(definition);
    }

    /// 获取定义数量
    pub fn get_definition_count(&self) -> usize {
        self.definitions.items.len()
    }

    /// 添加测试
    pub fn add_rpminfo_test(&mut self, test: RpmInfoTest) {
        debug!("添加RPM信息测试: {}", test.id);
        self.tests.rpminfo_tests.push(test);
    }

    /// 获取测试数量
    pub fn get_test_count(&self) -> usize {
        self.tests.rpminfo_tests.len()
    }

    /// 添加对象
    pub fn add_rpm_info_object(&mut self, object: RpmInfoObject) {
        debug!("添加RPM信息对象: {}", object.id);
        self.objects.add_rpm_info(object);
    }

    /// 获取对象数量
    pub fn get_object_count(&self) -> usize {
        self.objects.len()
    }

    /// 添加状态
    pub fn add_rpminfo_state(&mut self, state: RpmInfoState) {
        debug!("添加RPM信息状态: {}", state.id);
        if let Some(ref mut states) = self.states.rpminfo_states {
            states.push(state);
        } else {
            self.states.rpminfo_states = Some(vec![state]);
        }
    }

    /// 获取状态数量
    pub fn get_state_count(&self) -> usize {
        self.states
            .rpminfo_states
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// 检查是否为空（没有任何定义）
    pub fn is_empty(&self) -> bool {
        self.definitions.items.is_empty()
    }

    /// 清空所有内容
    pub fn clear(&mut self) {
        info!("清空所有OVAL定义内容");
        self.definitions.items.clear();
        self.tests.rpminfo_tests.clear();
        self.objects.clear();
        self.states.rpminfo_states = None;
    }

    /// 将OVAL定义转换为XML字符串
    ///
    /// # 返回值
    ///
    /// 返回Result<String>，成功时包含格式化的XML字符串，失败时包含错误信息
    pub fn to_oval_string(&self) -> Result<String> {
        info!("将OVAL定义转换为XML字符串");
        // for serd_xml_rs we need add xmlns for oval
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("", XMLNS)
            .namespace("oval", OVAL)
            .namespace("unix-def", UNIX_DEF)
            .namespace("red-def", RED_DEF)
            .namespace("xsi", XMLNS_XSI)
            .namespace("ind-def", IND_DEF);
        let oval_res = config.to_string(&self);
        match oval_res {
            Ok(oval) => {
                debug!("成功转换OVAL定义为XML字符串，长度: {}", oval.len());
                Ok(oval)
            }
            Err(error) => {
                error!("转换OVAL定义为XML字符串失败: {}", error);
                Err(error.into())
            }
        }
        //TODO:
        // oval xml string format and beatify the string
    }

    /// 保存到文件
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        info!("保存OVAL定义到文件: {}", path);
        let xml_string = self.to_oval_string()?;
        std::fs::write(path, xml_string)?;
        info!("成功保存OVAL定义到文件: {}", path);
        Ok(())
    }

    /// 合并另一个 OvalDefinitions 到当前实例
    ///
    /// 会自动去重 definitions、tests、objects、states，避免ID冲突
    ///
    /// # 参数
    ///
    /// * `other` - 要合并的另一个 OvalDefinitions 实例
    ///
    /// # 返回值
    ///
    /// 返回 Result<()>，成功时为 Ok(())，失败时包含错误信息
    pub fn merge(&mut self, other: OvalDefinitions) -> Result<()> {
        info!("开始合并 OvalDefinitions，当前 definitions: {}, 新增 definitions: {}",
              self.definitions.items.len(), other.definitions.items.len());

        use std::collections::HashSet;

        // 1. 合并 definitions（去重）
        let mut existing_def_ids: HashSet<String> =
            self.definitions.items.iter().map(|d| d.id.clone()).collect();

        for def in other.definitions.items {
            if !existing_def_ids.contains(&def.id) {
                debug!("添加新的 definition: {}", def.id);
                existing_def_ids.insert(def.id.clone());
                self.definitions.items.push(def);
            } else {
                debug!("跳过重复的 definition: {}", def.id);
            }
        }

        // 2. 合并 tests（去重）
        let mut existing_test_ids: HashSet<String> =
            self.tests.rpminfo_tests.iter().map(|t| t.id.clone()).collect();

        for test in other.tests.rpminfo_tests {
            if !existing_test_ids.contains(&test.id) {
                debug!("添加新的 rpminfo_test: {}", test.id);
                existing_test_ids.insert(test.id.clone());
                self.tests.rpminfo_tests.push(test);
            } else {
                debug!("跳过重复的 rpminfo_test: {}", test.id);
            }
        }

        // 合并 rpmverifyfile_tests
        let mut existing_verify_test_ids: HashSet<String> =
            self.tests.rpmverifyfile_tests.iter().map(|t| t.id.clone()).collect();

        for test in other.tests.rpmverifyfile_tests {
            if !existing_verify_test_ids.contains(&test.id) {
                debug!("添加新的 rpmverifyfile_test: {}", test.id);
                existing_verify_test_ids.insert(test.id.clone());
                self.tests.rpmverifyfile_tests.push(test);
            } else {
                debug!("跳过重复的 rpmverifyfile_test: {}", test.id);
            }
        }

        // 3. 合并 objects（去重）
        let mut existing_obj_ids: HashSet<String> =
            self.objects.rpm_info_objects.iter().map(|o| o.id.clone()).collect();

        for obj in other.objects.rpm_info_objects {
            if !existing_obj_ids.contains(&obj.id) {
                debug!("添加新的 rpminfo_object: {}", obj.id);
                existing_obj_ids.insert(obj.id.clone());
                self.objects.rpm_info_objects.push(obj);
            } else {
                debug!("跳过重复的 rpminfo_object: {}", obj.id);
            }
        }

        // 合并 rpmverifyfile_objects
        let mut existing_verify_obj_ids: HashSet<String> =
            self.objects.rpmverifyfile_objects.iter().map(|o| o.id.clone()).collect();

        for obj in other.objects.rpmverifyfile_objects {
            if !existing_verify_obj_ids.contains(&obj.id) {
                debug!("添加新的 rpmverifyfile_object: {}", obj.id);
                existing_verify_obj_ids.insert(obj.id.clone());
                self.objects.rpmverifyfile_objects.push(obj);
            } else {
                debug!("跳过重复的 rpmverifyfile_object: {}", obj.id);
            }
        }

        // 4. 合并 states（去重）
        if let Some(other_states) = other.states.rpminfo_states {
            let mut existing_state_ids: HashSet<String> = if let Some(ref states) = self.states.rpminfo_states {
                states.iter().map(|s| s.id.clone()).collect()
            } else {
                HashSet::new()
            };

            let mut merged_states = self.states.rpminfo_states.take().unwrap_or_default();

            for state in other_states {
                if !existing_state_ids.contains(&state.id) {
                    debug!("添加新的 rpminfo_state: {}", state.id);
                    existing_state_ids.insert(state.id.clone());
                    merged_states.push(state);
                } else {
                    debug!("跳过重复的 rpminfo_state: {}", state.id);
                }
            }

            self.states.rpminfo_states = Some(merged_states);
        }

        // 合并 rpmverifyfile_states
        if let Some(other_verify_states) = other.states.rpmverifyfile_states {
            let mut existing_verify_state_ids: HashSet<String> = if let Some(ref states) = self.states.rpmverifyfile_states {
                states.iter().map(|s| s.id.clone()).collect()
            } else {
                HashSet::new()
            };

            let mut merged_verify_states = self.states.rpmverifyfile_states.take().unwrap_or_default();

            for state in other_verify_states {
                if !existing_verify_state_ids.contains(&state.id) {
                    debug!("添加新的 rpmverifyfile_state: {}", state.id);
                    existing_verify_state_ids.insert(state.id.clone());
                    merged_verify_states.push(state);
                } else {
                    debug!("跳过重复的 rpmverifyfile_state: {}", state.id);
                }
            }

            self.states.rpmverifyfile_states = Some(merged_verify_states);
        }

        // 5. 更新时间戳为最新
        self.generator.time_stamp = other.generator.time_stamp;

        info!("合并完成，最终 definitions: {}, tests: {}, objects: {}, states: {}",
              self.definitions.items.len(),
              self.tests.rpminfo_tests.len(),
              self.objects.rpm_info_objects.len(),
              self.states.rpminfo_states.as_ref().map(|s| s.len()).unwrap_or(0));

        Ok(())
    }

    /// 批量合并多个 OvalDefinitions 到一个新实例
    ///
    /// # 参数
    ///
    /// * `oval_list` - OvalDefinitions 实例的向量
    ///
    /// # 返回值
    ///
    /// 返回 Result<OvalDefinitions>，成功时包含合并后的实例
    pub fn merge_multiple(oval_list: Vec<OvalDefinitions>) -> Result<OvalDefinitions> {
        info!("开始批量合并 {} 个 OvalDefinitions", oval_list.len());

        if oval_list.is_empty() {
            info!("输入列表为空，返回空的 OvalDefinitions");
            return Ok(OvalDefinitions::new());
        }

        let mut iter = oval_list.into_iter();
        let mut merged = iter.next().unwrap();

        for oval in iter {
            merged.merge(oval)?;
        }

        // 更新时间戳为当前时间
        let now = chrono::Utc::now();
        merged.generator.time_stamp = now.to_rfc3339();

        info!("批量合并完成，最终包含 {} 个 definitions", merged.definitions.items.len());
        Ok(merged)
    }
}

/// 生成器信息结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generator {
    /// 产品名称
    #[serde(rename = "oval:product_name")]
    pub product_name: String,

    /// Schema版本
    #[serde(rename = "oval:schema_version")]
    pub schema_version: String,

    /// 时间戳（xs:dateTime格式，例如：2024-01-01T12:00:00Z）
    #[serde(rename = "oval:timestamp")]
    pub time_stamp: String,
    // 内容版本
    // #[serde(rename = "oval:content_version")]
    // pub content_version: u64,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    /// 创建新的Generator实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Generator实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Generator实例");
        Self {
            product_name: DEF_PRODUCT_NAME.into(),
            schema_version: DEF_SCHEMA_VERSION.into(),
            time_stamp: "".to_string(),
            // content_version: 0,
        }
    }
}

/// 定义列表结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Definitions {
    /// 定义项列表
    #[serde(rename = "definition")]
    pub items: Vec<Definition>,
}

impl Default for Definitions {
    fn default() -> Self {
        Self::new()
    }
}

impl Definitions {
    /// 创建新的Definitions实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Definitions实例，包含空的定义列表
    pub fn new() -> Self {
        debug!("创建新的Definitions实例");
        Self { items: Vec::new() }
    }

    /// 添加定义
    pub fn add(&mut self, definition: Definition) {
        self.items.push(definition);
    }

    /// 获取定义数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 根据ID查找定义
    pub fn find_by_id(&self, id: &str) -> Option<&Definition> {
        self.items.iter().find(|d| d.id == id)
    }
}

/// 定义结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Definition {
    /// 定义类别
    #[serde(rename = "@class")]
    pub class: String,

    /// 定义ID
    #[serde(rename = "@id")]
    pub id: String,

    /// 定义版本
    #[serde(rename = "@version")]
    pub version: u32,
    /// 元数据
    pub metadata: Metadata,
    /// 检查条件
    pub criteria: Criteria,
}

impl Default for Definition {
    fn default() -> Self {
        Self::new()
    }
}

impl Definition {
    /// 创建新的Definition实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Definition实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Definition实例");
        Self {
            class: "patch".to_string(),
            id: "".to_string(),
            version: 0,
            metadata: Metadata::new(),
            criteria: Criteria::new(),
        }
    }

    /// 设置ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// 设置类别
    pub fn with_class(mut self, class: String) -> Self {
        self.class = class;
        self
    }

    /// 设置版本
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// 设置条件
    pub fn with_criteria(mut self, criteria: Criteria) -> Self {
        self.criteria = criteria;
        self
    }

    /// 获取ID
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// 获取标题
    pub fn get_title(&self) -> &str {
        &self.metadata.title
    }
}

/// 元数据结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// 标题
    pub title: String,

    /// 影响范围
    #[serde(rename = "affected")]
    pub affected: Affected,

    /// 引用列表（可选）
    #[serde(rename = "reference", default)]
    pub references: Option<Vec<Reference>>,

    /// 描述信息
    pub description: String,

    /// 建议信息
    #[serde(rename = "advisory")]
    pub advisory: Advisory,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    /// 创建新的Metadata实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Metadata实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Metadata实例");
        Self {
            title: "".to_string(),
            affected: Affected::new(),
            references: None,
            description: "".to_string(),
            advisory: Advisory::new(),
        }
    }

    /// 添加引用
    pub fn add_reference(&mut self, reference: Reference) {
        if let Some(ref mut refs) = self.references {
            refs.push(reference);
        } else {
            self.references = Some(vec![reference]);
        }
    }

    /// 获取引用数量
    pub fn get_reference_count(&self) -> usize {
        self.references.as_ref().map(|r| r.len()).unwrap_or(0)
    }
}

/// 引用信息结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reference {
    /// 引用ID
    #[serde(rename = "@ref_id")]
    pub ref_id: String,

    /// 引用URL
    #[serde(rename = "@ref_url")]
    pub ref_url: String,

    /// 来源
    #[serde(rename = "@source")]
    pub source: String,
}

impl Default for Reference {
    fn default() -> Self {
        Self::new()
    }
}

impl Reference {
    /// 创建新的Reference实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Reference实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Reference实例");
        Self {
            ref_id: "".to_string(),
            ref_url: "".to_string(),
            source: "".to_string(),
        }
    }
}

/// 影响范围结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Affected {
    /// 系列
    #[serde(rename = "@family")]
    pub family: String,

    /// 平台信息
    #[serde(rename = "platform")]
    pub platform: String,
}

impl Default for Affected {
    fn default() -> Self {
        Self::new()
    }
}

impl Affected {
    /// 创建新的Affected实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Affected实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Affected实例");
        Self {
            family: "unix".to_string(),
            platform: "".to_string(),
        }
    }
}

/// 建议信息结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Advisory {
    /// 来源
    #[serde(rename = "@from")]
    pub from: String,

    /// 严重性
    #[serde(rename = "severity")]
    pub severity: String,

    /// 版权信息
    #[serde(rename = "rights")]
    pub rights: String,

    /// 发布日期
    #[serde(rename = "issued")]
    pub issued: Issued,

    /// 更新日期
    #[serde(rename = "updated")]
    pub updated: Updated,

    /// CVE列表
    #[serde(rename = "cve", default)]
    pub cve: Vec<CVE>,
    /*
    #[serde(rename = "bugzilla", default)]
    pub bugzilla: Vec<Bugzilla>,

    #[serde(rename = "affected_cpe_list", default)]
    pub affected_cpe_list: Vec<CPE>,
    */
}

impl Default for Advisory {
    fn default() -> Self {
        Self::new()
    }
}

impl Advisory {
    /// 创建新的Advisory实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Advisory实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Advisory实例");
        Self {
            from: ADVISORY_FROM.to_string(),
            rights: CU_LINUX_COPY_RIGHT.to_string(),
            severity: "".to_string(),
            issued: Issued::new(),
            updated: Updated::new(),
            cve: Vec::new(),
        }
    }

    /// 添加CVE
    pub fn add_cve(&mut self, cve: CVE) {
        debug!("添加CVE: {}", cve.content);
        self.cve.push(cve);
    }

    /// 获取CVE数量
    pub fn get_cve_count(&self) -> usize {
        self.cve.len()
    }

    /// 获取所有CVE ID
    pub fn get_cve_ids(&self) -> Vec<&str> {
        self.cve.iter().map(|c| c.content.as_str()).collect()
    }

    /// 检查是否包含指定CVE
    pub fn contains_cve(&self, cve_id: &str) -> bool {
        self.cve.iter().any(|c| c.content == cve_id)
    }
}

/// 发布日期结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Issued {
    /// 日期
    #[serde(rename = "@date")]
    pub date: String,
}

impl Default for Issued {
    fn default() -> Self {
        Self::new()
    }
}

impl Issued {
    /// 创建新的Issued实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Issued实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Issued实例");
        Self {
            date: "".to_string(),
        }
    }
}

/// 更新日期结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Updated {
    /// 日期
    #[serde(rename = "@date")]
    pub date: String,
}

impl Default for Updated {
    fn default() -> Self {
        Self::new()
    }
}

impl Updated {
    /// 创建新的Updated实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Updated实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Updated实例");
        Self {
            date: "".to_string(),
        }
    }
}

/// CVE信息结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CVE {
    /// CVSS3评分
    #[serde(rename = "@cvss3")]
    pub cvss3: String,

    /// 链接
    #[serde(rename = "@href")]
    pub href: String,

    /// 影响程度
    #[serde(rename = "@impact")]
    pub impact: String,

    /// 内容
    #[serde(rename = "#text")]
    pub content: String,
}

impl Default for CVE {
    fn default() -> Self {
        Self::new()
    }
}

impl CVE {
    /// 创建新的CVE实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的CVE实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的CVE实例");
        Self {
            cvss3: "".to_string(),
            href: "".to_string(),
            impact: "".to_string(),
            content: "".to_string(),
        }
    }

    /// 使用构建器模式设置属性
    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn with_cvss3(mut self, cvss3: String) -> Self {
        self.cvss3 = cvss3;
        self
    }

    pub fn with_href(mut self, href: String) -> Self {
        self.href = href;
        self
    }

    pub fn with_impact(mut self, impact: String) -> Self {
        self.impact = impact;
        self
    }

    /// 获取CVE ID
    pub fn get_id(&self) -> &str {
        &self.content
    }
}

/// 严重性级别枚举，按严重程度从低到高排序
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityLevel {
    None,
    Low,
    Medium,
    Moderate,
    Important,
    High,
    Critical,
}

impl SeverityLevel {
    /// 从字符串解析严重性级别（注意：不是 std::str::FromStr trait 的实现）
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => SeverityLevel::Critical,
            "high" => SeverityLevel::High,
            "important" => SeverityLevel::Important,
            "moderate" => SeverityLevel::Moderate,
            "medium" => SeverityLevel::Medium,
            "low" => SeverityLevel::Low,
            "none" => SeverityLevel::None,
            _ => SeverityLevel::None,
        }
    }

    /// 转换为字符串表示
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            SeverityLevel::Critical => "Critical".to_string(),
            SeverityLevel::High => "High".to_string(),
            SeverityLevel::Important => "Important".to_string(),
            SeverityLevel::Moderate => "Moderate".to_string(),
            SeverityLevel::Medium => "Medium".to_string(),
            SeverityLevel::Low => "Low".to_string(),
            SeverityLevel::None => "None".to_string(),
        }
    }
}

/// 从多个 CVE 中计算最高严重性级别
///
/// # 参数
///
/// * `cves` - CVE列表
///
/// # 返回值
///
/// 返回最高严重性级别的字符串表示
pub fn calculate_max_severity(cves: &[CVE]) -> String {
    if cves.is_empty() {
        debug!("CVE列表为空，返回默认严重性级别");
        return "".to_string();
    }

    let max_level = cves
        .iter()
        .map(|cve| SeverityLevel::from_str(&cve.impact))
        .max()
        .unwrap_or(SeverityLevel::None);

    let severity_str = max_level.to_string();
    info!("从 {} 个CVE中计算得到最高严重性级别: {}", cves.len(), severity_str);
    severity_str
}

/*
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bugzilla {
    #[serde(rename = "@href")]
    pub href: String,

    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "#text")]
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "cpe")]
pub struct CPE {
    #[serde(rename = "#text")]
    pub content: String,
}
*/

/// 检查条件结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Criteria {
    /// 操作符
    #[serde(rename = "@operator")]
    pub operator: String,

    /// 条件列表
    pub criterion: Vec<Criterion>,

    /// 子条件列表（可选）
    #[serde(rename = "criteria", default)]
    pub sub_criteria: Option<Vec<Criteria>>,
}

impl Default for Criteria {
    fn default() -> Self {
        Self::new()
    }
}

impl Criteria {
    /// 创建新的Criteria实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Criteria实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Criteria实例");
        Self {
            operator: "".to_string(),
            criterion: Vec::new(),
            sub_criteria: None,
        }
    }

    /// 添加条件
    pub fn add_criterion(&mut self, criterion: Criterion) {
        self.criterion.push(criterion);
    }

    /// 添加子条件
    pub fn add_sub_criteria(&mut self, criteria: Criteria) {
        if let Some(ref mut subs) = self.sub_criteria {
            subs.push(criteria);
        } else {
            self.sub_criteria = Some(vec![criteria]);
        }
    }

    /// 获取条件数量
    pub fn get_criterion_count(&self) -> usize {
        self.criterion.len()
    }

    /// 获取子条件数量
    pub fn get_sub_criteria_count(&self) -> usize {
        self.sub_criteria.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    /// 设置操作符
    pub fn with_operator(mut self, operator: String) -> Self {
        self.operator = operator;
        self
    }
}

/// 条件结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Criterion {
    /// 注释
    #[serde(rename = "@comment")]
    pub comment: String,

    /// 测试引用
    #[serde(rename = "@test_ref")]
    pub test_ref: String,
}

impl Default for Criterion {
    fn default() -> Self {
        Self::new()
    }
}

impl Criterion {
    /// 创建新的Criterion实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Criterion实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Criterion实例");
        Self {
            comment: "".to_string(),
            test_ref: "".to_string(),
        }
    }
}

/// 测试列表结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tests {
    /// RPM信息测试列表
    #[serde(rename = "red-def:rpminfo_test", default)]
    pub rpminfo_tests: Vec<RpmInfoTest>,
    /// RPM验证文件测试列表
    #[serde(rename = "red-def:rpmverifyfile_test", default)]
    pub rpmverifyfile_tests: Vec<RpmVerifyFileTest>,
}

impl Default for Tests {
    fn default() -> Self {
        Self::new()
    }
}

impl Tests {
    /// 创建新的Tests实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Tests实例，包含空的测试列表
    pub fn new() -> Self {
        debug!("创建新的Tests实例");
        Self {
            rpminfo_tests: Vec::new(),
            rpmverifyfile_tests: Vec::new(),
        }
    }

    /// 添加RPM信息测试
    pub fn add_rpminfo_test(&mut self, test: RpmInfoTest) {
        self.rpminfo_tests.push(test);
    }

    /// 获取测试数量
    pub fn len(&self) -> usize {
        self.rpminfo_tests.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.rpminfo_tests.is_empty()
    }

    /// 根据ID查找测试
    pub fn find_by_id(&self, id: &str) -> Option<&RpmInfoTest> {
        self.rpminfo_tests.iter().find(|t| t.id == id)
    }
}

/// RPM信息测试结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpmInfoTest {
    /// 检查方式
    #[serde(rename = "@check")]
    pub check: String,

    /// 注释
    #[serde(rename = "@comment")]
    pub comment: String,

    /// ID
    #[serde(rename = "@id")]
    pub id: String,

    /// 版本
    #[serde(rename = "@version")]
    pub version: u32,

    /// 对象引用
    #[serde(rename = "red-def:object")]
    pub object: ObjectReference,

    /// 状态引用
    #[serde(rename = "red-def:state")]
    pub state: StateReference,
}

impl Default for RpmInfoTest {
    fn default() -> Self {
        Self::new()
    }
}

impl RpmInfoTest {
    /// 创建新的RpmInfoTest实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的RpmInfoTest实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的RpmInfoTest实例");
        Self {
            check: "".to_string(),
            comment: "".to_string(),
            id: "".to_string(),
            version: 0,
            object: ObjectReference::new(),
            state: StateReference::new(),
        }
    }

    /// 使用构建器模式设置属性
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_check(mut self, check: String) -> Self {
        self.check = check;
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = comment;
        self
    }

    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    pub fn with_object_ref(mut self, object_ref: String) -> Self {
        self.object.object_ref = object_ref;
        self
    }

    pub fn with_state_ref(mut self, state_ref: String) -> Self {
        self.state.state_ref = state_ref;
        self
    }
}

/// RPM验证文件测试结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpmVerifyFileTest {
    /// 检查方式
    #[serde(rename = "@check")]
    pub check: String,

    /// 注释
    #[serde(rename = "@comment")]
    pub comment: String,

    /// ID
    #[serde(rename = "@id")]
    pub id: String,

    /// 版本
    #[serde(rename = "@version")]
    pub version: u32,

    /// 对象引用
    #[serde(rename = "red-def:object")]
    pub object: ObjectReference,

    /// 状态引用
    #[serde(rename = "red-def:state")]
    pub state: StateReference,
}

impl Default for RpmVerifyFileTest {
    fn default() -> Self {
        Self::new()
    }
}

impl RpmVerifyFileTest {
    /// 创建新的RpmVerifyFileTest实例
    pub fn new() -> Self {
        Self {
            check: "".to_string(),
            comment: "".to_string(),
            id: "".to_string(),
            version: 0,
            object: ObjectReference::new(),
            state: StateReference::new(),
        }
    }
}

/// 对象引用结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectReference {
    /// 对象引用ID
    #[serde(rename = "@object_ref")]
    pub object_ref: String,
}

impl Default for ObjectReference {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectReference {
    /// 创建新的ObjectReference实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的ObjectReference实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的ObjectReference实例");
        Self {
            object_ref: "".to_string(),
        }
    }
}

/// 状态引用结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateReference {
    /// 状态引用ID
    #[serde(rename = "@state_ref")]
    pub state_ref: String,
}

impl Default for StateReference {
    fn default() -> Self {
        Self::new()
    }
}

impl StateReference {
    /// 创建新的StateReference实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的StateReference实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的StateReference实例");
        Self {
            state_ref: "".to_string(),
        }
    }
}

/// 对象列表结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Objects {
    /// RPM信息对象列表
    #[serde(rename = "red-def:rpminfo_object", default)]
    pub rpm_info_objects: Vec<RpmInfoObject>,
    #[serde(rename = "red-def:rpmverifyfile_object", default)]
    pub rpmverifyfile_objects: Vec<RpmVerifyFileObject>,
}

impl Default for Objects {
    fn default() -> Self {
        Self::new()
    }
}

impl Objects {
    // 创建一个空的 Objects 实例
    /// 创建新的Objects实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Objects实例，包含空的对象列表
    pub fn new() -> Self {
        debug!("创建新的Objects实例");
        Objects {
            rpm_info_objects: Vec::new(),
            rpmverifyfile_objects: Vec::new(),
        }
    }

    // 清空所有对象
    /// 清空所有对象
    pub fn clear(&mut self) {
        debug!("清空所有对象");
        self.rpm_info_objects.clear();
    }

    // 检查是否为空
    /// 检查对象列表是否为空
    ///
    /// # 返回值
    ///
    /// 如果对象列表为空返回true，否则返回false
    pub fn is_empty(&self) -> bool {
        self.rpm_info_objects.is_empty()
    }

    // 获取总对象数
    /// 获取对象总数
    ///
    /// # 返回值
    ///
    /// 返回对象列表中的对象数量
    pub fn len(&self) -> usize {
        self.rpm_info_objects.len()
    }

    // 是否包含 rpm_info 对象
    /// 检查是否包含RPM信息对象
    ///
    /// # 返回值
    ///
    /// 如果包含RPM信息对象返回true，否则返回false
    pub fn has_rpm_info_objects(&self) -> bool {
        !self.rpm_info_objects.is_empty()
    }

    // 添加一个 RpmInfoObject
    /// 添加一个RPM信息对象
    ///
    /// # 参数
    ///
    /// * `obj` - 要添加的RpmInfoObject实例
    pub fn add_rpm_info(&mut self, obj: RpmInfoObject) {
        debug!("添加RPM信息对象: {}", obj.rpm_name);
        self.rpm_info_objects.push(obj);
    }

    // 获取 rpm_info_objects 的只读迭代器
    /// 获取RPM信息对象的只读迭代器
    ///
    /// # 返回值
    ///
    /// 返回RPM信息对象列表的只读迭代器
    pub fn iter_rpm_info(&self) -> impl Iterator<Item = &RpmInfoObject> {
        self.rpm_info_objects.iter()
    }

    // 获取 rpm_info_objects 的可变迭代器
    /// 获取RPM信息对象的可变迭代器
    ///
    /// # 返回值
    ///
    /// 返回RPM信息对象列表的可变迭代器
    pub fn iter_mut_rpm_info(&mut self) -> impl Iterator<Item = &mut RpmInfoObject> {
        self.rpm_info_objects.iter_mut()
    }

    // 获取 rpm_info_objects 的数量
    /// 获取RPM信息对象的数量
    ///
    /// # 返回值
    ///
    /// 返回RPM信息对象列表中的对象数量
    pub fn rpm_info_count(&self) -> usize {
        self.rpm_info_objects.len()
    }
}

/// RPM信息对象结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpmInfoObject {
    /// ID
    #[serde(rename = "@id")]
    pub id: String,

    /// 版本
    #[serde(rename = "@version")]
    pub ver: u64,

    /// RPM名称
    #[serde(rename = "red-def:name")]
    pub rpm_name: String,
}

impl Default for RpmInfoObject {
    fn default() -> Self {
        Self::new()
    }
}

impl RpmInfoObject {
    /// 创建新的RpmInfoObject实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的RpmInfoObject实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的RpmInfoObject实例");
        Self {
            id: "".to_string(),
            ver: 0,
            rpm_name: "".to_string(),
        }
    }

    /// 设置ID
    ///
    /// # 参数
    ///
    /// * `id` - 要设置的ID值
    ///
    /// # 返回值
    ///
    /// 返回修改后的RpmInfoObject实例
    pub fn with_id(mut self, id: String) -> Self {
        debug!("设置RpmInfoObject ID: {}", id);
        self.id = id.to_string();
        self
    }

    /// 设置版本
    ///
    /// # 参数
    ///
    /// * `ver` - 要设置的版本值
    ///
    /// # 返回值
    ///
    /// 返回修改后的RpmInfoObject实例
    pub fn with_ver(mut self, ver: u64) -> Self {
        debug!("设置RpmInfoObject版本: {}", ver);
        self.ver = ver;
        self
    }

    /// 设置RPM名称
    ///
    /// # 参数
    ///
    /// * `rpm_name` - 要设置的RPM名称
    ///
    /// # 返回值
    ///
    /// 返回修改后的RpmInfoObject实例
    pub fn with_rpm_name(mut self, rpm_name: String) -> Self {
        debug!("设置RpmInfoObject RPM名称: {}", rpm_name);
        self.rpm_name = rpm_name;
        self
    }
}

/// RPM验证文件对象结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpmVerifyFileObject {
    /// ID
    #[serde(rename = "@id")]
    pub id: String,

    /// 版本
    #[serde(rename = "@version")]
    pub ver: u64,

    /// 行为配置
    #[serde(rename = "red-def:behaviors")]
    pub behaviors: Behaviors,

    /// 名称
    #[serde(rename = "red-def:name")]
    pub name: Data,

    /// epoch
    #[serde(rename = "red-def:epoch")]
    pub epoch: Data,

    /// 版本
    #[serde(rename = "red-def:version")]
    pub version: Data,

    /// 发布版本
    #[serde(rename = "red-def:release")]
    pub release: Data,

    /// 架构
    #[serde(rename = "red-def:arch")]
    pub arch: Data,

    /// 文件路径
    #[serde(rename = "red-def:filepath")]
    pub filepath: String,
}

impl Default for RpmVerifyFileObject {
    fn default() -> Self {
        Self::new()
    }
}

impl RpmVerifyFileObject {
    /// 创建新的RpmVerifyFileObject实例
    pub fn new() -> Self {
        Self {
            id: "".to_string(),
            ver: 0,
            behaviors: Behaviors::new(),
            name: Data::new(),
            epoch: Data::new(),
            version: Data::new(),
            release: Data::new(),
            arch: Data::new(),
            filepath: "".to_string(),
        }
    }
}

/// 行为配置结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Behaviors {
    /// 不检查配置文件
    #[serde(rename = "@noconfigfiles")]
    pub noconfigfiles: bool,
    /// 不检查ghost文件
    #[serde(rename = "@noghostfiles")]
    pub noghostfiles: bool,
    /// 不检查组
    #[serde(rename = "@nogroup")]
    pub nogroup: bool,
    /// 不检查链接目标
    #[serde(rename = "@nolinkto")]
    pub nolinkto: bool,
    /// 不检查MD5
    #[serde(rename = "@nomd5")]
    pub nomd5: bool,
    /// 不检查模式
    #[serde(rename = "@nomode")]
    pub nomode: bool,
    /// 不检查修改时间
    #[serde(rename = "@nomtime")]
    pub nomtime: bool,
    /// 不检查设备号
    #[serde(rename = "@nordev")]
    pub nordev: bool,
    /// 不检查大小
    #[serde(rename = "@nosize")]
    pub nosize: bool,
    /// 不检查用户
    #[serde(rename = "@nouser")]
    pub nouser: bool,
}

impl Default for Behaviors {
    fn default() -> Self {
        Self::new()
    }
}

impl Behaviors {
    /// 创建新的Behaviors实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Behaviors实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Behaviors实例");
        Self {
            noconfigfiles: true,
            noghostfiles: true,
            nogroup: true,
            nolinkto: true,
            nomd5: true,
            nomode: true,
            nomtime: true,
            nordev: true,
            nosize: true,
            nouser: true,
        }
    }
}

/// 数据结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    /// 操作方式
    #[serde(rename = "@operation")]
    pub operation: String,
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl Data {
    /// 创建新的Data实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Data实例，包含默认值
    pub fn new() -> Self {
        debug!("创建新的Data实例");
        Self {
            operation: "".to_string(),
        }
    }
}

/// 状态列表结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct States {
    /// RPM信息状态列表（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "red-def:rpminfo_state")]
    pub rpminfo_states: Option<Vec<RpmInfoState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "red-def:rpmverifyfile_state")]
    pub rpmverifyfile_states: Option<Vec<RpmVerifyFileState>>,
}

impl Default for States {
    fn default() -> Self {
        todo!()
    }
}

impl States {
    /// 创建新的States实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的States实例，包含默认值
    pub fn new() -> Self {
        todo!()
    }

    /// 添加RPM信息状态
    pub fn add_rpminfo_state(&mut self, state: RpmInfoState) {
        todo!()
    }

    /// 获取状态数量
    pub fn len(&self) -> usize {
        todo!()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// 根据ID查找状态
    pub fn find_by_id(&self, id: &str) -> Option<&RpmInfoState> {
        todo!()
    }
}

/// RPM信息状态结构体
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "rpminfo_states")]
pub struct RpmInfoState {
    /// ID
    #[serde(rename = "@id")]
    pub id: String,

    /// 版本
    #[serde(rename = "@version")]
    pub version: String,

    /// EVR信息（可选）
    #[serde(rename = "red-def:evr")]
    pub evr: Option<Evr>,
    /*
    #[serde(rename = "red-def:signature_keyid")]
    signature_keyid: Option<SignatureKeyId>,
    */
}

impl Default for RpmInfoState {
    fn default() -> Self {
        todo!()
    }
}

impl RpmInfoState {
    /// 创建新的RpmInfoState实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的RpmInfoState实例，包含默认值
    pub fn new() -> Self {
        todo!()
    }

    /// 设置ID
    ///
    /// # 参数
    ///
    /// * `id` - 要设置的ID值
    ///
    /// # 返回值
    ///
    /// 返回修改后的RpmInfoState实例
    pub fn with_id(mut self, id: String) -> Self {
        todo!()
    }

    /// 设置版本
    ///
    /// # 参数
    ///
    /// * `version` - 要设置的版本值
    ///
    /// # 返回值
    ///
    /// 返回修改后的RpmInfoState实例
    pub fn with_version(mut self, version: String) -> Self {
        todo!()
    }

    /// 设置EVR信息
    ///
    /// # 参数
    ///
    /// * `evr` - 要设置的Evr信息（可选）
    ///
    /// # 返回值
    ///
    /// 返回修改后的RpmInfoState实例
    pub fn with_evr(mut self, evr: Option<Evr>) -> Self {
        todo!()
    }
}

// 定义 evr 元素
/// EVR信息结构体
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Evr {
    /// 数据类型
    #[serde(rename = "@datatype")]
    pub datatype: String,

    /// 操作方式
    #[serde(rename = "@operation")]
    pub operation: String,
    /// EVR值
    #[serde(rename = "#text")]
    pub evr: String,
}

impl Default for Evr {
    fn default() -> Self {
        todo!()
    }
}

impl Evr {
    /// 创建新的Evr实例
    ///
    /// # 返回值
    ///
    /// 返回一个新的Evr实例，包含默认值
    pub fn new() -> Self {
        todo!()
    }

    /// 使用构建器模式设置属性
    pub fn with_datatype(mut self, datatype: String) -> Self {
        todo!()
    }

    pub fn with_operation(mut self, operation: String) -> Self {
        todo!()
    }

    pub fn with_evr(mut self, evr: String) -> Self {
        todo!()
    }
}

/// RPM验证文件状态数据结构体
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StateData {
    /// 操作方式
    #[serde(rename = "@operation")]
    pub operation: String,
    /// 内容值
    #[serde(rename = "#text")]
    pub content: String,
}

impl Default for StateData {
    fn default() -> Self {
        todo!()
    }
}

impl StateData {
    /// 创建新的StateData实例
    pub fn new() -> Self {
        todo!()
    }
}

/// RPM验证文件状态结构体
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RpmVerifyFileState {
    /// ID
    #[serde(rename = "@id")]
    pub id: String,

    /// 版本
    #[serde(rename = "@version")]
    pub version: String,

    /// 名称模式匹配
    #[serde(rename = "red-def:name")]
    pub name: StateData,

    /// 版本模式匹配（可选）
    #[serde(rename = "red-def:version", skip_serializing_if = "Option::is_none")]
    pub os_version: Option<StateData>,
}

impl Default for RpmVerifyFileState {
    fn default() -> Self {
        todo!()
    }
}

impl RpmVerifyFileState {
    /// 创建新的RpmVerifyFileState实例
    pub fn new() -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oval_to_string() {
        todo!()
    }

    #[test]
    fn test_oval_definitions_basic_operations() {
        todo!()
    }

    #[test]
    fn test_definition_builder() {
        todo!()
    }

    #[test]
    fn test_definitions_operations() {
        todo!()
    }

    #[test]
    fn test_metadata_references() {
        todo!()
    }

    #[test]
    fn test_advisory_cve_operations() {
        todo!()
    }

    #[test]
    fn test_cve_builder() {
        todo!()
    }

    #[test]
    fn test_criteria_operations() {
        todo!()
    }

    #[test]
    fn test_tests_operations() {
        todo!()
    }

    #[test]
    fn test_rpminfo_test_builder() {
        todo!()
    }

    #[test]
    fn test_objects_operations() {
        todo!()
    }

    #[test]
    fn test_rpminfo_object_builder() {
        todo!()
    }

    #[test]
    fn test_states_operations() {
        todo!()
    }

    #[test]
    fn test_rpminfo_state_builder() {
        todo!()
    }

    #[test]
    fn test_evr_builder() {
        todo!()
    }

    #[test]
    fn test_complete_oval_workflow() {
        todo!()
    }

    #[test]
    fn test_merge_two_ovals() {
        todo!()
    }

    #[test]
    fn test_merge_with_duplicate_ids() {
        todo!()
    }

    #[test]
    fn test_merge_multiple() {
        todo!()
    }

    #[test]
    fn test_merge_empty_oval() {
        todo!()
    }

    #[test]
    fn test_merge_multiple_empty() {
        todo!()
    }
}
