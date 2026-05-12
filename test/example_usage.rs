//! 测试配置使用示例

use test_utils::TestConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从配置文件加载测试配置
    let config = TestConfig::load_from_file("test_config.toml")?;

    println!("=== CSAF测试文件 ===");
    for file in config.get_csaf_files() {
        println!("  {}", file);
    }
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_usage() {
        todo!()
    }
}
