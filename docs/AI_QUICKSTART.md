# StableRWA AI 功能快速开始指南

## 🚀 快速体验 AI 功能

### 前置条件

1. **获取 OpenAI API Key**
   - 访问 [OpenAI Platform](https://platform.openai.com/)
   - 创建账户并获取 API Key
   - 确保账户有足够的使用额度

2. **环境准备**
   ```bash
   # 确保已安装 Rust 1.75+
   rustc --version
   
   # 确保已安装 Docker
   docker --version
   ```

### 🔧 配置 AI 服务

1. **设置环境变量**
   ```bash
   # 复制环境配置文件
   cp .env.example .env
   
   # 编辑配置文件，添加 OpenAI API Key
   echo "OPENAI_API_KEY=sk-your-openai-api-key-here" >> .env
   echo "AI_SERVICE_ENABLED=true" >> .env
   echo "AI_DEFAULT_MODEL=gpt-3.5-turbo" >> .env
   ```

2. **启动 AI 服务**
   ```bash
   # 使用 Docker 启动完整环境
   docker-compose up -d
   
   # 或者单独启动 AI 服务
   cargo run --bin ai-service
   ```

3. **验证 AI 服务**
   ```bash
   # 检查 AI 服务健康状态
   curl http://localhost:8090/health
   
   # 测试基础 AI 功能
   curl -X POST http://localhost:8090/ai-complete \
     -H 'Content-Type: application/json' \
     -d '{"prompt": "Hello, AI!"}'
   ```

## 🤖 AI 功能演示

### 1. 智能资产估值

```bash
# 房地产估值示例
curl -X POST http://localhost:8090/ai-asset-valuation \
  -H 'Content-Type: application/json' \
  -d '{
    "asset_type": "real_estate",
    "location": "上海市浦东新区",
    "area": 100,
    "year_built": 2018,
    "floor": 15,
    "total_floors": 30,
    "amenities": ["地铁", "学校", "医院", "商场"],
    "condition": "excellent"
  }'
```

**预期响应:**
```json
{
  "success": true,
  "data": {
    "estimated_value": 6800000,
    "currency": "CNY",
    "confidence_score": 0.89,
    "valuation_factors": [
      {
        "factor": "location",
        "weight": 0.4,
        "score": 9.2,
        "description": "浦东新区核心地段，交通便利"
      },
      {
        "factor": "building_quality",
        "weight": 0.3,
        "score": 8.5,
        "description": "建筑质量优良，维护状况良好"
      }
    ]
  }
}
```

### 2. 智能风险评估

```bash
# 投资风险评估示例
curl -X POST http://localhost:8090/ai-risk-assessment \
  -H 'Content-Type: application/json' \
  -d '{
    "asset_id": "real-estate-001",
    "investment_amount": 1000000,
    "investment_period": "12个月",
    "investor_profile": {
      "risk_tolerance": "moderate",
      "investment_experience": "intermediate",
      "age": 35
    }
  }'
```

### 3. 智能市场分析

```bash
# 市场趋势分析示例
curl -X POST http://localhost:8090/ai-market-analysis \
  -H 'Content-Type: application/json' \
  -d '{
    "market_type": "real_estate",
    "region": "一线城市",
    "time_horizon": "6个月",
    "analysis_depth": "comprehensive"
  }'
```

### 4. 智能投资建议

```bash
# 个性化投资建议示例
curl -X POST http://localhost:8090/ai-investment-advice \
  -H 'Content-Type: application/json' \
  -d '{
    "user_id": "user-123",
    "available_capital": 5000000,
    "investment_goals": ["capital_appreciation", "income_generation"],
    "time_horizon": "3-5年",
    "risk_preference": "balanced"
  }'
```

## 💻 代码集成示例

### Rust 代码示例

```rust
use stablerwa_framework::core_ai::{AIService, AIRequest};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 AI 服务
    let ai_service = AIService::new();
    
    // 创建资产估值请求
    let valuation_request = AIRequest::new(
        "请估算一套位于北京市朝阳区，面积120平米，2020年建造的公寓价值".to_string()
    )
    .with_model("gpt-4".to_string())
    .with_parameter("temperature".to_string(), serde_json::json!(0.3))
    .with_parameter("max_tokens".to_string(), serde_json::json!(1000));
    
    // 处理请求
    match ai_service.process_request(valuation_request).await {
        Ok(response) => {
            println!("AI 估值结果: {}", response.content);
            if let Some(usage) = response.usage {
                println!("Token 使用量: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            eprintln!("AI 处理错误: {}", e);
        }
    }
    
    Ok(())
}
```

### JavaScript/Node.js 示例

```javascript
const axios = require('axios');

async function getAIAssetValuation(assetData) {
    try {
        const response = await axios.post('http://localhost:8090/ai-asset-valuation', {
            asset_type: assetData.type,
            location: assetData.location,
            area: assetData.area,
            year_built: assetData.yearBuilt,
            amenities: assetData.amenities
        }, {
            headers: {
                'Content-Type': 'application/json'
            }
        });
        
        console.log('AI 估值结果:', response.data);
        return response.data;
    } catch (error) {
        console.error('AI 估值失败:', error.message);
        throw error;
    }
}

// 使用示例
const assetData = {
    type: 'real_estate',
    location: '深圳市南山区',
    area: 90,
    yearBuilt: 2019,
    amenities: ['地铁', '学校', '公园']
};

getAIAssetValuation(assetData);
```

### Python 示例

```python
import requests
import json

class StableRWAAI:
    def __init__(self, base_url="http://localhost:8090"):
        self.base_url = base_url
    
    def asset_valuation(self, asset_data):
        """智能资产估值"""
        url = f"{self.base_url}/ai-asset-valuation"
        response = requests.post(url, json=asset_data)
        
        if response.status_code == 200:
            return response.json()
        else:
            raise Exception(f"AI 估值失败: {response.text}")
    
    def risk_assessment(self, risk_data):
        """智能风险评估"""
        url = f"{self.base_url}/ai-risk-assessment"
        response = requests.post(url, json=risk_data)
        
        if response.status_code == 200:
            return response.json()
        else:
            raise Exception(f"风险评估失败: {response.text}")

# 使用示例
ai_client = StableRWAAI()

# 资产估值
asset_data = {
    "asset_type": "real_estate",
    "location": "广州市天河区",
    "area": 110,
    "year_built": 2021
}

valuation_result = ai_client.asset_valuation(asset_data)
print(f"估值结果: {valuation_result}")
```

## 🔧 自定义 AI 插件开发

### 创建自定义 AI 插件

```rust
use stablerwa_framework::core_ai::{AIPlugin, AIRequest, AIResponse, AIResult};
use async_trait::async_trait;

pub struct CustomMarketAnalysisPlugin {
    name: String,
}

impl CustomMarketAnalysisPlugin {
    pub fn new() -> Self {
        Self {
            name: "custom_market_analysis".to_string(),
        }
    }
}

#[async_trait]
impl AIPlugin for CustomMarketAnalysisPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn process(&self, request: AIRequest) -> AIResult<AIResponse> {
        // 实现自定义市场分析逻辑
        let analysis_result = self.perform_market_analysis(&request.prompt).await?;
        
        Ok(AIResponse::new(
            request.id,
            analysis_result,
            "custom_market_model".to_string(),
        ))
    }
    
    fn can_handle(&self, request: &AIRequest) -> bool {
        request.prompt.contains("市场分析") || 
        request.prompt.contains("market analysis")
    }
}

impl CustomMarketAnalysisPlugin {
    async fn perform_market_analysis(&self, prompt: &str) -> AIResult<String> {
        // 这里实现具体的市场分析逻辑
        // 可以调用外部 API、数据库查询、机器学习模型等
        
        let analysis = format!(
            "基于当前市场数据分析: {}\n\
            - 市场趋势: 稳中有升\n\
            - 风险等级: 中等\n\
            - 建议操作: 谨慎投资",
            prompt
        );
        
        Ok(analysis)
    }
}
```

### 注册和使用自定义插件

```rust
use stablerwa_framework::core_ai::AIService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ai_service = AIService::new();
    
    // 注册自定义插件
    ai_service.add_plugin(Box::new(CustomMarketAnalysisPlugin::new()));
    
    // 使用插件处理请求
    let request = AIRequest::new("请进行房地产市场分析".to_string());
    let response = ai_service.process_request(request).await?;
    
    println!("自定义分析结果: {}", response.content);
    
    Ok(())
}
```

## 📊 AI 性能监控

### 监控 AI 服务状态

```bash
# 检查 AI 服务健康状态
curl http://localhost:8090/health

# 查看 AI 服务指标
curl http://localhost:8090/metrics

# 查看 AI 模型使用统计
curl http://localhost:8090/ai-stats
```

### 性能优化建议

1. **缓存策略**
   - 对相似请求启用智能缓存
   - 设置合适的缓存过期时间

2. **模型选择**
   - 简单任务使用 GPT-3.5-turbo
   - 复杂分析使用 GPT-4
   - 考虑使用本地模型降低成本

3. **批处理**
   - 将多个相关请求合并处理
   - 减少 API 调用次数

## 🛠️ 故障排除

### 常见问题

1. **API Key 无效**
   ```bash
   # 检查 API Key 是否正确设置
   echo $OPENAI_API_KEY
   
   # 测试 API Key 有效性
   curl -H "Authorization: Bearer $OPENAI_API_KEY" \
        https://api.openai.com/v1/models
   ```

2. **服务连接失败**
   ```bash
   # 检查服务是否运行
   docker ps | grep ai-service
   
   # 查看服务日志
   docker logs stablerwa-ai-service
   ```

3. **响应超时**
   - 检查网络连接
   - 增加请求超时时间
   - 考虑使用更快的模型

### 调试模式

```bash
# 启用调试日志
export RUST_LOG=debug
cargo run --bin ai-service

# 查看详细请求日志
tail -f logs/ai-service.log
```

## 📚 更多资源

- **完整 API 文档**: [docs/api/ai-endpoints.md](./api/ai-endpoints.md)
- **AI 架构设计**: [docs/architecture/ai-architecture.md](./architecture/ai-architecture.md)
- **最佳实践指南**: [docs/development/ai-best-practices.md](./development/ai-best-practices.md)
- **示例项目**: [examples/ai-integration/](../examples/ai-integration/)

---

**开始您的 AI 赋能之旅！** 🚀🤖
