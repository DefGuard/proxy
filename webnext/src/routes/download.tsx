import { createFileRoute } from '@tanstack/react-router';
import { ClientDownloadPage } from '../pages/ClientDownload/ClientDownloadPage';
import { updateServiceApi } from '../shared/api/update-service';

export const Route = createFileRoute('/download')({
  component: ClientDownloadPage,
  loader: async () => {
    const clientVersionData = await updateServiceApi
      .getClientArtifacts()
      .catch(() => null);
    return clientVersionData;
  },
});
