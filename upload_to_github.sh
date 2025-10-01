#!/bin/bash

# DD Blind Box 项目自动上传到 GitHub 脚本
# 使用方法: ./upload_to_github.sh

set -e  # 遇到错误时退出

echo "🚀 开始上传 DD Blind Box 项目到 GitHub..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 请在项目根目录运行此脚本"
    exit 1
fi

# 检查 Git 状态
echo "📋 检查 Git 状态..."
git status

# 确认远程仓库设置
echo "🔗 确认远程仓库设置..."
git remote -v

# 推送代码到 GitHub
echo "⬆️  推送代码到 GitHub..."
git push -u origin main

echo "✅ 项目已成功上传到 GitHub!"
echo "🌐 仓库地址: https://github.com/LuckeeDAO/dd_blind_box"

# 显示项目信息
echo ""
echo "📊 项目统计:"
echo "   - 总文件数: $(find . -type f | wc -l)"
echo "   - 代码行数: $(find . -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')"
echo "   - 测试文件: $(find . -name "*test*.rs" | wc -l)"
echo "   - 文档文件: $(find . -name "*.md" | wc -l)"

echo ""
echo "🎉 上传完成! 您现在可以访问 https://github.com/LuckeeDAO/dd_blind_box 查看您的项目"
