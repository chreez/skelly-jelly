# Gamification Module

Non-intrusive motivational system for ADHD users, providing ambient support without disrupting focus states.

## 🎯 Core Philosophy

The Skelly-Jelly Gamification Module is designed with ADHD users at the center, prioritizing:

- **User Agency**: Never interrupt flow states or critical work
- **Meaningful Recognition**: Celebrate genuine progress, not trivial actions  
- **Adaptive Learning**: Personalize interventions based on user responses
- **Accessibility**: Support diverse needs and preferences
- **Privacy**: All data stays local, no cloud dependencies

## ✨ Features

### 🛡️ Flow State Protection
- Automatically detects and protects deep focus states (confidence > 80%)
- Respects hyperfocus periods under 90 minutes
- Never interrupts critical or high-urgency work contexts
- User-configurable protection settings

### 🎁 Intelligent Reward System
- Variable ratio reinforcement to maintain intrinsic motivation
- Progressive achievements that celebrate meaningful milestones
- Coins and unlockables without creating addiction patterns
- Context-aware reward timing

### 📊 Progress Tracking
- Session metrics with focus time, productivity scores, and flow quality
- Personal record tracking for longest focus, best productivity, deepest flow
- Streak tracking for consistency without pressure
- Trend analysis with actionable insights

### 🦴 Companion Behavior
- Ambient skeleton companion that reacts to your state
- Subtle animations and expressions that enhance without distracting
- Personality system that adapts to user preferences
- Visual celebration of achievements with appropriate intensity

### 💬 Personalized Messaging
- Context-aware message generation
- Adaptive tone based on user mood, time of day, and work type
- Non-patronizing language that respects user intelligence
- Accessibility features for screen readers and cognitive differences

## 🚀 Quick Start

### Installation

```bash
npm install @skelly-jelly/gamification
```

### Basic Usage

```typescript
import { createGamificationModule, DEFAULT_CONFIG } from '@skelly-jelly/gamification';
import { createLogger } from 'winston';

// Create logger and event bus
const logger = createLogger({ /* your config */ });
const eventBus = createEventBus(); // Your event bus instance

// Create gamification module with default settings
const gamification = createGamificationModule(
  eventBus,
  logger,
  DEFAULT_CONFIG,
  {
    id: 'user-id',
    preferences: {
      interventionFrequency: 'moderate',
      respectFlowStates: true,
      companionAnimations: true,
      // ... other preferences
    }
  }
);

// Start the module
await gamification.start();

// The module will now automatically:
// - Listen for state changes from the analysis engine
// - Provide appropriate interventions and rewards
// - Coordinate companion animations
// - Track progress and unlock achievements
```

### Event Integration

The module integrates with the Skelly-Jelly event bus:

**Listens for:**
- `StateChange` - From analysis-engine module
- `InterventionResponse` - User responses to interventions  
- `UserInteraction` - Direct companion interactions

**Publishes:**
- `InterventionRequest` - Requests for AI-generated messages
- `AnimationCommand` - Commands for cute-figurine module
- `RewardEvent` - Achievement and milestone notifications

## 🎮 User Profiles

Choose from pre-configured user profiles or create custom settings:

### Minimal Profile
```typescript
import { USER_PROFILES } from '@skelly-jelly/gamification';

const minimalUser = {
  ...USER_PROFILES.minimal,
  preferences: {
    interventionFrequency: 'minimal',
    animationIntensity: 'minimal', 
    messageStyle: 'minimal',
    celebrationStyle: 'subtle'
  }
};
```

### Engaging Profile
```typescript
const engagingUser = {
  ...USER_PROFILES.engaging,
  preferences: {
    interventionFrequency: 'frequent',
    animationIntensity: 'full',
    messageStyle: 'encouraging', 
    celebrationStyle: 'enthusiastic'
  }
};
```

## 🔧 Configuration

### Intervention Settings
```typescript
const config = {
  intervention: {
    minCooldownMinutes: 15,        // Minimum time between interventions
    adaptiveCooldown: true,        // Learn from user responses
    maxInterventionsPerHour: 3,    // Rate limiting
    respectFlowStates: true,       // Protect focus states
    flowStateThreshold: 0.8        // Confidence threshold for protection
  }
};
```

### Reward Settings
```typescript
const config = {
  rewards: {
    coinsPerFocusMinute: 1,        // Base coin rate
    bonusMultiplier: 1.5,          // Streak and achievement bonus
    variableRatioBase: 0.15,       // Random reward frequency (15%)
    achievementCoins: {
      common: 25,
      rare: 100,
      epic: 250, 
      legendary: 500
    }
  }
};
```

### Companion Personality
```typescript
const config = {
  companion: {
    personalityTraits: {
      cheerfulness: 0.7,           // 0-1, affects animation energy
      humor: 0.5,                  // Playfulness in animations
      formality: 0.2,              // Professional vs. casual
      supportiveness: 0.8          // Emotional support level
    }
  }
};
```

## 🧪 Testing

```bash
# Run all tests
npm test

# Run with coverage
npm run test:coverage

# Run specific test file
npm test InterventionController.test.ts
```

### Key Test Areas

- **Flow State Protection**: Ensures interventions never interrupt deep focus
- **Adaptive Learning**: Verifies cooldown adjustments based on user responses
- **Context Sensitivity**: Tests appropriate intervention timing
- **Accessibility**: Validates screen reader and accessibility compliance
- **Performance**: Monitors response times and resource usage

## 🔍 Examples

### Running the Demo
```bash
npm run example
```

This runs a complete simulation showing:
- State change processing
- Reward distribution
- Achievement unlocking
- Intervention decision making
- Progress tracking
- Companion behavior coordination

### Manual Intervention Testing
```typescript
// Test different intervention types
await gamification.triggerIntervention('encouragement');
await gamification.triggerIntervention('gentle_nudge');
await gamification.triggerIntervention('celebration');
```

### Progress Monitoring
```typescript
const progress = await gamification.getProgress();

console.log('Focus Time:', progress.session.totalFocusTime);
console.log('Achievements:', progress.achievements.length);
console.log('Current Coins:', progress.wallet.coins);
console.log('Productivity Trend:', progress.trends.productivityTrend);
```

## 🎯 Design Principles

### User-Centered Design
- **Respect User Agency**: Users maintain full control over their experience
- **Non-Interruption**: Never break focus or flow states
- **Meaningful Feedback**: Only provide feedback that adds genuine value
- **Accessibility First**: Design for cognitive and physical accessibility needs

### ADHD-Specific Considerations
- **Executive Function Support**: Help with task transitions and focus maintenance
- **Dopamine Regulation**: Provide appropriate stimulation without overstimulation
- **Working Memory Assistance**: Visual cues and progress tracking
- **Rejection Sensitivity**: Gentle, non-judgmental communication

### Privacy and Ethics
- **Local Data**: All personal data stays on the user's device
- **Transparent Algorithms**: Clear explanation of decision-making logic
- **User Ownership**: Users own and control their data
- **No Dark Patterns**: No manipulative or addictive design elements

## 🔧 API Reference

### Core Module Interface
```typescript
interface GamificationModule {
  // Lifecycle
  start(): Promise<void>;
  stop(): Promise<void>;
  isRunning(): boolean;
  
  // Core functionality
  processStateChange(event: StateChangeEvent): Promise<void>;
  getProgress(): Promise<ProgressSummary>;
  updatePreferences(preferences: UserPreferences): Promise<void>;
  
  // Testing and debugging
  triggerIntervention(type: string): Promise<boolean>;
  getMetrics(): Promise<GamificationMetrics>;
}
```

### State Types
```typescript
interface ADHDState {
  type: 'Flow' | 'Hyperfocus' | 'Distracted' | 'Transitioning' | 'Neutral';
  confidence: number;  // 0-1
  depth?: number;      // Flow depth (0-1)
  duration: number;    // Milliseconds in this state
}
```

### User Preferences
```typescript
interface UserPreferences {
  // Intervention preferences
  interventionFrequency: 'minimal' | 'moderate' | 'frequent';
  respectFlowStates: boolean;
  
  // Companion preferences  
  companionAnimations: boolean;
  companionSounds: boolean;
  animationIntensity: 'minimal' | 'moderate' | 'full';
  
  // Message preferences
  messageStyle: 'minimal' | 'informative' | 'encouraging';
  personalizedMessages: boolean;
  
  // Reward preferences
  celebrationStyle: 'subtle' | 'noticeable' | 'enthusiastic';
  rewardTypes: ('coins' | 'achievements' | 'milestones' | 'celebrations')[];
}
```

## 🤝 Contributing

We welcome contributions that improve accessibility, user experience, and ADHD support:

1. **User Research**: Share insights about ADHD user needs
2. **Accessibility**: Improve screen reader support, keyboard navigation
3. **Personalization**: Add new adaptation algorithms
4. **Testing**: Expand test coverage for edge cases
5. **Documentation**: Improve examples and guides

### Development Setup
```bash
git clone https://github.com/skelly-jelly/skelly-jelly.git
cd skelly-jelly/modules/gamification
npm install
npm run dev
```

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- ADHD community feedback and research
- Accessibility guidelines from W3C and ADHD organizations
- Motivation research from self-determination theory
- Game design principles that avoid dark patterns

---

Built with ❤️ for the ADHD community by the Skelly-Jelly team.