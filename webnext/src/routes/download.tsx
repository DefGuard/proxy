import { createFileRoute } from '@tanstack/react-router';
import { queryClient } from '../app/query';
import { ClientDownloadPage } from '../pages/ClientDownload/ClientDownloadPage';
import { getClientArtifactsQueryOptions } from '../shared/query/queryOptions';

export const Route = createFileRoute('/download')({
  component: ClientDownloadPage,
  loader: () => queryClient.ensureQueryData(getClientArtifactsQueryOptions),
});
