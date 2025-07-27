# Testing Configuration

## Browser Support Strategy

**Chrome/Chromium Only**: This project focuses on Chrome and Chromium browsers for E2E testing to optimize for:

- **Development Speed**: Faster test execution with fewer browser targets
- **Consistency**: Single rendering engine reduces test complexity  
- **Target Audience**: ADHD focus tools primarily used in Chrome-based environments
- **Resource Efficiency**: Reduced CI/testing overhead

### Supported Browsers
- **Chromium** (Playwright default)
- **Google Chrome** (stable channel)

### Testing Commands
```bash
# Run E2E tests (Chrome/Chromium only)
npm run test:visual

# Run unit tests
npm test

# Run tests with coverage
npm run test -- --coverage
```

### Playwright MCP Integration
- SuperClaude integration via `.cursor/mcp.json`
- Automated browser testing with `playwright-mcp` server
- Live interaction testing and debugging capabilities

### Performance Targets
- **Load Time**: <3s initial render
- **Frame Rate**: >30fps for animations  
- **Memory**: <100MB for desktop sessions
- **CPU**: <2% average usage

### Test Coverage Requirements
- **Unit Tests**: ≥80% line coverage
- **Integration Tests**: ≥70% component coverage  
- **E2E Tests**: 100% critical user journeys

---

*This configuration prioritizes development velocity and target platform optimization over comprehensive cross-browser compatibility.*