import { createFileRoute } from '@tanstack/react-router';
import { api } from '../shared/api/api';

export const Route = createFileRoute('/server-warning')({
  loader: async () => {
    const response = await api.appInfo.callbackFn({ params: undefined });
    return response.data.server_state;
  },
  component: ServerWarningPage,
});

function ServerWarningPage() {
  const serverState = Route.useLoaderData();

  return <main>{serverState}</main>;
}
