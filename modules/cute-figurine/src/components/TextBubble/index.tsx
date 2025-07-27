import React from 'react';
import type { Message } from '../../types';

export interface TextBubbleProps {
  message: Message;
  position: 'top' | 'bottom' | 'left' | 'right';
  onDismiss?: () => void;
  onInteraction?: () => void;
}

// Placeholder component - to be implemented
export const TextBubble: React.FC<TextBubbleProps> = ({
  message,
  position,
  onDismiss,
  onInteraction,
}) => {
  return (
    <div
      style={{
        position: 'absolute',
        [position]: position === 'top' ? '-50px' : position === 'bottom' ? '100%' : '50%',
        left: position === 'left' ? '-200px' : position === 'right' ? '100%' : '50%',
        transform: 'translateX(-50%)',
        backgroundColor: '#fff',
        border: '1px solid #ccc',
        borderRadius: '8px',
        padding: '8px 12px',
        fontSize: '14px',
        boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
        cursor: 'pointer',
        maxWidth: '200px',
        wordWrap: 'break-word',
        whiteSpace: 'normal',
      }}
      onClick={onInteraction}
    >
      {message.text}
      {onDismiss && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDismiss();
          }}
          style={{
            marginLeft: '8px',
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            fontSize: '12px',
          }}
        >
          ×
        </button>
      )}
    </div>
  );
};

export default TextBubble;
