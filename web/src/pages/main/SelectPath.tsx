import { useNavigate } from 'react-router-dom';
import { shallow } from 'zustand/shallow';

import { Card } from '../../shared/components/layout/Card/Card';
import EnrollmentSelectGraphic from '../../shared/components/svg/EnrollmentSelectGraphic';
import PasswordResetSelectGraphic from '../../shared/components/svg/PasswordResetSelectGraphic';
import { routes } from '../../shared/routes';
import { usePasswordResetStore } from '../passwordReset/hooks/usePasswordResetStore';
import { PathSelectCard } from './DeviceSetupMethodCard/DeviceSetupMethodCard';

export const SelectPath = () => {
  const navigate = useNavigate();
  const [reset] = usePasswordResetStore((state) => [state.reset], shallow);

  return (
    <Card shaded id="setup-method-step">
      <PathSelectCard
        onSelect={() => navigate(routes.token)}
        title="Enrollment process"
        subtitle="Confirm your new account"
        logo={<EnrollmentSelectGraphic />}
      />

      <PathSelectCard
        onSelect={() => {
          reset();
          navigate(routes.passwordReset);
        }}
        title="Password reset"
        subtitle="Reset password for existing account"
        logo={<PasswordResetSelectGraphic />}
      />
    </Card>
  );
};
