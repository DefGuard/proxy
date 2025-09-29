import { createFileRoute } from '@tanstack/react-router';
import { HomePage } from '../pages/Home/HomePage';
import { useEnrollmentStore } from '../shared/hooks/useEnrollmentStore';

export const Route = createFileRoute('/')({
  component: HomePage,
  loader: () => {
    useEnrollmentStore.getState().reset();
  },
});
