self.addEventListener('push', (event) => {
  const message = event.data ? event.data.json() : {};
  event.waitUntil(
    self.registration.showNotification(message.title || 'FeatureDoc', {
      body: message.body || '',
      tag: message.tag,
    }),
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((windows) => {
      const open = windows.find((w) => 'focus' in w);
      return open ? open.focus() : self.clients.openWindow('/');
    }),
  );
});
