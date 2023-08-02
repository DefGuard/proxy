import './style.scss';

import dayjs from 'dayjs';
import { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

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

  const { token } = useParams();

  // check if navigated from link with token if not do noting
  useEffect(() => {
    if (token && token.length) {
      startEnrollment({
        token,
      })
        .then((res) => {
          const sessionEndDate = dayjs.unix(res.deadline_timestamp).toDate();
          initEnrollment({
            step: 0,
            userInfo: res.user,
            adminInfo: res.admin,
            sessionEnd: sessionEndDate.toISOString(),
            vpnOptional: res.vpn_setup_optional,
            endContent: res.final_page_content,
          });
          navigate(routes.enrollment, { replace: true });
        })
        .catch(() => {
          navigate(routes.token, { replace: true });
        });
    } else {
      navigate(routes.token, { replace: true });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return <PageContainer id="main-page"></PageContainer>;
};
