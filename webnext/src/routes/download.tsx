import { createFileRoute } from '@tanstack/react-router';
import { ClientDownloadPage } from '../pages/ClientDownload/ClientDownloadPage';

export const Route = createFileRoute('/download')({
  component: ClientDownloadPage,
});
