import { useNavigate } from '@tanstack/react-router';
import dayjs from 'dayjs';
import { useCallback, useEffect } from 'react';
import { useEnrollmentStore } from '../shared/hooks/useEnrollmentStore';

export const SessionGuard = () => {
  const navigate = useNavigate();
  const sessionEnd = useEnrollmentStore((s) => s.enrollmentData?.deadline_timestamp);

  const handleSessionEnd = useCallback(() => {
    navigate({
      to: '/session-end',
      replace: true,
    });
  }, [navigate]);

  useEffect(() => {
    if (!sessionEnd) return;

    const deadline = dayjs.unix(sessionEnd).diff(dayjs());
    if (deadline > 0) {
      const timeout = setTimeout(handleSessionEnd, deadline);
      return () => {
        clearTimeout(timeout);
      };
    } else {
      handleSessionEnd();
    }
  }, [sessionEnd, handleSessionEnd]);

  return null;
};
