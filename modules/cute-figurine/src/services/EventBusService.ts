import { useCompanionStore } from '../state/companionStore';
import { globalMessageQueue, messageUtils } from '../state/messageQueue';
import { MoodState, ActivityState } from '../types/state.types';
import { taskPersistence } from './TaskPersistenceService';
import type {
  AnimationCommandEvent,
  StateClassificationEvent,
  InterventionRequestEvent,
  RewardEarnedEvent,
  UserInteractionEvent,
  PerformanceEvent,
  CompanionEvent,
} from '../types/events.types';

export type EventHandler<T = any> = (event: T) => void | Promise<void>;

export interface EventSubscription {
  id: string;
  eventType: string;
  handler: EventHandler;
  priority: number;
  once: boolean;
}

export class EventBusService {
  private subscriptions: Map<string, EventSubscription[]> = new Map();
  private eventHistory: CompanionEvent[] = [];
  private maxHistorySize = 100;
  private subscriptionCounter = 0;

  constructor() {
    this.setupDefaultHandlers();
    this.setupPerformanceMonitoring();
  }

  private setupDefaultHandlers(): void {
    // Animation command handling
    this.subscribe('AnimationCommand', this.handleAnimationCommand.bind(this), { priority: 100 });

    // State classification handling
    this.subscribe('StateClassification', this.handleStateClassification.bind(this), {
      priority: 90,
    });

    // Intervention request handling
    this.subscribe('InterventionRequest', this.handleInterventionRequest.bind(this), {
      priority: 80,
    });

    // Reward earned handling
    this.subscribe('RewardEarned', this.handleRewardEarned.bind(this), { priority: 70 });

    // User interaction handling
    this.subscribe('UserInteraction', this.handleUserInteraction.bind(this), { priority: 60 });

    // Performance event handling
    this.subscribe('Performance', this.handlePerformanceEvent.bind(this), { priority: 50 });
  }

  private setupPerformanceMonitoring(): void {
    // Monitor event processing performance
    if (typeof window !== 'undefined') {
      let eventCount = 0;
      let lastReport = Date.now();

      const reportPerformance = () => {
        if (eventCount > 0) {
          const now = Date.now();
          const duration = now - lastReport;
          const eventsPerSecond = (eventCount / duration) * 1000;

          this.emit({
            type: 'Performance',
            source: 'EventBusService',
            timestamp: now,
            payload: {
              metric: 'events_per_second',
              value: eventsPerSecond,
              threshold: 100, // Events per second threshold
            },
          });

          eventCount = 0;
          lastReport = now;
        }
      };

      // Report every 5 seconds
      setInterval(reportPerformance, 5000);

      // Count events
      this.subscribe(
        '*',
        () => {
          eventCount++;
        },
        { priority: 1 }
      );
    }
  }

  public subscribe<T = any>(
    eventType: string,
    handler: EventHandler<T>,
    options: {
      priority?: number;
      once?: boolean;
      id?: string;
    } = {}
  ): string {
    const subscription: EventSubscription = {
      id: options.id || `sub_${++this.subscriptionCounter}`,
      eventType,
      handler: handler as EventHandler,
      priority: options.priority || 0,
      once: options.once || false,
    };

    if (!this.subscriptions.has(eventType)) {
      this.subscriptions.set(eventType, []);
    }

    const subs = this.subscriptions.get(eventType)!;
    subs.push(subscription);

    // Sort by priority (higher priority first)
    subs.sort((a, b) => b.priority - a.priority);

    return subscription.id;
  }

  public unsubscribe(eventType: string, handlerIdOrFunction: string | EventHandler): boolean {
    const subs = this.subscriptions.get(eventType);
    if (!subs) return false;

    const index = subs.findIndex(
      (sub) => sub.id === handlerIdOrFunction || sub.handler === handlerIdOrFunction
    );

    if (index !== -1) {
      subs.splice(index, 1);
      return true;
    }

    return false;
  }

  public emit<T extends CompanionEvent>(event: T): Promise<void> {
    // Add to history
    this.addToHistory(event);

    // Get subscribers for this specific event type
    const specificSubs = this.subscriptions.get(event.type) || [];

    // Get subscribers for wildcard events
    const wildcardSubs = this.subscriptions.get('*') || [];

    // Combine and sort by priority
    const allSubs = [...specificSubs, ...wildcardSubs].sort((a, b) => b.priority - a.priority);

    // Execute handlers
    const promises = allSubs.map(async (sub) => {
      try {
        await sub.handler(event);

        // Remove one-time subscriptions
        if (sub.once) {
          this.unsubscribe(sub.eventType, sub.id);
        }
      } catch (error) {
        console.error(`Error in event handler for ${event.type}:`, error);

        // Emit error event
        this.emit({
          type: 'Error',
          source: 'EventBusService',
          timestamp: Date.now(),
          payload: {
            originalEvent: event,
            error: error instanceof Error ? error.message : String(error),
            handlerId: sub.id,
          },
        });
      }
    });

    return Promise.all(promises).then(() => {});
  }

  private addToHistory(event: CompanionEvent): void {
    this.eventHistory.push(event);

    // Trim history if too large
    if (this.eventHistory.length > this.maxHistorySize) {
      this.eventHistory = this.eventHistory.slice(-this.maxHistorySize);
    }
  }

  // Default event handlers

  private async handleAnimationCommand(event: AnimationCommandEvent): Promise<void> {
    const { command, animation, parameters, message, priority } = event.payload;
    const store = useCompanionStore.getState();

    switch (command) {
      case 'play':
        if (animation) {
          // Emit custom event for animation engine
          window.dispatchEvent(
            new CustomEvent('skellyPlayAnimation', {
              detail: {
                animation,
                parameters: {
                  loop: parameters?.loop,
                  duration: parameters?.duration,
                  transition: parameters?.transition !== false,
                  weight: parameters?.weight,
                  timeScale: parameters?.timeScale,
                },
              },
            })
          );
        }
        break;

      case 'transition':
        if (animation) {
          window.dispatchEvent(
            new CustomEvent('skellyTransitionAnimation', {
              detail: {
                animation,
                duration: parameters?.duration || 0.5,
              },
            })
          );
        }
        break;

      case 'setMood':
        if (parameters?.mood) {
          store.updateMood(parameters.mood);
        }
        break;

      case 'setEnergy':
        if (parameters?.energy !== undefined) {
          store.updateEnergy(parameters.energy);
        }
        break;

      case 'setMeltLevel':
        if (parameters?.meltLevel !== undefined) {
          store.updateMeltLevel(parameters.meltLevel);
        }
        break;
    }

    // Queue message if provided
    if (message) {
      const queuedMessage = messageUtils.createInterventionMessage(
        message.text,
        priority || message.priority || 2
      );

      if (message.style) {
        queuedMessage.style = message.style;
      }

      globalMessageQueue.enqueue(queuedMessage);
    }
  }

  private async handleStateClassification(event: StateClassificationEvent): Promise<void> {
    const { state, confidence, source } = event.payload;
    const store = useCompanionStore.getState();

    // Only update if confidence is high enough
    if (confidence < 0.7) return;

    // Different confidence thresholds for different sources
    const minConfidence = source === 'ai_analysis' ? 0.8 : 0.7;
    if (confidence < minConfidence) return;

    switch (state) {
      case 'focused':
        store.updateMood(MoodState.FOCUSED);
        store.setActivity(ActivityState.WORKING);
        store.updateFocus(Math.min(store.focus + 10, 100));
        break;

      case 'distracted':
        store.updateMood(MoodState.THINKING);
        store.updateEnergy(store.energy - 5);
        store.updateFocus(Math.max(store.focus - 15, 0));
        break;

      case 'tired':
        store.updateMood(MoodState.TIRED);
        store.updateEnergy(Math.max(store.energy - 10, 0));
        store.updateMeltLevel(store.meltLevel + 10);
        break;

      case 'excited':
        store.updateMood(MoodState.EXCITED);
        store.updateEnergy(Math.min(store.energy + 15, 100));
        store.updateHappiness(Math.min(store.happiness + 10, 100));
        break;

      case 'stressed':
        store.updateMood(MoodState.MELTING);
        store.updateMeltLevel(store.meltLevel + 20);
        store.updateEnergy(Math.max(store.energy - 15, 0));
        break;
    }

    // Emit animation command based on new state
    this.emit({
      type: 'AnimationCommand',
      source: 'StateClassification',
      timestamp: Date.now(),
      payload: {
        command: 'play',
        animation: store.getCurrentAnimation(),
        parameters: { transition: true },
      },
    });
  }

  private async handleInterventionRequest(event: InterventionRequestEvent): Promise<void> {
    const { interventionType, message, priority, urgency } = event.payload;
    const store = useCompanionStore.getState();

    if (!store.canIntervene() && urgency !== 'high') return;

    // Create appropriate message
    let interventionMessage;
    switch (interventionType) {
      case 'break_reminder':
        interventionMessage = messageUtils.createInterventionMessage(
          message || 'Time for a quick break! 🌸',
          priority || 3
        );
        break;
      case 'focus_help':
        interventionMessage = messageUtils.createInterventionMessage(
          message || "Let's refocus together! 🎯",
          priority || 3
        );
        break;
      case 'encouragement':
        interventionMessage = messageUtils.createEncouragementMessage(
          message || "You're doing great! Keep it up! ✨"
        );
        break;
      case 'celebration':
        interventionMessage = messageUtils.createCelebrationMessage(message || 'Amazing work! 🎉');
        break;
      case 'check_in':
        interventionMessage = messageUtils.createInterventionMessage(
          message || 'How are you doing? 😊'
        );
        break;
      default:
        interventionMessage = messageUtils.createInterventionMessage(message || 'Hey there! 👋');
    }

    // Queue the message
    if (urgency === 'high') {
      globalMessageQueue.forceDisplay(interventionMessage);
    } else {
      globalMessageQueue.enqueue(interventionMessage);
    }

    // Play appropriate animation
    const animationMap = {
      break_reminder: 'interaction_wave',
      focus_help: 'focused_breathing',
      celebration: 'celebration_dance',
      encouragement: 'happy_bounce',
      check_in: 'interaction_wave',
    };

    const animation = animationMap[interventionType] || 'interaction_wave';

    this.emit({
      type: 'AnimationCommand',
      source: 'InterventionRequest',
      timestamp: Date.now(),
      payload: {
        command: 'play',
        animation,
        parameters: { transition: true, loop: false },
      },
    });

    // Record interaction
    store.recordInteraction();

    // Track intervention in persistence
    const currentSession = taskPersistence.getCurrentSession();
    if (currentSession) {
      taskPersistence.updateCurrentSession({
        interventions: (currentSession.interventions || 0) + 1,
      });
    }
  }

  private async handleRewardEarned(event: RewardEarnedEvent): Promise<void> {
    const { celebrationLevel, message, value, achievement } = event.payload;
    const store = useCompanionStore.getState();

    // Update companion state
    store.updateHappiness(store.happiness + value);
    store.updateEnergy(Math.min(store.energy + value / 2, 100));

    // Show celebration
    store.updateMood(MoodState.CELEBRATING);

    // Emit particles effect
    window.dispatchEvent(
      new CustomEvent('skellyEmitParticles', {
        detail: {
          type: 'confetti',
          count: celebrationLevel === 'large' ? 100 : celebrationLevel === 'medium' ? 50 : 25,
          duration: 3000,
          colors: ['#FFD700', '#FF69B4', '#00CED1', '#98FB98'],
        },
      })
    );

    // Play celebration animation
    this.emit({
      type: 'AnimationCommand',
      source: 'RewardEarned',
      timestamp: Date.now(),
      payload: {
        command: 'play',
        animation: 'celebration_dance',
        parameters: { transition: true, loop: false },
      },
    });

    // Show celebration message
    const celebrationMessage = messageUtils.createCelebrationMessage(
      message || `Great job on ${achievement}! 🎉`
    );
    globalMessageQueue.enqueue(celebrationMessage);

    // Track reward in persistence
    const currentSession = taskPersistence.getCurrentSession();
    if (currentSession) {
      taskPersistence.updateCurrentSession({
        rewards: (currentSession.rewards || 0) + 1,
        productivity: Math.min((currentSession.productivity || 0) + value, 100),
      });
    }
  }

  private async handleUserInteraction(event: UserInteractionEvent): Promise<void> {
    const { interactionType, position } = event.payload;
    const store = useCompanionStore.getState();

    switch (interactionType) {
      case 'pet':
        store.updateHappiness(Math.min(store.happiness + 5, 100));
        store.recordInteraction();

        this.emit({
          type: 'AnimationCommand',
          source: 'UserInteraction',
          timestamp: Date.now(),
          payload: {
            command: 'play',
            animation: 'happy_bounce',
            parameters: { transition: true, loop: false },
          },
        });

        // Random happy response
        const petResponses = [
          'That feels nice! 😊',
          'Thank you for the pets! 💕',
          'I love attention! ✨',
          "You're so kind! 🌟",
        ];
        const randomResponse = petResponses[Math.floor(Math.random() * petResponses.length)];

        globalMessageQueue.enqueue(messageUtils.createEncouragementMessage(randomResponse));
        break;

      case 'click':
        store.recordInteraction();

        this.emit({
          type: 'AnimationCommand',
          source: 'UserInteraction',
          timestamp: Date.now(),
          payload: {
            command: 'play',
            animation: 'interaction_wave',
            parameters: { transition: true, loop: false },
          },
        });
        break;

      case 'hover':
        // Subtle response to hover
        this.emit({
          type: 'AnimationCommand',
          source: 'UserInteraction',
          timestamp: Date.now(),
          payload: {
            command: 'setEnergy',
            parameters: { energy: store.energy + 1 },
          },
        });
        break;

      case 'drag':
        // Update position if provided
        if (position) {
          store.updatePosition(position);
        }
        break;
    }
  }

  private async handlePerformanceEvent(event: PerformanceEvent): Promise<void> {
    const { metric, value, threshold } = event.payload;

    // Auto-adjust quality based on performance
    if (metric === 'frame_time' && threshold && value > threshold) {
      window.dispatchEvent(
        new CustomEvent('skellyAdjustQuality', {
          detail: { action: 'reduce' },
        })
      );
    } else if (metric === 'events_per_second' && threshold && value > threshold) {
      console.warn(`High event processing load: ${value} events/second`);
    }
  }

  // Utility methods

  public getEventHistory(filter?: {
    type?: string;
    source?: string;
    since?: number;
    limit?: number;
  }): CompanionEvent[] {
    let filtered = this.eventHistory;

    if (filter) {
      if (filter.type) {
        filtered = filtered.filter((e) => e.type === filter.type);
      }
      if (filter.source) {
        filtered = filtered.filter((e) => e.source === filter.source);
      }
      if (filter.since) {
        filtered = filtered.filter((e) => e.timestamp >= filter.since!);
      }
      if (filter.limit) {
        filtered = filtered.slice(-filter.limit);
      }
    }

    return filtered;
  }

  public getSubscriptionCount(eventType?: string): number {
    if (eventType) {
      return this.subscriptions.get(eventType)?.length || 0;
    }
    return Array.from(this.subscriptions.values()).reduce((sum, subs) => sum + subs.length, 0);
  }

  public clearHistory(): void {
    this.eventHistory = [];
  }

  public dispose(): void {
    this.subscriptions.clear();
    this.eventHistory = [];
  }
}

// Singleton instance
export const globalEventBus = new EventBusService();

// Convenience functions
export const eventBus = {
  on: globalEventBus.subscribe.bind(globalEventBus),
  off: globalEventBus.unsubscribe.bind(globalEventBus),
  emit: globalEventBus.emit.bind(globalEventBus),
  once: (eventType: string, handler: EventHandler, options: any = {}) =>
    globalEventBus.subscribe(eventType, handler, { ...options, once: true }),
};
