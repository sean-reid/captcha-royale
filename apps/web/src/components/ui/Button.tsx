import type { CSSProperties, ReactNode, ButtonHTMLAttributes } from 'react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  children: ReactNode;
}

const variantStyles: Record<string, CSSProperties> = {
  primary: {
    background: 'linear-gradient(135deg, #ff6b6b, #ee5a24)',
    color: '#fff',
    border: 'none',
  },
  secondary: {
    background: 'transparent',
    color: '#e0e0e0',
    border: '1px solid #444',
  },
  danger: {
    background: '#c0392b',
    color: '#fff',
    border: 'none',
  },
};

const sizeStyles: Record<string, CSSProperties> = {
  sm: { padding: '6px 16px', fontSize: '13px' },
  md: { padding: '10px 24px', fontSize: '15px' },
  lg: { padding: '14px 36px', fontSize: '18px' },
};

export function Button({ variant = 'primary', size = 'md', children, style, ...props }: ButtonProps) {
  return (
    <button
      style={{
        ...baseStyle,
        ...variantStyles[variant],
        ...sizeStyles[size],
        ...style,
      }}
      {...props}
    >
      {children}
    </button>
  );
}

const baseStyle: CSSProperties = {
  borderRadius: '8px',
  cursor: 'pointer',
  fontWeight: 'bold',
  fontFamily: 'inherit',
  transition: 'opacity 0.15s',
};
