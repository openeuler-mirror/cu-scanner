//! 数据库模块
//!
//! 该模块提供了数据库相关的功能实现。

pub mod config;
pub mod converter;
pub mod db_converter;
pub mod entities;
pub mod id_counter;
pub mod id_generator;
pub mod maintenance;
pub mod query;
pub mod schema;
pub mod storage;

// 重新导出常用的类型，保持向后兼容性
pub use config::{DatabaseConfig, DatabaseError, DatabaseManager};
pub use db_converter::{csaf_to_oval_with_db_counter, csaf_to_oval_with_default_db_counter};
pub use entities::{
    Criteria, Criterion, Cve, OsInfo, OvalDefinition, Reference, RpmInfoObject, RpmInfoState,
    RpmInfoTest,
};
pub use id_counter::PersistentIdCounter;
pub use id_generator::DatabaseIdGenerator;

/// 根据 os_info_id 生成操作系统检查相关的固定 ID
///
/// # 参数
///
/// * `os_info_id` - 操作系统信息 ID
///
/// # 返回值
///
/// 返回元组 (object_id, state_full_id, state_name_only_id, test_must_id, test_is_id)
///
/// # ID 分配策略
///
/// - 每个 OS 占用 10 个 ID 空间
/// - base_id = os_info_id * 10
/// - object: base_id + 0
/// - state_full (name + version): base_id + 1
/// - state_name_only (仅 name): base_id + 2
/// - test (must be installed): base_id + 3
/// - test (is installed): base_id + 4
/// - 支持 os_info_id 范围：1-999（ID 范围：10-9999）
///
/// # 示例
///
/// ```
/// use database::generate_os_check_ids;
/// let (obj_id, ste_full_id, ste_name_id, tst_must_id, tst_is_id) = generate_os_check_ids(1);
/// // obj_id = "oval:cn.chinaunicom.culinux.cusa:obj:10"
/// // ste_full_id = "oval:cn.chinaunicom.culinux.cusa:ste:11"
/// // ste_name_id = "oval:cn.chinaunicom.culinux.cusa:ste:12"
/// // tst_must_id = "oval:cn.chinaunicom.culinux.cusa:tst:13"
/// // tst_is_id = "oval:cn.chinaunicom.culinux.cusa:tst:14"
/// ```
pub fn generate_os_check_ids(os_info_id: i64) -> (String, String, String, String, String) {
    let base_id = os_info_id * 10;

    let os_object_id = format!("{}{}", oval::CU_LINUX_SA_OBJ_PREFIX, base_id);
    let os_state_full_id = format!("{}{}", oval::CU_LINUX_SA_STE_PREFIX, base_id + 1);
    let os_state_name_only_id = format!("{}{}", oval::CU_LINUX_SA_STE_PREFIX, base_id + 2);
    let os_test_must_id = format!("{}{}", oval::CU_LINUX_SA_TST_PREFIX, base_id + 3);
    let os_test_is_id = format!("{}{}", oval::CU_LINUX_SA_TST_PREFIX, base_id + 4);

    (
        os_object_id,
        os_state_full_id,
        os_state_name_only_id,
        os_test_must_id,
        os_test_is_id,
    )
}
