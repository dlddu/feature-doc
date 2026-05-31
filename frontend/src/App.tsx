// App shell + routing. Two routes only: `/` is the S02 Repositories home, `/setup`
// is the S01 Credentials Setup. A setup gate decides which one a visitor lands on:
// until credentials are ready (GitHub App installed + an active LLM key) `/`
// redirects to `/setup`; once ready, `/` shows the home.

import { useEffect, useState } from 'react';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { CredentialsSetup } from './CredentialsSetup';
import { HomeRepositories } from './HomeRepositories';
import { getConnection, getMe, listKeys } from './api';

type Readiness = 'loading' | 'ready' | 'not-ready';

/** `/` — show the home only when credentials are ready, else fall back to setup. */
function HomeGate() {
  const [readiness, setReadiness] = useState<Readiness>('loading');

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const user = await getMe();
        if (!user) {
          if (!cancelled) setReadiness('not-ready');
          return;
        }
        const [connection, keys] = await Promise.all([getConnection(), listKeys()]);
        const ready = connection.installed && keys.some((k) => k.status === 'active');
        if (!cancelled) setReadiness(ready ? 'ready' : 'not-ready');
      } catch {
        if (!cancelled) setReadiness('not-ready');
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (readiness === 'loading') {
    return (
      <main className="screen">
        <p className="body sm" style={{ marginTop: 28 }}>
          불러오는 중…
        </p>
      </main>
    );
  }
  if (readiness === 'not-ready') {
    return <Navigate to="/setup" replace />;
  }
  return <HomeRepositories />;
}

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<HomeGate />} />
        <Route path="/setup" element={<CredentialsSetup />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
