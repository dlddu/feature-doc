import { getPushKey, savePushSubscription } from './api';

let serverKey: string | null | undefined;

function supported(): boolean {
  return 'serviceWorker' in navigator && 'PushManager' in window && 'Notification' in window;
}

export function preparePush(): void {
  if (!supported() || serverKey !== undefined) return;
  getPushKey().then(
    (key) => {
      serverKey = key;
    },
    () => {
      serverKey = null;
    },
  );
}

function decodeKey(key: string): Uint8Array<ArrayBuffer> {
  const base64 = key.replace(/-/g, '+').replace(/_/g, '/');
  const raw = atob(base64 + '='.repeat((4 - (base64.length % 4)) % 4));
  const bytes = new Uint8Array(new ArrayBuffer(raw.length));
  for (let i = 0; i < raw.length; i++) bytes[i] = raw.charCodeAt(i);
  return bytes;
}

export async function enablePush(): Promise<void> {
  const key = serverKey;
  if (!supported() || !key) return;
  try {
    const permission =
      Notification.permission === 'default'
        ? await Notification.requestPermission()
        : Notification.permission;
    if (permission !== 'granted') return;
    const registration = await navigator.serviceWorker.register('/sw.js');
    const subscription =
      (await registration.pushManager.getSubscription()) ??
      (await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: decodeKey(key),
      }));
    await savePushSubscription(subscription.toJSON());
  } catch {
    return;
  }
}
