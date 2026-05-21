//! 测试配置使用示例

use test_utils::TestConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从配置文件加载测试配置
    let config = TestConfig::load_from_file("test_config.toml")?;

    println!("=== CSAF测试文件 ===");
    for file in config.get_csaf_files() {
        println!("  {}", file);
    }

    println!("\n=== Parser测试文件 ===");
    for file in config.get_parser_files() {
        println!("  {}", file);
    }

    println!("\n=== 通用测试文件 ===");
    for file in config.get_common_files() {
        println!("  {}", file);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_usage() {
        // 这只是一个示例，展示如何使用测试配置
        let config = TestConfig::load_from_file("test_config.toml").unwrap();

        // 验证CSAF文件配置
        let csaf_files = config.get_csaf_files();
        assert!(!csaf_files.is_empty());

        // 验证Parser文件配置
        let parser_files = config.get_parser_files();
        assert!(!parser_files.is_empty());

        println!("CSAF测试文件数量: {}", csaf_files.len());
        println!("Parser测试文件数量: {}", parser_files.len());
    }
}
