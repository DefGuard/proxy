import { createRootRoute, Outlet } from '@tanstack/react-router';
import { SessionGuard } from '../app/SessionGuard';

export const Route = createRootRoute({
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
