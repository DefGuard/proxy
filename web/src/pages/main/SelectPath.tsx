import { useNavigate } from 'react-router-dom';
import { shallow } from 'zustand/shallow';

import { Card } from '../../shared/components/layout/Card/Card';
import SvgEnrollmentSelectGraphic from '../../shared/components/svg/EnrollmentSelectGraphic';
import SvgPasswordResetSelectGraphic from '../../shared/components/svg/PasswordResetSelectGraphic';
import { routes } from '../../shared/routes';
import { usePasswordResetStore } from '../passwordReset/hooks/usePasswordResetStore';
import { PathSelectCard } from './DeviceSetupMethodCard/DeviceSetupMethodCard';

export const SelectPath = () => {
  const navigate = useNavigate();
  const [reset] = usePasswordResetStore((state) => [state.reset], shallow);

  return (
    <Card shaded id="setup-method-step">
      <PathSelectCard
        testId="select-enrollment"
        onSelect={() => navigate(routes.token)}
        title="Enrollment process"
        subtitle="Confirm your new account"
        logo={<SvgEnrollmentSelectGraphic />}
      />

      <PathSelectCard
        testId="select-password-reset"
        onSelect={() => {
          reset();
          navigate(routes.passwordReset);
        }}
        title="Password reset"
        subtitle="Reset password for existing account"
        logo={<SvgPasswordResetSelectGraphic />}
      />
    </Card>
  );
};
