import './style.scss';

import { ReactNode } from 'react';

import { LogoContainer } from '../../components/LogoContainer/LogoContainer';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { usePasswordResetStore } from './hooks/usePasswordResetStore';
import { CodeStep } from './steps/CodeStep/CodeStep';
import { EmailStep } from './steps/EmailStep/EmailStep';
import { FailureStep } from './steps/FailureStep/FailureStep';
import { LinkSentStep } from './steps/LinkSentStep/LinkSentStep';
import { PasswordStep } from './steps/PasswordStep/PasswordStep';
import { SuccessStep } from './steps/SuccessStep/SuccessStep';

const steps: ReactNode[] = [
  <EmailStep key={1} />,
  <LinkSentStep key={2} />,
  <CodeStep key={3} />,
  <PasswordStep key={4} />,
  <SuccessStep key={5} />,
  <FailureStep key={6} />,
];

export const PasswordResetPage = () => {
  const currentStep = usePasswordResetStore((state) => state.step);

  return (
    <PageContainer id="password-reset">
      <LogoContainer />
      {steps[currentStep] ?? null}
    </PageContainer>
  );
};
