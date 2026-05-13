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
        todo!();
    }

    #[test]
    fn test_logger_creation() {
        todo!()
    }
}
