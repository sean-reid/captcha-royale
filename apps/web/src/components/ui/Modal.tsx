import type { ReactNode, CSSProperties } from 'react';

interface ModalProps {
  open: boolean;
  onClose: () => void;
  children: ReactNode;
  title?: string;
}

export function Modal({ open, onClose, children, title }: ModalProps) {
  if (!open) return null;

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
        {title && <h2 style={styles.title}>{title}</h2>}
        {children}
      </div>
    </div>
  );
}

const styles: Record<string, CSSProperties> = {
  overlay: {
    position: 'fixed',
    inset: 0,
    background: 'rgba(0,0,0,0.7)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modal: {
    background: '#16213e',
    borderRadius: '12px',
    padding: '32px',
    maxWidth: '500px',
    width: '90%',
    border: '1px solid #2a2a4e',
  },
  title: {
    color: '#e0e0e0',
    margin: '0 0 16px',
    fontSize: '20px',
  },
};
