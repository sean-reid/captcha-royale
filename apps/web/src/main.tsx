import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { extractTokenFromHash } from './lib/session';

// Pick up session token from OAuth redirect hash
extractTokenFromHash();

// GitHub Pages SPA redirect: restore the path from 404.html redirect
const redirectPath = sessionStorage.getItem('redirect');
if (redirectPath) {
  sessionStorage.removeItem('redirect');
  const base = import.meta.env.BASE_URL.replace(/\/$/, '');
  const route = redirectPath.startsWith(base) ? redirectPath.slice(base.length) : redirectPath;
  if (route && route !== '/') {
    history.replaceState(null, '', redirectPath);
  }
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
