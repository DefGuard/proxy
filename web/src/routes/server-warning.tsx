import { createFileRoute } from '@tanstack/react-router';
import { ServerWarningPage } from '../pages/ServerWarning/ServerWarningPage';
import { api } from '../shared/api/api';

export const Route = createFileRoute('/server-warning')({
  loader: async () => {
    const response = await api.appInfo.callbackFn({});
    return response.data.server_state;
  },
  component: ServerWarningPage,
});
