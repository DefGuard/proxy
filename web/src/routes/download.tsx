import { createFileRoute, redirect } from '@tanstack/react-router';
import { queryClient } from '../app/query';
import { ClientDownloadPage } from '../pages/ClientDownload/ClientDownloadPage';
import { useEnrollmentStore } from '../shared/hooks/useEnrollmentStore';
import {
  getAppInfoQueryOptions,
  getClientArtifactsQueryOptions,
} from '../shared/query/queryOptions';

export const Route = createFileRoute('/download')({
  component: ClientDownloadPage,
  loaderDeps: () => {
    const storeState = useEnrollmentStore.getState();
    if (storeState.enrollmentData === undefined || storeState.token === undefined) {
      throw redirect({
        to: '/',
        replace: true,
      });
    }
    return {
      enrollmentState: {
        token: storeState.token,
        enrollmentData: storeState.enrollmentData,
      },
    };
  },
  loader: async ({ deps }) => {
    void queryClient.ensureQueryData(getClientArtifactsQueryOptions);
    // Redirect to client-setup if the download step is hidden.
    const appInfo = await queryClient.ensureQueryData(getAppInfoQueryOptions);
    if (!appInfo.data.display_download_step) {
      throw redirect({
        to: '/client-setup',
        replace: true,
      });
    }
    return deps;
  },
});
