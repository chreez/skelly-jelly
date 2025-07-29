# ADR 003: Module Naming Convention

## Status
Accepted (Implemented 2024-01-27)

## Context
The project initially had inconsistent module naming:
- Some modules prefixed with `skelly-jelly-` (storage, data-capture, event-bus)
- Others without prefix (orchestrator, ai-integration)

This inconsistency made it difficult to:
- Identify project modules vs external dependencies
- Maintain consistent package naming
- Search and filter modules

## Decision
All modules must follow the naming pattern: `skelly-jelly-{module-name}`

## Rationale

1. **Namespace Clarity**: Clear distinction between project modules and external dependencies
2. **Discoverability**: Easy to search for all project modules
3. **Consistency**: Uniform naming across Rust and TypeScript modules
4. **Publishing Ready**: If modules are published to crates.io/npm, names are unique
5. **Tooling Benefits**: Build scripts can easily identify project modules

## Implementation

### Renamed Modules
- `orchestrator` → `skelly-jelly-orchestrator`
- `ai-integration` → `skelly-jelly-ai-integration`

### Naming Rules
1. Prefix: Always `skelly-jelly-`
2. Module name: Lowercase with hyphens (kebab-case)
3. Package name in Cargo.toml/package.json must match directory name
4. Full pattern: `skelly-jelly-{descriptive-name}`

### Examples
```
✅ Correct:
- skelly-jelly-event-bus
- skelly-jelly-data-capture
- skelly-jelly-cute-figurine

❌ Incorrect:
- event-bus
- SkellyJellyEventBus
- skelly_jelly_event_bus
```

## Consequences

### Positive
- Immediate recognition of project modules
- Consistent grep/search patterns
- Clear module ownership
- Professional appearance

### Negative
- Longer module names
- More typing for imports
- Breaking change for existing imports

### Migration
1. Rename module directories
2. Update Cargo.toml/package.json package names
3. Update all import statements
4. Update documentation references
5. Update CI/CD configurations

## Enforcement
- CI check to verify module naming
- Developer onboarding documentation
- Code review checklist

## References
- Implementation commit: [Update module naming]
- Updated modules: `modules/skelly-jelly-orchestrator/`, `modules/skelly-jelly-ai-integration/`