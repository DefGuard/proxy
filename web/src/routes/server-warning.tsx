import { createFileRoute } from '@tanstack/react-router';
import { api } from '../shared/api/api';
import { PageProcessEnd } from '../shared/components/PageProcessEnd/PageProcessEnd';

export const Route = createFileRoute('/server-warning')({
  loader: async () => {
    const response = await api.appInfo.callbackFn({ params: undefined });
    return response.data.server_state;
  },
  component: ServerWarningPage,
});

function ServerWarningPage() {
  const serverState = Route.useLoaderData();

  const title = serverState === 'setup' ? 'Server is in setup mode' : 'Core is disconnected';
  const subtitle =
    serverState === 'setup'
      ? 'Proxy setup is not complete yet. Most actions are unavailable until setup finishes.'
      : 'Proxy is configured, but it is not connected to Defguard Core. Try again in a moment.';

  return (
    <PageProcessEnd
      icon="warning"
      title={title}
      subtitle={subtitle}
      link="/"
      linkText="Try again"
    />
  );
}
