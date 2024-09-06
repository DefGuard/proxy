import './style.scss';

import dayjs from 'dayjs';
import { useEffect, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { LogoContainer } from '../../components/LogoContainer/LogoContainer';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { useApi } from '../../shared/hooks/api/useApi';
import { routes } from '../../shared/routes';
import { useEnrollmentStore } from '../enrollment/hooks/store/useEnrollmentStore';
import { SelectPath } from './SelectPath';

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
            endContent: res.final_page_content,
            enrollmentSettings: res.settings,
          });
          navigate(routes.enrollment, { replace: true });
        })
        .catch(() => {
          requestPending.current = false;
          navigate(routes.token, { replace: true });
        });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams]);

  return (
    <PageContainer id="main-page">
      <LogoContainer />
      <SelectPath />
    </PageContainer>
  );
};
