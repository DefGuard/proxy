import { createFileRoute } from '@tanstack/react-router';
import { m } from '../paraglide/messages';
import { api } from '../shared/api/api';
import { PageInfo } from '../shared/components/PageInfo/PageInfo';

export const Route = createFileRoute('/server-warning')({
  loader: async () => {
    const response = await api.appInfo.callbackFn({ params: undefined });
    return response.data.server_state;
  },
  component: ServerWarningPage,
});

function ServerWarningPage() {
  const serverState = Route.useLoaderData();

  const title =
    serverState === 'setup'
      ? m.server_warning_setup_title()
      : m.server_warning_disconnected_title();
  const subtitle =
    serverState === 'setup'
      ? m.server_warning_setup_subtitle()
      : m.server_warning_disconnected_subtitle();

  return (
    <PageInfo
      icon="warning"
      title={title}
      subtitle={subtitle}
      link="/"
      linkText={m.server_warning_retry()}
    />
  );
}
