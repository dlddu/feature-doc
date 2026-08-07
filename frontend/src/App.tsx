// Screen routing for the journey the docs describe (user-journey flow 1):
// S01 Credentials → S02 Repositories → S03 Connect Repository.
//
// Deliberately a state machine rather than a router dependency: the walking
// skeleton has three screens and no deep-link requirement yet. A URL router
// arrives when a screen needs to be addressable (S04 onwards).

import { useState } from 'react';
import { ConnectRepository } from './ConnectRepository';
import { CredentialsSetup } from './CredentialsSetup';
import { HomeRepositories } from './HomeRepositories';

type Screen = 'credentials' | 'home' | 'connect';

export function App() {
  const [screen, setScreen] = useState<Screen>('credentials');
  // Bumped on returning from S03 so the home list refetches the new job.
  const [homeEpoch, setHomeEpoch] = useState(0);

  function backToHome() {
    setHomeEpoch((n) => n + 1);
    setScreen('home');
  }

  if (screen === 'home') {
    return (
      <HomeRepositories
        key={homeEpoch}
        onConnectRepository={() => setScreen('connect')}
        onOpenCredentials={() => setScreen('credentials')}
      />
    );
  }
  if (screen === 'connect') {
    return <ConnectRepository onDone={backToHome} />;
  }
  return <CredentialsSetup onReady={() => setScreen('home')} />;
}
