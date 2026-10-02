import { useLoaderData } from '@tanstack/react-router';
import { m } from '../../paraglide/messages';
import { PageInfo } from '../../shared/components/PageInfo/PageInfo';

export const ServerWarningPage = () => {
  const serverState = useLoaderData({ from: '/server-warning' });

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
      icon="warning-outlined"
      title={title}
      subtitle={subtitle}
      link="/"
      linkText={m.server_warning_retry()}
    />
  );
};
