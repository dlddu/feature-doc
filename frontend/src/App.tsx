// Screen routing for the journey the docs describe (user-journey flow 1 → 2):
// Credentials → Repositories → Connect Repository → Analysis.
//
// Still a state machine rather than a router dependency, with one addition: Analysis Progress is
// *addressable* (`#/analyses/<id>`). AC1.5 requires that closing the app and coming
// back shows the same run (test/01 시나리오 5), and a screen you cannot navigate
// straight to cannot demonstrate that. The hash is enough — no history API, no
// server-side route table, no dependency.

import { useEffect, useState } from 'react';
import { AnalysisProgress } from './AnalysisProgress';
import { CrossCuttingConcerns } from './CrossCuttingConcerns';
import { DiscoveryStrategy } from './DiscoveryStrategy';
import { FeatureCandidates } from './FeatureCandidates';
import { ConnectRepository } from './ConnectRepository';
import { CredentialsSetup } from './CredentialsSetup';
import { HomeRepositories } from './HomeRepositories';

type Screen = 'credentials' | 'home' | 'connect';

/** Which analysis screen a hash addresses, if any. */
export type AnalysisRoute = {
  id: string;
  view: 'progress' | 'cross-cutting' | 'discovery-strategy' | 'candidates';
};

/**
 * `#/analyses/<id>` → Analysis Progress, `.../cross-cutting` → Cross-cutting Concerns, `.../discovery-strategy` → Discovery Strategy,
 * `.../candidates` → Feature Candidates, null otherwise.
 *
 * All four are addressable for the same reason Analysis Progress is (AC1.5): a screen you cannot
 * navigate straight to cannot demonstrate that its content is server state. For Discovery Strategy
 * and Feature Candidates it is also what makes the review resumable — a reviewer who leaves
 * mid-decision comes back to the list the server has, not to a lost draft, which is
 * the whole of the mockup's "여기까지 저장하고 나가기".
 */
export function analysisRouteFromHash(hash: string): AnalysisRoute | null {
  const match =
    /^#\/analyses\/([^/?#]+)(?:\/(cross-cutting|discovery-strategy|candidates))?$/.exec(hash);
  if (!match) return null;
  return {
    id: decodeURIComponent(match[1]),
    view: (match[2] as AnalysisRoute['view'] | undefined) ?? 'progress',
  };
}

export function App() {
  const [screen, setScreen] = useState<Screen>('credentials');
  // Bumped on returning from Connect Repository so the home list refetches the new job.
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

  /** Leaving Analysis Progress clears the hash, which is what re-renders the home screen. */
  function leaveAnalysis() {
    window.location.hash = '';
    setRoute(null);
    backToHome();
  }

  function openAnalysis(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}`;
    setRoute({ id, view: 'progress' });
  }

  /** Analysis Progress → Cross-cutting Concerns, for a run whose cross-cutting stage has produced its document. */
  function openCrossCutting(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/cross-cutting`;
    setRoute({ id, view: 'cross-cutting' });
  }

  /** Analysis Progress → Discovery Strategy, for a run whose discovery-strategy stage has proposed one (AC1.3). */
  function openDiscoveryStrategy(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/discovery-strategy`;
    setRoute({ id, view: 'discovery-strategy' });
  }

  // Discovery Strategy → Feature Candidates. The mockup wires it exactly here: `이 전략으로 후보 뽑기` carries
  // `data-goto="STP-sift-candidates"`, so the button that approves a strategy is
  // also the way into the list it produces. Analysis Progress draws no entry of its own.
  /** Discovery Strategy → Feature Candidates, for a run whose feature-candidates stage has extracted a list (AC1.4). */
  function openCandidates(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/candidates`;
    setRoute({ id, view: 'candidates' });
  }

  // The hash wins over the state machine: a deep link must land on Analysis Progress or Cross-cutting Concerns even on
  // a cold load, before the user has walked the journey in this session.
  if (route !== null) {
    if (route.view === 'candidates') {
      return (
        <FeatureCandidates
          key={`${route.id}-fc`}
          id={route.id}
          onBack={() => openAnalysis(route.id)}
        />
      );
    }
    if (route.view === 'discovery-strategy') {
      return (
        <DiscoveryStrategy
          key={`${route.id}-ds`}
          id={route.id}
          onBack={() => openAnalysis(route.id)}
          onOpenCandidates={() => openCandidates(route.id)}
        />
      );
    }
    if (route.view === 'cross-cutting') {
      return (
        <CrossCuttingConcerns
          key={`${route.id}-cc`}
          id={route.id}
          onBack={() => openAnalysis(route.id)}
          onOpenDiscoveryStrategy={() => openDiscoveryStrategy(route.id)}
        />
      );
    }
    return (
      <AnalysisProgress
        key={route.id}
        id={route.id}
        onBack={leaveAnalysis}
        onOpenCrossCutting={() => openCrossCutting(route.id)}
        onOpenDiscoveryStrategy={() => openDiscoveryStrategy(route.id)}
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
