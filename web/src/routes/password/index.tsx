import { createFileRoute, redirect } from '@tanstack/react-router';
import { queryClient } from '../../app/query';
import { PasswordStartPage } from '../../pages/PasswordStart/PasswordStartPage';
import { getAppInfoQueryOptions } from '../../shared/query/queryOptions';

export const Route = createFileRoute('/password/')({
  component: PasswordStartPage,
  loader: async () => {
    const appInfo = await queryClient.ensureQueryData(getAppInfoQueryOptions);
    if (!appInfo.data.display_password_reset) {
      throw redirect({
        to: '/',
        replace: true,
      });
    }
  },
});
