# Changelog

All notable changes to the RipTide CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-01-03

### Added
- Initial release of RipTide CLI
- Core commands: `crawl`, `search`, `health`
- Streaming support with `stream` command
- Session management commands
- Worker management and monitoring
- Spider deep crawling
- Batch processing from files
- Interactive mode
- Configuration management
- Multiple output formats (text, JSON, markdown, NDJSON, CSV)
- Colored terminal output
- Progress spinners and loading indicators
- Global options for API URL and key
- Environment variable support
- Auto-update notifications
- Comprehensive help and examples
- Error handling with retry logic
- Debug mode

### Command List
- `riptide crawl` - Crawl one or more URLs
- `riptide search` - Deep search with content extraction
- `riptide health` - Check API health status
- `riptide stream` - Stream crawl results in real-time
- `riptide session` - Manage crawling sessions
- `riptide worker` - Manage worker queue
- `riptide monitor` - Real-time monitoring
- `riptide spider` - Deep crawl from a URL
- `riptide batch` - Process URLs from file
- `riptide config` - Manage configuration
- `riptide interactive` - Interactive mode
- `riptide examples` - Show usage examples

### Features
- ✅ 11 main commands with subcommands
- ✅ Full RipTide API coverage (59 endpoints)
- ✅ Watch mode for continuous monitoring
- ✅ Streaming support for real-time results
- ✅ Batch processing with concurrency control
- ✅ Session management for stateful crawling
- ✅ Multiple output formats
- ✅ Configuration file support
- ✅ Interactive mode with prompts
- ✅ Colored output with progress indicators
- ✅ Error handling and retry logic
- ✅ Environment variable support
- ✅ Auto-update notifications

### Documentation
- Complete README with examples
- Inline help for all commands
- Example scripts (basic and advanced)
- API reference
- Troubleshooting guide

## [Unreleased]

### Planned
- Proxy support configuration
- Custom headers management
- Output templates
- Shell completions (bash, zsh, fish)
- Integration with other tools (jq, fzf)
- Performance profiling mode
- Export to different formats (YAML, XML)
- Plugin system for extensibility
