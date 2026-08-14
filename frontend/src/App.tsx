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
import { CrossCuttingConcerns } from './CrossCuttingConcerns';
import { ConnectRepository } from './ConnectRepository';
import { CredentialsSetup } from './CredentialsSetup';
import { HomeRepositories } from './HomeRepositories';

type Screen = 'credentials' | 'home' | 'connect';

/** Which analysis screen a hash addresses, if any. */
export type AnalysisRoute = { id: string; view: 'progress' | 'cross-cutting' };

/**
 * `#/analyses/<id>` → S04, `#/analyses/<id>/cross-cutting` → S05, null otherwise.
 *
 * Both are addressable for the same reason S04 is (AC1.5): a screen you cannot
 * navigate straight to cannot demonstrate that its content is server state.
 */
export function analysisRouteFromHash(hash: string): AnalysisRoute | null {
  const match = /^#\/analyses\/([^/?#]+)(\/cross-cutting)?$/.exec(hash);
  if (!match) return null;
  return {
    id: decodeURIComponent(match[1]),
    view: match[2] ? 'cross-cutting' : 'progress',
  };
}

export function App() {
  const [screen, setScreen] = useState<Screen>('credentials');
  // Bumped on returning from S03 so the home list refetches the new job.
  const [homeEpoch, setHomeEpoch] = useState(0);
  const [route, setRoute] = useState<AnalysisRoute | null>(() =>
    analysisRouteFromHash(window.location.hash),
  );

  useEffect(() => {
    const onHashChange = () => setRoute(analysisRouteFromHash(window.location.hash));
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
    setRoute(null);
    backToHome();
  }

  function openAnalysis(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}`;
    setRoute({ id, view: 'progress' });
  }

  /** S04 → S05, for a run whose cross-cutting stage has produced its document. */
  function openCrossCutting(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/cross-cutting`;
    setRoute({ id, view: 'cross-cutting' });
  }

  // The hash wins over the state machine: a deep link must land on S04/S05 even on
  // a cold load, before the user has walked the journey in this session.
  if (route !== null) {
    if (route.view === 'cross-cutting') {
      return (
        <CrossCuttingConcerns
          key={`${route.id}-cc`}
          id={route.id}
          onBack={() => openAnalysis(route.id)}
        />
      );
    }
    return (
      <AnalysisProgress
        key={route.id}
        id={route.id}
        onBack={leaveAnalysis}
        onOpenCrossCutting={() => openCrossCutting(route.id)}
      />
    );
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
