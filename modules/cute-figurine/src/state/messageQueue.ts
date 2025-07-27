import type { Message } from '../types';

export interface QueuedMessage extends Message {
  id: string;
  timestamp: number;
  priority: number;
  processed: boolean;
}

export class MessageQueue {
  private queue: QueuedMessage[] = [];
  private processing = false;
  private currentMessage: QueuedMessage | null = null;
  private onMessageDisplay?: (message: QueuedMessage) => void;
  private onMessageClear?: () => void;

  constructor(onMessageDisplay?: (message: QueuedMessage) => void, onMessageClear?: () => void) {
    this.onMessageDisplay = onMessageDisplay;
    this.onMessageClear = onMessageClear;
  }

  /**
   * Add a message to the queue
   */
  enqueue(message: Message): string {
    const queuedMessage: QueuedMessage = {
      ...message,
      id: message.id || this.generateId(),
      timestamp: Date.now(),
      priority: message.priority || 1,
      processed: false,
      duration: message.duration || 5000,
    };

    // Insert message in priority order (higher priority first)
    let insertIndex = this.queue.length;
    for (let i = 0; i < this.queue.length; i++) {
      if (this.queue[i].priority < queuedMessage.priority) {
        insertIndex = i;
        break;
      }
    }

    this.queue.splice(insertIndex, 0, queuedMessage);

    // Start processing if not already processing
    if (!this.processing) {
      this.processNext();
    }

    return queuedMessage.id;
  }

  /**
   * Remove a message from the queue by ID
   */
  remove(messageId: string): boolean {
    const index = this.queue.findIndex((msg) => msg.id === messageId);
    if (index !== -1) {
      this.queue.splice(index, 1);
      return true;
    }
    return false;
  }

  /**
   * Clear all messages from the queue
   */
  clear(): void {
    this.queue = [];
    if (this.currentMessage) {
      this.clearCurrent();
    }
  }

  /**
   * Clear the current message and process next
   */
  clearCurrent(): void {
    if (this.currentMessage) {
      this.currentMessage = null;
      this.processing = false;
      this.onMessageClear?.();

      // Process next message after a short delay
      setTimeout(() => this.processNext(), 200);
    }
  }

  /**
   * Get the current message being displayed
   */
  getCurrent(): QueuedMessage | null {
    return this.currentMessage;
  }

  /**
   * Get all queued messages
   */
  getQueue(): QueuedMessage[] {
    return [...this.queue];
  }

  /**
   * Get queue statistics
   */
  getStats() {
    return {
      queueLength: this.queue.length,
      isProcessing: this.processing,
      currentMessage: this.currentMessage?.id || null,
      highPriorityCount: this.queue.filter((msg) => msg.priority >= 3).length,
      oldestMessage: this.queue.length > 0 ? this.queue[this.queue.length - 1].timestamp : null,
    };
  }

  /**
   * Process the next message in the queue
   */
  private processNext(): void {
    if (this.processing || this.queue.length === 0) {
      return;
    }

    this.processing = true;
    const nextMessage = this.queue.shift();

    if (!nextMessage) {
      this.processing = false;
      return;
    }

    this.currentMessage = nextMessage;
    this.currentMessage.processed = true;

    // Display the message
    this.onMessageDisplay?.(this.currentMessage);

    // Auto-clear after duration
    setTimeout(() => {
      this.clearCurrent();
    }, this.currentMessage.duration || 5000);
  }

  /**
   * Generate a unique message ID
   */
  private generateId(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Check if a message type should interrupt current message
   */
  private shouldInterrupt(newMessage: QueuedMessage): boolean {
    if (!this.currentMessage) return true;

    // High priority messages can interrupt medium/low priority
    if (newMessage.priority >= 4 && this.currentMessage.priority < 3) {
      return true;
    }

    // Emergency messages always interrupt
    if (newMessage.priority >= 5) {
      return true;
    }

    return false;
  }

  /**
   * Force display of a high-priority message, interrupting current if needed
   */
  forceDisplay(message: Message): string {
    const queuedMessage: QueuedMessage = {
      ...message,
      id: message.id || this.generateId(),
      timestamp: Date.now(),
      priority: Math.max(message.priority || 1, 4), // Ensure high priority
      processed: false,
      duration: message.duration || 5000,
    };

    if (this.shouldInterrupt(queuedMessage)) {
      // Clear current message and display immediately
      if (this.currentMessage) {
        this.clearCurrent();
      }

      this.currentMessage = queuedMessage;
      this.currentMessage.processed = true;
      this.processing = true;

      this.onMessageDisplay?.(this.currentMessage);

      setTimeout(() => {
        this.clearCurrent();
      }, this.currentMessage.duration || 5000);

      return queuedMessage.id;
    } else {
      // Add to queue as normal high priority
      return this.enqueue(queuedMessage);
    }
  }

  /**
   * Update message handlers
   */
  setHandlers(
    onMessageDisplay?: (message: QueuedMessage) => void,
    onMessageClear?: () => void
  ): void {
    this.onMessageDisplay = onMessageDisplay;
    this.onMessageClear = onMessageClear;
  }
}

// Singleton instance for global use
export const globalMessageQueue = new MessageQueue();

// Export utility functions
export const messageUtils = {
  /**
   * Create a standard intervention message
   */
  createInterventionMessage: (text: string, priority: number = 3): Message => ({
    text,
    duration: 7000,
    style: 'intervention',
    priority,
  }),

  /**
   * Create a celebration message
   */
  createCelebrationMessage: (text: string): Message => ({
    text,
    duration: 5000,
    style: 'celebration',
    priority: 2,
  }),

  /**
   * Create an encouragement message
   */
  createEncouragementMessage: (text: string): Message => ({
    text,
    duration: 6000,
    style: 'encouragement',
    priority: 2,
  }),

  /**
   * Create an emergency/urgent message
   */
  createUrgentMessage: (text: string): Message => ({
    text,
    duration: 10000,
    style: 'intervention',
    priority: 5,
  }),
};
