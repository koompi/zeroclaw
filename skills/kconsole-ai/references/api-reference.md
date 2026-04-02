# KConsole AI Gateway API Reference

Base URL: `https://ai.koompi.cloud/v1`
Auth: `Authorization: Bearer {KCONSOLE_AI_KEY}`

## Endpoints

### POST /v1/chat/completions
Standard OpenAI-compatible chat. Supports `stream: true`.

### POST /v1/images/generations
Works for both images and video (Veo). Returns `b64_json`.

### GET /v1/models
List all available models.

## Available Models

**Chat:** gemini-3.1-pro-preview, gemini-3-flash-preview, gemini-2.5-pro, glm-5-turbo, glm-5, glm-4.7-flash, glm-4-plus, glm-4-air

**Image:** gemini-3.1-flash-image-preview, gemini-3-pro-image-preview, imagen-3.0-generate-001

**Video:** veo-3.1-lite-generate-preview

## Notes
- Video generation via Veo is async — gateway blocks and polls until complete (2-5 min)
- Increase HTTP timeout to 300s+ for video requests
- Response format: `{"data": [{"b64_json": "..."}]}`
