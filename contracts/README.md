# Module Interface Contracts

This directory contains formal contracts defining the message formats exchanged between Skelly-Jelly modules.

## Purpose

These contracts serve as:
1. **Documentation** - Clear specification of inter-module communication
2. **Validation** - Runtime validation of message formats
3. **Testing** - Contract testing between modules
4. **Evolution** - Versioned schemas for backward compatibility

## Contract Files

- `storage-analysis.yaml` - EventBatch messages from Storage to Analysis Engine
- `analysis-gamification.yaml` - StateClassification messages from Analysis to Gamification
- `gamification-ai.yaml` - InterventionRequest messages from Gamification to AI Integration
- `ai-figurine.yaml` - AnimationCommand messages from AI to Cute Figurine

## Contract Structure

Each contract includes:
- **Metadata**: Producer, consumer, message type, version
- **Schema**: JSON Schema format specification
- **Definitions**: Reusable type definitions
- **Validation**: Business rules and constraints

## Usage

### In Code
```rust
// Load and validate against contract
let contract = Contract::from_file("contracts/storage-analysis.yaml")?;
contract.validate(&event_batch)?;
```

### In Tests
```rust
#[test]
fn test_event_batch_contract() {
    let batch = create_test_batch();
    assert!(validate_against_contract("storage-analysis", &batch).is_ok());
}
```

### For Documentation
Contracts serve as the source of truth for module integration documentation.

## Versioning

Contracts follow semantic versioning:
- **Major**: Breaking changes (incompatible)
- **Minor**: New optional fields (backward compatible)
- **Patch**: Documentation updates

## Adding New Contracts

1. Create YAML file following the pattern
2. Include all required fields
3. Add comprehensive validation rules
4. Update this README
5. Implement validation in code