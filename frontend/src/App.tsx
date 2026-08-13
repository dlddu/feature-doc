// Screen routing for the journey the docs describe (user-journey flow 1 → 2):
// S01 Credentials → S02 Repositories → S03 Connect Repository → S04 Analysis.
//
// Still a state machine rather than a router dependency, with one addition: S04 is
// *addressable* (`#/analyses/<id>`). AC1.5 requires that closing the app and coming
// back shows the same run (test/01 시나리오 5), and a screen you cannot navigate
// straight to cannot demonstrate that. The hash is enough — no history API, no
// server-side route table, no dependency.

import { useEffect, useState } from 'react';
import { AnalysisProgress } from './AnalysisProgress';
import { ConnectRepository } from './ConnectRepository';
import { CredentialsSetup } from './CredentialsSetup';
import { HomeRepositories } from './HomeRepositories';

type Screen = 'credentials' | 'home' | 'connect';

/** `#/analyses/<id>` → the id, or null for every other (or no) hash. */
export function analysisIdFromHash(hash: string): string | null {
  const match = /^#\/analyses\/([^/?#]+)$/.exec(hash);
  return match ? decodeURIComponent(match[1]) : null;
}

export function App() {
  const [screen, setScreen] = useState<Screen>('credentials');
  // Bumped on returning from S03 so the home list refetches the new job.
  const [homeEpoch, setHomeEpoch] = useState(0);
  const [analysisId, setAnalysisId] = useState<string | null>(() =>
    analysisIdFromHash(window.location.hash),
  );

  useEffect(() => {
    const onHashChange = () => setAnalysisId(analysisIdFromHash(window.location.hash));
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  }, []);

  function backToHome() {
    setHomeEpoch((n) => n + 1);
    setScreen('home');
  }

  /** Leaving S04 clears the hash, which is what re-renders the home screen. */
  function leaveAnalysis() {
    window.location.hash = '';
    setAnalysisId(null);
    backToHome();
  }

  function openAnalysis(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}`;
    setAnalysisId(id);
  }

  // The hash wins over the state machine: a deep link must land on S04 even on a
  // cold load, before the user has walked the journey in this session.
  if (analysisId !== null) {
    return <AnalysisProgress key={analysisId} id={analysisId} onBack={leaveAnalysis} />;
  }
  if (screen === 'home') {
    return (
      <HomeRepositories
        key={homeEpoch}
        onConnectRepository={() => setScreen('connect')}
        onOpenCredentials={() => setScreen('credentials')}
        onOpenAnalysis={openAnalysis}
      />
    );
  }
  if (screen === 'connect') {
    return <ConnectRepository onDone={backToHome} />;
  }
  return <CredentialsSetup onReady={() => setScreen('home')} />;
}
