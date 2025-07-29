# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Skelly-Jelly project.

## What is an ADR?

An Architecture Decision Record captures an important architectural decision made along with its context and consequences.

## ADR Format

Each ADR follows this structure:
- **Status**: Proposed, Accepted, Deprecated, Superseded
- **Context**: Why we needed to make this decision
- **Decision**: What we decided
- **Rationale**: Why we made this choice
- **Consequences**: What happens as a result (positive and negative)

## Current ADRs

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](001-event-bus-architecture.md) | Event Bus Architecture | Accepted | 2024-01-27 |
| [002](002-hybrid-rust-typescript.md) | Hybrid Rust/TypeScript Architecture | Accepted | 2024-01-27 |
| [003](003-module-naming-convention.md) | Module Naming Convention | Accepted | 2024-01-27 |

## Creating New ADRs

1. Copy the template: `cp template.md XXX-title-of-decision.md`
2. Fill in all sections
3. Submit PR for review
4. Update this README once accepted

## ADR Lifecycle

1. **Proposed**: Under discussion
2. **Accepted**: Decision made and being implemented
3. **Deprecated**: No longer relevant but kept for history
4. **Superseded**: Replaced by a newer ADR

## Why ADRs?

- Document the "why" behind decisions
- Help new team members understand the system
- Prevent revisiting the same discussions
- Track evolution of the architecture
- Enable informed changes to past decisions