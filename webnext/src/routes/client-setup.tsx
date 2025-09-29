import { createFileRoute, redirect } from '@tanstack/react-router';
import { ConfigureClientPage } from '../pages/enrollment/ConfigureClient/ConfigureClientPage';
import type { EnrollmentStartResponse } from '../shared/api/types';
import { useEnrollmentStore } from '../shared/hooks/useEnrollmentStore';

const _fetchDesktop = async () => {
  const _resp = await fetch('https://api.git.com/repos/defguard/client/releases/latest', {
    method: 'GET',
  });
};

export const Route = createFileRoute('/client-setup')({
  component: ConfigureClientPage,
  beforeLoad: () => {
    const state = useEnrollmentStore.getState();
    if (state.token === undefined || state.enrollmentData === undefined) {
      throw redirect({
        to: '/enrollment-start',
        replace: true,
      });
    }
  },
  loader: () => {
    const state = useEnrollmentStore.getState();
    return {
      token: state.token as string,
      enrollmentData: state.enrollmentData as EnrollmentStartResponse,
    };
  },
});
