//! 测试批量CSAF到OVAL转换功能
//! 验证共享IdGenerator确保批次内ID不重复
use csaf::CSAF;
use parser::{IdGenerator, batch_csaf_to_oval, csaf_to_oval, csaf_to_oval_with_shared_generator};
use std::collections::HashSet;
use utils::Result;
fn main() -> Result<()> {
    todo!()
}
/// 收集OVAL定义中的所有ID
fn collect_all_ids(oval: &oval::OvalDefinitions) -> HashSet<String> {
    todo!()
}
/// 从ID字符串中提取数字部分
fn extract_id_number(id: &str) -> u64 {
    todo!()
}
