import { createFileRoute } from '@tanstack/react-router';
import { SessionEndPage } from '../pages/SessionEnd/SessionEndPage';
import { useEnrollmentStore } from '../shared/hooks/useEnrollmentStore';

export const Route = createFileRoute('/session-end')({
  component: SessionEndPage,
  loader: () => {
    useEnrollmentStore.getState().reset();
  },
});
