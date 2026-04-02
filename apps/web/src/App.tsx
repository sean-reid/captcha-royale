import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { DesktopGate } from './components/DesktopGate';
import { Header } from './components/layout/Header';
import { Footer } from './components/layout/Footer';
import { Home } from './pages/Home';
import { Play } from './pages/Play';
import { Queue } from './pages/Queue';
import { Match } from './pages/Match';
import { Results } from './pages/Results';
import { Profile } from './pages/Profile';
import { Leaderboard } from './pages/Leaderboard';
import { Login } from './pages/Login';
import { Playtest } from './pages/Playtest';

export function App() {
  return (
    <DesktopGate>
      <BrowserRouter basename={import.meta.env.BASE_URL}>
        <div style={styles.layout}>
          <Header />
          <main style={styles.main}>
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/play" element={<Play />} />
              <Route path="/queue" element={<Queue />} />
              <Route path="/match/:id" element={<Match />} />
              <Route path="/results" element={<Results />} />
              <Route path="/profile" element={<Profile />} />
              <Route path="/profile/:id" element={<Profile />} />
              <Route path="/leaderboard" element={<Leaderboard />} />
              <Route path="/login" element={<Login />} />
              {import.meta.env.DEV && (
                <Route path="/playtest" element={<Playtest />} />
              )}
            </Routes>
          </main>
          <Footer />
        </div>
      </BrowserRouter>
    </DesktopGate>
  );
}

const styles: Record<string, React.CSSProperties> = {
  layout: {
    display: 'flex',
    flexDirection: 'column',
    minHeight: '100vh',
  },
  main: {
    flex: 1,
  },
};
