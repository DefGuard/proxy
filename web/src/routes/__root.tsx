import { createRootRoute, Outlet, redirect } from '@tanstack/react-router';
import { SessionGuard } from '../app/SessionGuard';
import { api } from '../shared/api/api';

export const Route = createRootRoute({
  beforeLoad: async ({ location }) => {
    if (location.pathname === '/server-warning') {
      return;
    }

    const response = await api.appInfo.callbackFn({ params: undefined });
    if (response.data.server_state !== 'connected') {
      throw redirect({
        to: '/server-warning',
        replace: true,
      });
    }
  },
  component: RootComponent,
});

function RootComponent() {
  return (
    <>
      <Outlet />
      <SessionGuard />
    </>
  );
}
