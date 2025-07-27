import React from 'react';

export interface AnimationCanvasProps {
  width: number;
  height: number;
  visible?: boolean;
  onAnimationChange?: (animationName: string) => void;
  performanceMode?: 'active' | 'paused';
}

// Placeholder component - to be implemented
export const AnimationCanvas: React.FC<AnimationCanvasProps> = ({
  width,
  height,
  visible = true,
  onAnimationChange,
  performanceMode = 'active',
}) => {
  return (
    <div
      style={{
        width,
        height,
        backgroundColor: '#f0f0f0',
        border: '1px solid #ccc',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '12px',
        color: '#666',
      }}
    >
      Animation Canvas ({width}x{height}){!visible && ' - Hidden'}
    </div>
  );
};

export default AnimationCanvas;
