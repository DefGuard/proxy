import { createFileRoute } from '@tanstack/react-router';
import { ConfigureClientPage } from '../pages/enrollment/ConfigureClient/ConfigureClientPage';

export const Route = createFileRoute('/client-setup')({
  component: ConfigureClientPage,
});
