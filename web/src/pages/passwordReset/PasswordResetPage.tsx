import './style.scss';

import dayjs from 'dayjs';
import { ReactNode, useEffect, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { LogoContainer } from '../../components/LogoContainer/LogoContainer';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { useApi } from '../../shared/hooks/api/useApi';
import { routes } from '../../shared/routes';
import { usePasswordResetStore } from './hooks/usePasswordResetStore';
import { EmailStep } from './steps/EmailStep/EmailStep';
import { FailureStep } from './steps/FailureStep/FailureStep';
import { LinkSentStep } from './steps/LinkSentStep/LinkSentStep';
import { PasswordStep } from './steps/PasswordStep/PasswordStep';
import { SuccessStep } from './steps/SuccessStep/SuccessStep';

const steps: ReactNode[] = [
  <EmailStep key={1} />,
  <LinkSentStep key={2} />,
  // <CodeStep key={3} />,
  <PasswordStep key={3} />,
  <SuccessStep key={4} />,
  <FailureStep key={5} />,
];

export const PasswordResetPage = () => {
  const navigate = useNavigate();
  const currentStep = usePasswordResetStore((state) => state.step);
  const setStore = usePasswordResetStore((state) => state.setState);

  const {
    passwordReset: { start },
  } = useApi();

  const [searchParams] = useSearchParams();

  const requestPending = useRef(false);

  useEffect(() => {
    const token = searchParams.get('token');
    if (token && token.length && !requestPending.current) {
      requestPending.current = true;
      start({
        token,
      })
        .then((res) => {
          const sessionEnd = dayjs.unix(res.deadline_timestamp).utc().local().format();
          const sessionStart = dayjs().local().format();
          setStore({
            step: 2,
            sessionStart,
            sessionEnd,
          });
          navigate(routes.passwordReset, { replace: true });
        })
        .catch(() => {
          requestPending.current = false;
          setStore({
            step: 4,
          });
          navigate(routes.passwordReset, { replace: true });
        });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams]);

  return (
    <PageContainer id="password-reset">
      <LogoContainer />
      {steps[currentStep] ?? null}
    </PageContainer>
  );
};
