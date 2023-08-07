import './style.scss';

import dayjs from 'dayjs';
import { useEffect, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { useApi } from '../../shared/hooks/api/useApi';
import { routes } from '../../shared/routes';
import { useEnrollmentStore } from '../enrollment/hooks/store/useEnrollmentStore';

export const MainPage = () => {
  const navigate = useNavigate();
  const {
    enrollment: { start: startEnrollment },
  } = useApi();

  const initEnrollment = useEnrollmentStore((state) => state.init);

  const [searchParams] = useSearchParams();

  const requestPending = useRef(false);

  // check if navigated from link with token if not do nothing
  useEffect(() => {
    const token = searchParams.get('token');
    if (token && token.length && !requestPending.current) {
      requestPending.current = true;
      startEnrollment({
        token,
      })
        .then((res) => {
          const sessionEnd = dayjs.unix(res.deadline_timestamp).utc().local().format();
          const sessionStart = dayjs().local().format();
          initEnrollment({
            step: 0,
            userInfo: res.user,
            adminInfo: res.admin,
            sessionStart,
            sessionEnd,
            vpnOptional: res.vpn_setup_optional,
            endContent: res.final_page_content,
          });
          navigate(routes.enrollment, { replace: true });
        })
        .catch(() => {
          requestPending.current = false;
          navigate(routes.token, { replace: true });
        });
    }

    if (!token && !requestPending.current) {
      navigate(routes.token, { replace: true });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams]);

  return <PageContainer id="main-page"></PageContainer>;
};
