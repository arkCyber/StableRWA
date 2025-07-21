# 📸 StableRWA 截图设置指南

## 🚨 重要说明

当前的截图文件是文本文件，需要替换为真正的图片文件才能在 GitHub 上正常显示。

## 📋 需要添加的截图

### 1. 主仪表板截图 (`assets/screenshots/dashboard-main.png`)
**您提供的第一张截图内容：**
- StableRWA Asset SDK Platform 界面
- 左侧导航栏：Dashboard, Assets, AI Services, Users, Transactions, Analytics, Compliance, Reports, Documentation, Settings
- 主要数据显示：
  - Total Assets: 2,847 (+12.5%)
  - Total Value Locked: $45.2M (+8.2%)
  - Active Users: 1,234 (+3.1%)
  - Transaction Volume: $12.8M (-2.4%)
- Asset Performance 图表
- Quick Actions 面板
- 深色主题，橙色高亮

### 2. AI 服务截图 (`assets/screenshots/ai-services.png`)
**您提供的第二张截图内容：**
- AI Services 页面
- 四个主要服务卡片：
  1. AI Asset Valuation (Available, 90% accuracy)
  2. Risk Assessment (Available, 91% accuracy)
  3. Document Analysis (Beta, 88% accuracy)
  4. Market Intelligence (Coming Soon, N/A accuracy)
- 每个服务都有功能标签和操作按钮
- 专业的深色主题界面

## 🔧 设置步骤

### 步骤 1: 保存截图文件
1. 将您提供的第一张截图保存为 `dashboard-main.png`
2. 将您提供的第二张截图保存为 `ai-services.png`
3. 确保图片格式为 PNG，分辨率建议 1920x1080 或更高

### 步骤 2: 放置文件
```bash
# 将截图文件复制到正确位置
cp /path/to/your/dashboard-screenshot.png assets/screenshots/dashboard-main.png
cp /path/to/your/ai-services-screenshot.png assets/screenshots/ai-services.png
```

### 步骤 3: 验证文件
```bash
# 检查文件是否为真正的图片文件
file assets/screenshots/dashboard-main.png
file assets/screenshots/ai-services.png

# 应该显示类似：
# assets/screenshots/dashboard-main.png: PNG image data, 1920 x 1080, 8-bit/color RGBA, non-interlaced
```

### 步骤 4: 提交和推送
```bash
# 添加文件到 Git
git add assets/screenshots/

# 提交更改
git commit -m "feat: add real dashboard and AI services screenshots"

# 推送到 GitHub
git push origin main
```

## 🖼️ 图片要求

### 技术规格
- **格式**: PNG (推荐) 或 JPG
- **分辨率**: 最小 1920x1080，推荐 2560x1440
- **文件大小**: 建议小于 2MB
- **颜色**: 真彩色，支持透明度

### 内容要求
- **清晰度**: 文字清晰可读
- **完整性**: 显示完整的界面，无截断
- **真实性**: 显示真实的数据和功能
- **专业性**: 界面整洁，数据合理

## 🔍 故障排除

### 问题 1: 图片不显示
**原因**: 文件不是真正的图片格式
**解决**: 确保使用真正的 PNG/JPG 图片文件

### 问题 2: GitHub 上显示不正常
**原因**: 文件路径错误或文件损坏
**解决**: 检查文件路径和文件完整性

### 问题 3: 图片太大
**原因**: 文件大小超过 GitHub 限制
**解决**: 压缩图片或降低分辨率

## 📝 自动化脚本

创建一个自动化脚本来处理截图：

```bash
#!/bin/bash
# scripts/setup-screenshots.sh

echo "🖼️ Setting up StableRWA screenshots..."

# 检查截图目录
if [ ! -d "assets/screenshots" ]; then
    mkdir -p assets/screenshots
    echo "✅ Created screenshots directory"
fi

# 检查截图文件
if [ ! -f "assets/screenshots/dashboard-main.png" ]; then
    echo "❌ Missing: assets/screenshots/dashboard-main.png"
    echo "   Please add the dashboard screenshot"
fi

if [ ! -f "assets/screenshots/ai-services.png" ]; then
    echo "❌ Missing: assets/screenshots/ai-services.png"
    echo "   Please add the AI services screenshot"
fi

# 验证文件类型
for file in assets/screenshots/*.png; do
    if [ -f "$file" ]; then
        file_type=$(file "$file" | grep -o "PNG image data")
        if [ -n "$file_type" ]; then
            echo "✅ Valid PNG: $file"
        else
            echo "❌ Invalid file: $file (not a PNG image)"
        fi
    fi
done

echo "🎉 Screenshot setup check complete!"
```

## 🚀 快速修复命令

如果您有截图文件，可以使用以下命令快速设置：

```bash
# 1. 确保在项目根目录
cd /Users/arksong/rwa-platform

# 2. 创建截图目录（如果不存在）
mkdir -p assets/screenshots

# 3. 复制您的截图文件（请替换为实际路径）
# cp /path/to/dashboard-screenshot.png assets/screenshots/dashboard-main.png
# cp /path/to/ai-services-screenshot.png assets/screenshots/ai-services.png

# 4. 验证文件
ls -la assets/screenshots/

# 5. 提交和推送
git add assets/screenshots/
git commit -m "feat: add real platform screenshots"
git push origin main
```

## 📞 需要帮助？

如果您需要帮助设置截图，请提供：
1. 截图文件的当前位置
2. 任何错误信息
3. 您希望如何处理这些文件

我可以帮助您创建正确的命令来设置截图文件。
