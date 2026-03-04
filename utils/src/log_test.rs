#[cfg(test)]
mod tests {
    use crate::log::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_logger_with_file_output() {
        let log_file = "tmp/test_log.txt";

        // 确保tmp目录存在
        let _ = std::fs::create_dir_all("tmp");

        // 初始化文件日志
        let target = LogTarget::File(log_file.to_string());
        // 使用较低的日志级别以便能捕获info日志
        let _ = init_logger_with_level_and_target(log::Level::Info, target);

        // 记录一些日志
        log::info!("Test info message");
        log::warn!("Test warning message");
        log::error!("Test error message");

        // 读取日志文件内容
        let mut file = fs::File::open(log_file).expect("Failed to open log file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read log file");

        // 验证日志内容
        assert!(contents.contains("Test info message"));
        assert!(contents.contains("Test warning message"));
        assert!(contents.contains("Test error message"));

        // 清理测试文件
        let _ = fs::remove_file(log_file);
    }

    #[test]
    fn test_logger_creation() {
        // 测试创建不同类型的日志记录器
        let _stdout_logger = CUScannerLogger::with_target(LogTarget::Stdout);
        assert!(true); // 只要不panic就说明创建成功

        // 确保tmp目录存在
        let _ = std::fs::create_dir_all("tmp");

        let _file_logger =
            CUScannerLogger::with_target(LogTarget::File("tmp/test_creation.txt".to_string()));
        assert!(true); // 只要不panic就说明创建成功

        // 清理测试文件
        let _ = fs::remove_file("tmp/test_creation.txt");
    }
}
