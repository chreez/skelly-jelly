/**
 * Collection of easing functions for smooth animations
 * Based on Robert Penner's easing equations
 */

export type EasingFunction = (t: number) => number;

export const easings = {
  // Linear
  linear: (t: number): number => t,

  // Quadratic
  easeInQuad: (t: number): number => t * t,
  easeOutQuad: (t: number): number => t * (2 - t),
  easeInOutQuad: (t: number): number => (t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t),

  // Cubic
  easeInCubic: (t: number): number => t * t * t,
  easeOutCubic: (t: number): number => 1 + --t * t * t,
  easeInOutCubic: (t: number): number =>
    t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1,

  // Quartic
  easeInQuart: (t: number): number => t * t * t * t,
  easeOutQuart: (t: number): number => 1 - --t * t * t * t,
  easeInOutQuart: (t: number): number => (t < 0.5 ? 8 * t * t * t * t : 1 - 8 * --t * t * t * t),

  // Quintic
  easeInQuint: (t: number): number => t * t * t * t * t,
  easeOutQuint: (t: number): number => 1 + --t * t * t * t * t,
  easeInOutQuint: (t: number): number =>
    t < 0.5 ? 16 * t * t * t * t * t : 1 + 16 * --t * t * t * t * t,

  // Sine
  easeInSine: (t: number): number => 1 - Math.cos((t * Math.PI) / 2),
  easeOutSine: (t: number): number => Math.sin((t * Math.PI) / 2),
  easeInOutSine: (t: number): number => -(Math.cos(Math.PI * t) - 1) / 2,

  // Exponential
  easeInExpo: (t: number): number => (t === 0 ? 0 : Math.pow(2, 10 * (t - 1))),
  easeOutExpo: (t: number): number => (t === 1 ? 1 : 1 - Math.pow(2, -10 * t)),
  easeInOutExpo: (t: number): number => {
    if (t === 0) return 0;
    if (t === 1) return 1;
    return t < 0.5 ? Math.pow(2, 20 * t - 10) / 2 : (2 - Math.pow(2, -20 * t + 10)) / 2;
  },

  // Circular
  easeInCirc: (t: number): number => 1 - Math.sqrt(1 - t * t),
  easeOutCirc: (t: number): number => Math.sqrt(1 - --t * t),
  easeInOutCirc: (t: number): number =>
    t < 0.5
      ? (1 - Math.sqrt(1 - 4 * t * t)) / 2
      : (Math.sqrt(1 - (-2 * t + 2) * (-2 * t + 2)) + 1) / 2,

  // Back
  easeInBack: (t: number): number => {
    const c1 = 1.70158;
    const c3 = c1 + 1;
    return c3 * t * t * t - c1 * t * t;
  },
  easeOutBack: (t: number): number => {
    const c1 = 1.70158;
    const c3 = c1 + 1;
    return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
  },
  easeInOutBack: (t: number): number => {
    const c1 = 1.70158;
    const c2 = c1 * 1.525;
    return t < 0.5
      ? (Math.pow(2 * t, 2) * ((c2 + 1) * 2 * t - c2)) / 2
      : (Math.pow(2 * t - 2, 2) * ((c2 + 1) * (t * 2 - 2) + c2) + 2) / 2;
  },

  // Elastic
  easeInElastic: (t: number): number => {
    const c4 = (2 * Math.PI) / 3;
    return t === 0 ? 0 : t === 1 ? 1 : -Math.pow(2, 10 * t - 10) * Math.sin((t * 10 - 10.75) * c4);
  },
  easeOutElastic: (t: number): number => {
    const c4 = (2 * Math.PI) / 3;
    return t === 0 ? 0 : t === 1 ? 1 : Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * c4) + 1;
  },
  easeInOutElastic: (t: number): number => {
    const c5 = (2 * Math.PI) / 4.5;
    return t === 0
      ? 0
      : t === 1
        ? 1
        : t < 0.5
          ? -(Math.pow(2, 20 * t - 10) * Math.sin((20 * t - 11.125) * c5)) / 2
          : (Math.pow(2, -20 * t + 10) * Math.sin((20 * t - 11.125) * c5)) / 2 + 1;
  },

  // Bounce
  easeInBounce: (t: number): number => 1 - easings.easeOutBounce(1 - t),
  easeOutBounce: (t: number): number => {
    const n1 = 7.5625;
    const d1 = 2.75;

    if (t < 1 / d1) {
      return n1 * t * t;
    } else if (t < 2 / d1) {
      return n1 * (t -= 1.5 / d1) * t + 0.75;
    } else if (t < 2.5 / d1) {
      return n1 * (t -= 2.25 / d1) * t + 0.9375;
    } else {
      return n1 * (t -= 2.625 / d1) * t + 0.984375;
    }
  },
  easeInOutBounce: (t: number): number =>
    t < 0.5
      ? (1 - easings.easeOutBounce(1 - 2 * t)) / 2
      : (1 + easings.easeOutBounce(2 * t - 1)) / 2,
};

/**
 * Animation-specific easing presets for common use cases
 */
export const animationEasings = {
  // Natural movement
  natural: easings.easeOutQuart,

  // UI interactions
  uiEnter: easings.easeOutCubic,
  uiExit: easings.easeInCubic,
  uiTransition: easings.easeInOutCubic,

  // Character animations
  bounce: easings.easeOutBounce,
  settle: easings.easeOutBack,
  excited: easings.easeOutElastic,
  calm: easings.easeInOutSine,

  // Mood transitions
  happyToSad: easings.easeInQuad,
  sadToHappy: easings.easeOutQuart,
  tiredToEnergetic: easings.easeOutBack,
  energeticToTired: easings.easeInCubic,

  // State changes
  focusIn: easings.easeInOutQuad,
  focusOut: easings.easeOutQuad,
  meltingIn: easings.easeInCubic,
  meltingOut: easings.easeOutCubic,

  // Interaction feedback
  petResponse: easings.easeOutElastic,
  clickResponse: easings.easeOutBack,
  hoverResponse: easings.easeOutQuad,
};

/**
 * Utility functions for creating custom easing curves
 */
export const easingUtils = {
  /**
   * Creates a custom cubic bezier easing function
   */
  cubicBezier: (x1: number, y1: number, x2: number, y2: number): EasingFunction => {
    return (t: number): number => {
      // Simplified cubic bezier approximation
      const cx = 3 * x1;
      const bx = 3 * (x2 - x1) - cx;
      const ax = 1 - cx - bx;

      const cy = 3 * y1;
      const by = 3 * (y2 - y1) - cy;
      const ay = 1 - cy - by;

      const sampleCurveX = (t: number) => ((ax * t + bx) * t + cx) * t;
      const sampleCurveY = (t: number) => ((ay * t + by) * t + cy) * t;

      // Newton-Raphson iteration to solve for t
      let x = t;
      for (let i = 0; i < 8; i++) {
        const currentX = sampleCurveX(x) - t;
        if (Math.abs(currentX) < 0.001) break;
        const currentSlope = 3 * (ax * x + bx) * x + cx;
        x -= currentX / currentSlope;
      }

      return sampleCurveY(x);
    };
  },

  /**
   * Creates an easing function that combines two easings
   */
  chain: (
    ease1: EasingFunction,
    ease2: EasingFunction,
    splitPoint: number = 0.5
  ): EasingFunction => {
    return (t: number): number => {
      if (t < splitPoint) {
        return ease1(t / splitPoint) * splitPoint;
      } else {
        return splitPoint + ease2((t - splitPoint) / (1 - splitPoint)) * (1 - splitPoint);
      }
    };
  },

  /**
   * Creates an easing function that oscillates
   */
  oscillate: (baseEasing: EasingFunction, frequency: number = 1): EasingFunction => {
    return (t: number): number => {
      const oscillation = Math.sin(t * Math.PI * 2 * frequency) * 0.1;
      return baseEasing(t) + oscillation * (1 - t); // Dampen oscillation over time
    };
  },

  /**
   * Creates an easing function with a customizable overshoot
   */
  overshoot: (baseEasing: EasingFunction, amount: number = 0.1): EasingFunction => {
    return (t: number): number => {
      const base = baseEasing(t);
      if (t > 0.8) {
        const overshootPhase = (t - 0.8) / 0.2;
        const overshootValue = Math.sin(overshootPhase * Math.PI) * amount;
        return base + overshootValue;
      }
      return base;
    };
  },

  /**
   * Creates an easing function that steps
   */
  steps: (steps: number, baseEasing: EasingFunction = easings.linear): EasingFunction => {
    return (t: number): number => {
      const step = Math.floor(t * steps) / steps;
      const nextStep = Math.min((Math.floor(t * steps) + 1) / steps, 1);
      const stepProgress = (t * steps) % 1;
      return step + (nextStep - step) * baseEasing(stepProgress);
    };
  },

  /**
   * Inverts an easing function
   */
  invert: (easing: EasingFunction): EasingFunction => {
    return (t: number): number => 1 - easing(1 - t);
  },

  /**
   * Creates a mirror effect (goes up then down)
   */
  mirror: (easing: EasingFunction): EasingFunction => {
    return (t: number): number => {
      if (t < 0.5) {
        return easing(t * 2);
      } else {
        return easing((1 - t) * 2);
      }
    };
  },
};

/**
 * Gets an appropriate easing function for a mood transition
 */
export function getTransitionEasing(fromMood: string, toMood: string): EasingFunction {
  const key =
    `${fromMood}To${toMood.charAt(0).toUpperCase() + toMood.slice(1)}` as keyof typeof animationEasings;
  return animationEasings[key] || animationEasings.natural;
}

/**
 * Gets an easing function based on character energy level
 */
export function getEnergyBasedEasing(energy: number): EasingFunction {
  if (energy < 0.3) return animationEasings.calm;
  if (energy < 0.7) return animationEasings.natural;
  return animationEasings.excited;
}

/**
 * Creates a custom easing based on character personality
 */
export function createPersonalityEasing(personality: {
  playfulness: number; // 0-1
  responsiveness: number; // 0-1
  calmness: number; // 0-1
}): EasingFunction {
  const { playfulness, responsiveness, calmness } = personality;

  if (playfulness > 0.7) {
    return easingUtils.overshoot(easings.easeOutBack, playfulness * 0.2);
  }

  if (responsiveness > 0.7) {
    return easings.easeOutQuart;
  }

  if (calmness > 0.7) {
    return easings.easeInOutSine;
  }

  return animationEasings.natural;
}
