# ZeroClaw Agent Guidelines

## Media & Attachments (Telegram)

When sending files back to the user on Telegram, you MUST use the following media markers. The system will detect these markers and upload the actual file or fetch the URL.

### Format
`[TYPE:/path/to/file/or/url]`

- **Important**: Use the **absolute path** for files.
- The path `/workspace/` is automatically remapped to the ZeroClaw workspace root.
- **Example**: `[IMAGE:/workspace/downloaded_image.webp]` or `[DOCUMENT:/workspace/report.pdf]`

### Supported Types
- `IMAGE`: `[IMAGE:/workspace/img.png]`
- `DOCUMENT`: `[DOCUMENT:/workspace/log.txt]`
- `VIDEO`: `[VIDEO:/workspace/video.mp4]`
- `AUDIO`: `[AUDIO:/workspace/song.mp3]`
- `VOICE`: `[VOICE:/workspace/audio.ogg]` (Sent as a playable voice message)

## Workspace Conventions

1. **Working Directory**: Always assume your working directory is the workspace root.
2. **File Downloads**: Save downloads to `/workspace/` to ensure the Telegram channel can find them.
3. **No Code Fences**: Do NOT wrap media markers in triple backticks or inline code fences. They must be plain text in your reply.
4. **Silent Tools**: Do not narrate your tool usage progress (e.g., "I am now downloading..."). Just provide the final answer once the tool finishes.

## Output Formatting
- Use **bold** for headers and key terms.
- Use `backticks` for technical terms and filenames.
- Do NOT use `#` or `##` headers as Telegram does not render them correctly (use **Bold** instead).
