import { useEffect, useState } from 'react';

type HelloResponse = { message: string };

type State =
  | { kind: 'loading' }
  | { kind: 'ready'; message: string }
  | { kind: 'error'; detail: string };

export function App() {
  const [state, setState] = useState<State>({ kind: 'loading' });

  useEffect(() => {
    const controller = new AbortController();

    fetch('/hello', { signal: controller.signal })
      .then(async (res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return (await res.json()) as HelloResponse;
      })
      .then((data) => {
        setState({ kind: 'ready', message: data.message });
      })
      .catch((err: unknown) => {
        if (controller.signal.aborted) return;
        setState({
          kind: 'error',
          detail: err instanceof Error ? err.message : String(err),
        });
      });

    return () => controller.abort();
  }, []);

  return (
    <main className="screen">
      {state.kind === 'loading' && (
        <p className="body-small">Reaching backend…</p>
      )}
      {state.kind === 'ready' && (
        <h1 className="h-display">{state.message}</h1>
      )}
      {state.kind === 'error' && (
        <p className="body-small">
          Could not reach backend: {state.detail}
        </p>
      )}
    </main>
  );
}
