import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { extractTokenFromHash } from './lib/session';

// Pick up session token from OAuth redirect hash
extractTokenFromHash();

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
