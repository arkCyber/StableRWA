# AI Service - StableRWA Platform

## Overview

The AI Service is a microservice component of the StableRWA platform that provides AI-powered text completion capabilities. It integrates with OpenAI-compatible APIs to offer intelligent text generation and analysis features for the Real World Asset (RWA) ecosystem.

## Features

- **Text Completion**: AI-powered text generation and completion
- **Model Information**: Query available AI models and capabilities
- **RESTful API**: HTTP-based microservice architecture
- **Enterprise Security**: Non-root container, secure configuration
- **Health Monitoring**: Built-in health checks and monitoring
- **Comprehensive Logging**: Detailed logging with timestamps
- **Error Handling**: Robust error handling and recovery

## Architecture

```
┌─────────────────┐    HTTP/REST    ┌─────────────────┐
│   Client Apps   │ ──────────────► │   AI Service    │
│                 │                 │   (Port 8090)   │
└─────────────────┘                 └─────────────────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │  OpenAI API     │
                                    │  (External)     │
                                    └─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker (optional)
- OpenAI API key

### Local Development

1. **Clone and setup**:
```bash
cd ai-service
cargo build
```

2. **Set environment variables**:
```bash
export OPENAI_API_KEY="your-openai-api-key"
export OPENAI_API_URL="https://api.openai.com/v1"
export RUST_LOG=info
```

3. **Run the service**:
```bash
cargo run
```

The service will start on `http://localhost:8090`

### Docker Deployment

1. **Build the image**:
```bash
docker build -t ai-service .
```

2. **Run the container**:
```bash
docker run -d \
  --name ai-service \
  -p 8090:8090 \
  -e OPENAI_API_KEY="your-api-key" \
  -e OPENAI_API_URL="https://api.openai.com/v1" \
  -e RUST_LOG=info \
  ai-service
```

## API Documentation

### Base URL
```
http://localhost:8090
```

### Endpoints

#### POST /ai/complete
AI text completion endpoint.

**Request Body**:
```json
{
  "prompt": "Complete this sentence: The future of finance is",
  "max_tokens": 50,
  "temperature": 0.7,
  "model": "gpt-3.5-turbo"
}
```

**Response**:
```json
{
  "choices": [
    {
      "text": " decentralized and accessible to everyone.",
      "index": 0,
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 8,
    "total_tokens": 18
  }
}
```

#### GET /ai/model
Get AI model information.

**Response**:
```json
{
  "model": "gpt-3.5-turbo",
  "version": "1.0.0",
  "capabilities": ["text-completion", "chat"]
}
```

### Error Responses

**400 Bad Request**:
```json
{
  "error": "Invalid request parameters"
}
```

**500 Internal Server Error**:
```json
{
  "error": "AI completion error: API rate limit exceeded"
}
```

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `OPENAI_API_KEY` | OpenAI API key | - | Yes |
| `OPENAI_API_URL` | OpenAI API base URL | `https://api.openai.com/v1` | No |
| `RUST_LOG` | Log level | `info` | No |
| `RUST_BACKTRACE` | Enable backtraces | `1` | No |

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test integration
```

### Manual Testing
```bash
# Test completion endpoint
curl -X POST http://localhost:8090/ai/complete \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello AI", "max_tokens": 10}'

# Test model info endpoint
curl http://localhost:8090/ai/model
```

## Docker Configuration

### Multi-stage Build
The Dockerfile uses a multi-stage build process:
1. **Builder stage**: Compiles Rust code with dependencies
2. **Runtime stage**: Minimal Debian image with compiled binary

### Security Features
- Non-root user (`aiuser`)
- Minimal runtime dependencies
- Health checks
- Proper file permissions

### Health Check
```bash
curl -f http://localhost:8090/ai/model || exit 1
```

## Monitoring and Logging

### Log Levels
- `error`: Error conditions
- `warn`: Warning conditions
- `info`: General information
- `debug`: Detailed debugging information

### Log Format
```
[2024-01-15T10:30:45Z] POST /ai/complete called
[2024-01-15T10:30:46Z] AI completion completed successfully
```

### Metrics (Future Enhancement)
- Request count
- Response time
- Error rate
- Token usage

## Development

### Project Structure
```
ai-service/
├── src/
│   ├── lib.rs          # Core AI client logic
│   └── main.rs         # HTTP server and API endpoints
├── tests/
│   └── integration.rs  # Integration tests
├── Cargo.toml          # Dependencies and metadata
├── Dockerfile          # Container configuration
└── README.md           # This file
```

### Adding New Features
1. Add core logic to `src/lib.rs`
2. Add API endpoints to `src/main.rs`
3. Add tests to `tests/integration.rs`
4. Update documentation

## Troubleshooting

### Common Issues

**API Key Error**:
```
Error: OPENAI_API_KEY not set
```
Solution: Set the `OPENAI_API_KEY` environment variable.

**Rate Limit Error**:
```
Error: API rate limit exceeded
```
Solution: Implement rate limiting or upgrade API plan.

**Connection Error**:
```
Error: Failed to connect to OpenAI API
```
Solution: Check network connectivity and API URL.

### Debug Mode
```bash
export RUST_LOG=debug
export RUST_BACKTRACE=1
cargo run
```

## Contributing

1. Follow the project coding standards
2. Add comprehensive tests
3. Update documentation
4. Ensure all tests pass
5. Submit pull request

## License

This project is part of the StableRWA platform and follows the same licensing terms.

## Contact

- **Author**: arkSong (arksong2018@gmail.com)
- **Project**: StableRWA Platform
- **Repository**: [GitHub Repository URL] 