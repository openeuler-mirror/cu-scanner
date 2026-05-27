#!/bin/bash
# 代码格式清理脚本
# 功能：去除项目中所有 Rust 源文件的多余空格和空行
# 使用方法：./clean_space.sh

echo "======================================================"
echo "代码格式清理工具"
echo "======================================================"
echo ""

# 使用 Python 脚本进行清理
python3 << 'PYEOF'
import os
import sys

def clean_file(filepath):
    """清理单个文件的多余空格和空行"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"✗ 读取失败: {filepath} - {e}")
        return False
    
    original_content = ''.join(lines)
    
    # 处理每一行
    cleaned_lines = []
    for line in lines:
        # 去除行尾空格和tab
        cleaned_line = line.rstrip() + '\n' if line.strip() else '\n'
        cleaned_lines.append(cleaned_line)
    
    # 压缩连续空行（保留最多2个连续空行）
    result_lines = []
    blank_count = 0
    
    for line in cleaned_lines:
        if line.strip() == '':
            blank_count += 1
            if blank_count <= 2:
                result_lines.append(line)
        else:
            blank_count = 0
            result_lines.append(line)
    
    # 去除文件末尾多余空行（保留最多1个）
    while len(result_lines) > 1 and result_lines[-1].strip() == '' and result_lines[-2].strip() == '':
        result_lines.pop()
    
    # 确保文件以换行符结尾
    if result_lines and not result_lines[-1].endswith('\n'):
        result_lines[-1] += '\n'
    
    new_content = ''.join(result_lines)
    
    # 检查是否有变化
    if new_content != original_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True
    return False

# 查找所有.rs文件
rs_files = []
for root, dirs, files in os.walk('.'):
    # 排除target目录
    if 'target' in root:
        continue
    for file in files:
        if file.endswith('.rs'):
            rs_files.append(os.path.join(root, file))

print(f"找到 {len(rs_files)} 个 Rust 源文件")
print("开始清理...\n")

modified_count = 0
for filepath in sorted(rs_files):
    if clean_file(filepath):
        print(f"✓ 已清理: {filepath}")
        modified_count += 1

print(f"\n" + "="*60)
print(f"处理完成！")
print(f"总文件数: {len(rs_files)}")
print(f"已修改: {modified_count}")
print(f"未修改: {len(rs_files) - modified_count}")
print("="*60)

# 退出码：如果有修改返回0，否则返回1
sys.exit(0 if modified_count > 0 else 1)
PYEOF

exit_code=$?

echo ""
if [ $exit_code -eq 0 ]; then
    echo "✓ 代码格式清理完成"
else
    echo "✓ 所有文件格式已符合规范，无需清理"
fi
