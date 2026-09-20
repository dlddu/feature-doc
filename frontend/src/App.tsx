// A state machine rather than a router dependency, with one addition: the analysis
// screens are *addressable* (`#/analyses/<id>`), because a screen you cannot
// navigate straight to cannot demonstrate that its content is server state. A hash
// is enough — no history API, no server-side route table, no dependency.

import { useEffect, useState } from 'react';
import { AnalysisDiff } from './AnalysisDiff';
import { AnalysisProgress } from './AnalysisProgress';
import { CrossCuttingConcerns } from './CrossCuttingConcerns';
import { DiscoveryStrategy } from './DiscoveryStrategy';
import { FeatureAcceptance } from './FeatureAcceptance';
import { FeatureCandidates } from './FeatureCandidates';
import { FeatureDependencies } from './FeatureDependencies';
import { GrantRepoAccess } from './GrantRepoAccess';
import { HomeRepositories } from './HomeRepositories';
import { RegisterLlmKey } from './RegisterLlmKey';

type Screen = 'grant' | 'key' | 'home';

export type AnalysisRoute = {
  id: string;
  view:
    | 'progress'
    | 'cross-cutting'
    | 'discovery-strategy'
    | 'candidates'
    | 'acceptance'
    | 'dependencies'
    | 'diff';
  featureKey?: string;
};

export function analysisRouteFromHash(hash: string): AnalysisRoute | null {
  const traced = /^#\/analyses\/([^/?#]+)\/features\/([^/?#]+)\/dependencies$/.exec(hash);
  if (traced) {
    return {
      id: decodeURIComponent(traced[1]),
      view: 'dependencies',
      featureKey: decodeURIComponent(traced[2]),
    };
  }
  const match =
    /^#\/analyses\/([^/?#]+)(?:\/(cross-cutting|discovery-strategy|candidates|acceptance|diff))?$/.exec(
      hash,
    );
  if (!match) return null;
  return {
    id: decodeURIComponent(match[1]),
    view: (match[2] as AnalysisRoute['view'] | undefined) ?? 'progress',
  };
}

export function App() {
  const [screen, setScreen] = useState<Screen>('grant');
  // 권한 부여 hands off to 키 등록 the moment an installation exists, so walking
  // *back* into it must suppress the hand-off — otherwise the two screens bounce.
  const [handOff, setHandOff] = useState(true);
  // Bumped after a run is queued so the home list refetches the new job.
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

  function openCredentials() {
    setHandOff(true);
    setScreen('grant');
  }

  function backToGrant() {
    setHandOff(false);
    setScreen('grant');
  }

  /**
   * The hash is cleared first: an analysis route left behind would otherwise
   * re-render a signed-in screen over the entry one.
   */
  function afterLogout() {
    window.location.hash = '';
    setRoute(null);
    setHomeEpoch((n) => n + 1);
    setHandOff(true);
    setScreen('grant');
  }

  function leaveAnalysis() {
    window.location.hash = '';
    setRoute(null);
    backToHome();
  }

  function openAnalysis(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}`;
    setRoute({ id, view: 'progress' });
  }

  function openCrossCutting(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/cross-cutting`;
    setRoute({ id, view: 'cross-cutting' });
  }

  function openDiscoveryStrategy(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/discovery-strategy`;
    setRoute({ id, view: 'discovery-strategy' });
  }

  function openCandidates(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/candidates`;
    setRoute({ id, view: 'candidates' });
  }

  function openDiff(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/diff`;
    setRoute({ id, view: 'diff' });
  }

  function openAcceptance(id: string) {
    window.location.hash = `#/analyses/${encodeURIComponent(id)}/acceptance`;
    setRoute({ id, view: 'acceptance' });
  }

  // The hash is read before the state machine: a deep link must land on its screen
  // even on a cold load, before the user has walked the journey in this session.
  if (route !== null) {
    if (route.view === 'dependencies' && route.featureKey !== undefined) {
      return (
        <FeatureDependencies
          key={`${route.id}-fd`}
          id={route.id}
          featureKey={route.featureKey}
          onBack={() => openAcceptance(route.id)}
        />
      );
    }
    if (route.view === 'diff') {
      return (
        <AnalysisDiff
          key={`${route.id}-ad`}
          id={route.id}
          onBack={() => openAnalysis(route.id)}
          onOpenFeature={() => openAcceptance(route.id)}
        />
      );
    }
    if (route.view === 'acceptance') {
      return (
        <FeatureAcceptance
          key={`${route.id}-fa`}
          id={route.id}
          onBack={() => openAnalysis(route.id)}
          onOpenCandidates={() => openCandidates(route.id)}
        />
      );
    }
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
        onOpenDiff={() => openDiff(route.id)}
      />
    );
  }
  if (screen === 'home') {
    return (
      <HomeRepositories
        key={homeEpoch}
        onOpenCredentials={openCredentials}
        onOpenAnalysis={openAnalysis}
        onLoggedOut={afterLogout}
        onAnalysisQueued={backToHome}
      />
    );
  }
  if (screen === 'key') {
    return <RegisterLlmKey onBack={backToGrant} onReady={() => setScreen('home')} />;
  }
  return (
    <GrantRepoAccess
      onInstalled={() => {
        if (handOff) setScreen('key');
      }}
    />
  );
}
